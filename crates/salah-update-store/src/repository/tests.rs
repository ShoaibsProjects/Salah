use super::*;
use ed25519_dalek::{Signer, SigningKey};
use salah_location::BOUNDARY_DATA_SHA256;
use salah_update::{TrustedPublicKey, candidate_signing_bytes};

const MANIFEST: &[u8] = include_bytes!("../../../salah-time/fixtures/global/manifest.json");
const PACK: &[u8] = include_bytes!("../../../salah-time/fixtures/global/tzif-pack.bin");
const LICENSE: &[u8] = include_bytes!("../../../salah-time/fixtures/global/IANA-LICENSE");

struct PrivateRoot(PathBuf);
impl PrivateRoot {
    fn new() -> Self {
        let path = new_stage_directory(&std::env::temp_dir()).unwrap();
        Self(path)
    }
}
impl Drop for PrivateRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn key() -> SigningKey {
    SigningKey::from_bytes(&[11; 32])
}
fn trust() -> TrustStore {
    TrustStore::new([TrustedPublicKey {
        key_id: "repository-test-only".to_owned(),
        bytes: key().verifying_key().to_bytes(),
    }])
    .unwrap()
}
fn candidate(sequence: u64, manifest: &[u8]) -> SignedPackCandidate<'_> {
    let mut candidate = SignedPackCandidate {
        key_id: "repository-test-only",
        sequence,
        boundary_sha256: BOUNDARY_DATA_SHA256,
        manifest_bytes: manifest,
        pack_bytes: PACK,
        license_bytes: LICENSE,
        signature: [0; 64],
    };
    candidate.signature = key()
        .sign(&candidate_signing_bytes(&candidate).unwrap())
        .to_bytes();
    candidate
}
fn install_confirmed(repo: &mut RulePackRepository<'_>, sequence: u64) {
    repo.begin_trial(candidate(sequence, MANIFEST)).unwrap();
    repo.confirm_trial(sequence).unwrap();
}

#[test]
fn confirmed_pack_survives_reopen_with_exact_signed_bytes() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert_eq!(repo.status().unwrap().high_water_sequence, 0);
    assert!(repo.selected_pack().unwrap().is_none());
    install_confirmed(&mut repo, 1);
    drop(repo);
    let repo = RulePackRepository::open(&root.0, &keys).unwrap();
    let status = repo.status().unwrap();
    assert_eq!(status.high_water_sequence, 1);
    assert_eq!(status.selected.unwrap().sequence, 1);
    assert!(!status.awaiting_confirmation);
    let stored = repo.selected_pack().unwrap().unwrap();
    assert_eq!(stored.manifest_bytes(), MANIFEST);
    assert_eq!(stored.pack_bytes(), PACK);
    assert_eq!(stored.license_bytes(), LICENSE);
    assert_eq!(stored.signature(), &candidate(1, MANIFEST).signature);
}

#[test]
fn interrupted_trial_restores_prior_pack_without_resetting_high_water() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    install_confirmed(&mut repo, 1);
    repo.begin_trial(candidate(2, MANIFEST)).unwrap();
    drop(repo);
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert_eq!(
        repo.status().unwrap().recovery,
        RepositoryRecovery::InterruptedTrial {
            rejected_sequence: 2,
            restored_sequence: Some(1)
        }
    );
    assert_eq!(repo.selected_pack().unwrap().unwrap().sequence(), 1);
    assert_eq!(repo.status().unwrap().high_water_sequence, 2);
    assert!(matches!(
        repo.begin_trial(candidate(2, MANIFEST)),
        Err(StoreError::Verification(UpdateError::SequenceNotNewer))
    ));
    install_confirmed(&mut repo, 3);
}

#[test]
fn interrupted_first_trial_returns_to_compiled_pack_and_retains_counter() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    repo.begin_trial(candidate(1, MANIFEST)).unwrap();
    drop(repo);
    let repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert!(repo.selected_pack().unwrap().is_none());
    assert_eq!(repo.status().unwrap().high_water_sequence, 1);
    assert_eq!(
        repo.status().unwrap().recovery,
        RepositoryRecovery::InterruptedTrial {
            rejected_sequence: 1,
            restored_sequence: None
        }
    );
}

#[test]
fn archive_written_before_state_commit_can_be_reused_exactly() {
    let root = PrivateRoot::new();
    let keys = trust();
    let repo = RulePackRepository::open(&root.0, &keys).unwrap();
    let verified = keys.verify_candidate(candidate(1, MANIFEST), 0).unwrap();
    repo.persist_archive(&verified).unwrap();
    drop(repo);
    // This simulates interruption after archive sync, before state replacement.
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert_eq!(repo.status().unwrap().high_water_sequence, 0);
    install_confirmed(&mut repo, 1);
}

#[test]
fn corrupted_selected_archive_recovers_prior_and_preserves_accepted_sequence() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    install_confirmed(&mut repo, 1);
    install_confirmed(&mut repo, 2);
    let archive = repo.archive_path(2);
    drop(repo);
    fs::write(archive.join("signature.ed25519"), [0; 64]).unwrap();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert_eq!(
        repo.status().unwrap().recovery,
        RepositoryRecovery::InvalidSelectedPack {
            rejected_sequence: 2,
            restored_sequence: Some(1)
        }
    );
    assert_eq!(repo.selected_pack().unwrap().unwrap().sequence(), 1);
    assert_eq!(repo.status().unwrap().high_water_sequence, 2);
    assert!(matches!(
        repo.begin_trial(candidate(2, MANIFEST)),
        Err(StoreError::Verification(UpdateError::SequenceNotNewer))
    ));
}

#[test]
fn corrupted_or_missing_state_cannot_reset_sequence() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    install_confirmed(&mut repo, 1);
    drop(repo);
    fs::write(root.0.join("state.json"), b"{truncated").unwrap();
    assert!(matches!(
        RulePackRepository::open(&root.0, &keys),
        Err(StoreError::CorruptState(_))
    ));
    fs::remove_file(root.0.join("state.json")).unwrap();
    assert!(matches!(
        RulePackRepository::open(&root.0, &keys),
        Err(StoreError::MissingState)
    ));
}

#[test]
fn lock_is_exclusive_and_symlinked_root_or_state_is_rejected() {
    let root = PrivateRoot::new();
    let keys = trust();
    let repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert!(matches!(
        RulePackRepository::open(&root.0, &keys),
        Err(StoreError::Busy)
    ));
    drop(repo);
    let sibling = PrivateRoot::new();
    let link = sibling.0.join("link");
    std::os::unix::fs::symlink(&root.0, &link).unwrap();
    assert!(matches!(
        RulePackRepository::open(&link, &keys),
        Err(StoreError::UnsafePath(_))
    ));
    fs::remove_file(root.0.join("state.json")).unwrap();
    std::os::unix::fs::symlink(sibling.0.join("outside.json"), root.0.join("state.json")).unwrap();
    assert!(matches!(
        RulePackRepository::open(&root.0, &keys),
        Err(StoreError::UnsafePath(_))
    ));
    assert!(!sibling.0.join("outside.json").exists());
}

#[test]
fn higher_sequence_cannot_downgrade_highest_accepted_iana_release() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    // Synthetic version metadata for state-policy testing, not an IANA 2027 pack.
    let future = std::str::from_utf8(MANIFEST)
        .unwrap()
        .replace("\"data_version\": \"2026d\"", "\"data_version\": \"2027a\"")
        .replace("\"release_year\": 2026", "\"release_year\": 2027");
    repo.begin_trial(candidate(1, future.as_bytes())).unwrap();
    repo.confirm_trial(1).unwrap();
    repo.rollback().unwrap();
    assert!(repo.selected_pack().unwrap().is_none());
    assert!(matches!(
        repo.begin_trial(candidate(2, MANIFEST)),
        Err(StoreError::ReleaseDowngrade)
    ));
    assert_eq!(repo.status().unwrap().high_water_sequence, 1);
}

#[test]
fn bad_signature_cannot_create_archive_or_advance_state() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    let mut invalid = candidate(1, MANIFEST);
    invalid.signature = [0; 64];
    assert!(matches!(
        repo.begin_trial(invalid),
        Err(StoreError::Verification(UpdateError::InvalidSignature))
    ));
    assert_eq!(repo.status().unwrap().high_water_sequence, 0);
    assert_eq!(fs::read_dir(root.0.join("packs")).unwrap().count(), 0);
}

#[test]
fn temporary_read_failures_do_not_demote_a_valid_selection_or_recovery_pack() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    install_confirmed(&mut repo, 1);
    install_confirmed(&mut repo, 2);
    let original_state = fs::read(root.0.join("state.json")).unwrap();
    repo.read_error = Some((2, io::ErrorKind::WouldBlock));
    assert!(
        matches!(repo.recover_selection(), Err(StoreError::Io(e)) if e.kind() == io::ErrorKind::WouldBlock)
    );
    assert_eq!(fs::read(root.0.join("state.json")).unwrap(), original_state);
    assert_eq!(repo.status().unwrap().selected.unwrap().sequence, 2);
    repo.read_error = Some((1, io::ErrorKind::Interrupted));
    assert!(
        matches!(repo.rollback(), Err(StoreError::Io(e)) if e.kind() == io::ErrorKind::Interrupted)
    );
    assert_eq!(fs::read(root.0.join("state.json")).unwrap(), original_state);
    repo.read_error = None;
    assert_eq!(repo.selected_pack().unwrap().unwrap().sequence(), 2);
    repo.rollback().unwrap();
    assert_eq!(repo.selected_pack().unwrap().unwrap().sequence(), 1);
}

#[test]
fn valid_json_state_corruption_is_detected_by_checksum() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    install_confirmed(&mut repo, 1);
    drop(repo);
    let path = root.0.join("state.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["state"]["high_water"] = serde_json::json!(2);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(matches!(
        RulePackRepository::open(&root.0, &keys),
        Err(StoreError::CorruptState(reason)) if reason.contains("checksum")
    ));
}

#[test]
fn a_failed_state_replacement_poison_session_until_reopen() {
    let root = PrivateRoot::new();
    let keys = trust();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    let state = root.0.join("state.json");
    let backup = root.0.join("state-backup.json");
    fs::rename(&state, &backup).unwrap();
    private_directory(&state).unwrap();
    // Force an actual rename failure after the candidate archive has synced.
    assert!(matches!(
        repo.begin_trial(candidate(1, MANIFEST)),
        Err(StoreError::CommitUncertain(_))
    ));
    assert!(matches!(repo.status(), Err(StoreError::ReopenRequired)));
    assert!(matches!(
        repo.selected_pack(),
        Err(StoreError::ReopenRequired)
    ));
    drop(repo);
    fs::remove_dir(&state).unwrap();
    fs::rename(&backup, &state).unwrap();
    let mut repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert_eq!(repo.status().unwrap().high_water_sequence, 0);
    install_confirmed(&mut repo, 1);
}

#[test]
fn abrupt_process_exit_releases_lock_and_recovers_trial() {
    let root = PrivateRoot::new();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "repository::tests::crash_trial_worker",
        ])
        .env("SALAH_TEST_CRASH_ROOT", &root.0)
        .stdout(std::process::Stdio::null())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(17));
    let keys = trust();
    let repo = RulePackRepository::open(&root.0, &keys).unwrap();
    assert_eq!(repo.status().unwrap().high_water_sequence, 1);
    assert!(repo.selected_pack().unwrap().is_none());
    assert_eq!(
        repo.status().unwrap().recovery,
        RepositoryRecovery::InterruptedTrial {
            rejected_sequence: 1,
            restored_sequence: None
        }
    );
}

#[test]
#[ignore = "subprocess fixture invoked by abrupt_process_exit_releases_lock_and_recovers_trial"]
fn crash_trial_worker() {
    let root =
        PathBuf::from(std::env::var_os("SALAH_TEST_CRASH_ROOT").expect("isolated test root"));
    let keys = trust();
    let mut repo = RulePackRepository::open(&root, &keys).unwrap();
    repo.begin_trial(candidate(1, MANIFEST)).unwrap();
    // Skip all destructors, including the repository's advisory-lock handle.
    std::process::exit(17);
}
