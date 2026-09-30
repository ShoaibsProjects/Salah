use std::fs::{self, DirBuilder, File, OpenOptions, TryLockError};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use salah_update::{
    MAX_LICENSE_BYTES, MAX_MANIFEST_BYTES, MAX_PACK_BYTES, SignedPackCandidate, TrustStore,
    UpdateError, VerifiedPackIdentity, VerifiedRulePack,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STATE_LIMIT: usize = 16 * 1024;
const METADATA_LIMIT: usize = 4096;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Verification(UpdateError),
    Busy,
    UnsafePath(PathBuf),
    MissingState,
    CorruptState(String),
    OccupiedSequence(u64),
    ReleaseDowngrade,
    ReleaseConflict,
    TrialInProgress,
    NoMatchingTrial,
    CommitUncertain(io::Error),
    ReopenRequired,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "rule-pack storage failed: {e}"),
            Self::Verification(e) => write!(f, "stored candidate verification failed: {e}"),
            Self::Busy => f.write_str("rule-pack repository is already open"),
            Self::UnsafePath(p) => write!(
                f,
                "repository path must be private and nonsymlinked: {}",
                p.display()
            ),
            Self::MissingState => f.write_str(
                "repository state is missing beside existing archives; sequence cannot be reset",
            ),
            Self::CorruptState(reason) => write!(f, "invalid repository state: {reason}"),
            Self::OccupiedSequence(n) => {
                write!(f, "archive sequence {n} already has different contents")
            }
            Self::ReleaseDowngrade => {
                f.write_str("candidate IANA release predates the highest accepted release")
            }
            Self::ReleaseConflict => {
                f.write_str("candidate changes an already accepted IANA release identity")
            }
            Self::TrialInProgress => f.write_str("confirm or roll back the current trial first"),
            Self::NoMatchingTrial => f.write_str("no trial matches the supplied sequence"),
            Self::CommitUncertain(e) => write!(
                f,
                "state commit may have reached disk; close and reopen: {e}"
            ),
            Self::ReopenRequired => f.write_str("close and reopen after an uncertain state commit"),
        }
    }
}

impl std::error::Error for StoreError {}
impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<UpdateError> for StoreError {
    fn from(value: UpdateError) -> Self {
        Self::Verification(value)
    }
}

/// Identity and sequence of a repository selection; this is not a calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPackStatus {
    pub sequence: u64,
    pub identity: VerifiedPackIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryRecovery {
    None,
    InterruptedTrial {
        rejected_sequence: u64,
        restored_sequence: Option<u64>,
    },
    InvalidSelectedPack {
        rejected_sequence: u64,
        restored_sequence: Option<u64>,
    },
}

/// A `None` selection means use the application's compiled recovery pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub high_water_sequence: u64,
    pub selected: Option<StoredPackStatus>,
    pub awaiting_confirmation: bool,
    pub recovery: RepositoryRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PackRef {
    sequence: u64,
    key_id: String,
    boundary_sha256: String,
    schema_version: u32,
    tzdb_version: String,
    release_year: i32,
    sha256: String,
    inventory_sha256: String,
}

impl PackRef {
    fn from_pack(pack: &VerifiedRulePack) -> Self {
        let id = pack.identity();
        Self {
            sequence: pack.sequence(),
            key_id: pack.key_id().to_owned(),
            boundary_sha256: pack.boundary_sha256().to_owned(),
            schema_version: id.schema_version,
            tzdb_version: id.tzdb_version.clone(),
            release_year: id.release_year,
            sha256: id.sha256.clone(),
            inventory_sha256: id.inventory_sha256.clone(),
        }
    }

    fn status(&self) -> StoredPackStatus {
        StoredPackStatus {
            sequence: self.sequence,
            identity: VerifiedPackIdentity {
                schema_version: self.schema_version,
                tzdb_version: self.tzdb_version.clone(),
                release_year: self.release_year,
                sha256: self.sha256.clone(),
                inventory_sha256: self.inventory_sha256.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    revision: u64,
    high_water: u64,
    highest: Option<PackRef>,
    selected: Option<PackRef>,
    previous: Option<PackRef>,
    trial: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StateEnvelope {
    schema: u32,
    state: State,
    checksum: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageMetadata {
    schema: u32,
    sequence: u64,
    key_id: String,
    boundary_sha256: String,
}

/// Holds an OS advisory lock until dropped. Mutations require `&mut self`.
pub struct RulePackRepository<'a> {
    root: PathBuf,
    _lock: File,
    trust: &'a TrustStore,
    state: State,
    recovery: RepositoryRecovery,
    poisoned: bool,
    #[cfg(test)]
    read_error: Option<(u64, io::ErrorKind)>,
}

impl<'a> RulePackRepository<'a> {
    /// Open a caller-selected private local directory. The parent must exist.
    /// An interrupted trial is reverted before this call returns; accepted
    /// sequence/release high-water records survive that recovery.
    pub fn open(root: &Path, trust: &'a TrustStore) -> Result<Self, StoreError> {
        if private_directory(root)? {
            sync_directory(
                root.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new(".")),
            )?;
        }
        let root = root.canonicalize()?;
        let lock_path = root.join("repository.lock");
        check_existing_file(&lock_path)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&lock_path)?;
        check_file(&lock_path)?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Err(StoreError::Busy),
            Err(TryLockError::Error(e)) => return Err(StoreError::Io(e)),
        }
        private_directory(&root.join("packs"))?;
        sync_directory(&root)?;
        let state_path = root.join("state.json");
        let state = match fs::symlink_metadata(&state_path) {
            Ok(_) => read_state(&state_path)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if fs::read_dir(root.join("packs"))?.next().is_some() {
                    return Err(StoreError::MissingState);
                }
                let state = State::default();
                persist_state(&root, &state).map_err(StoreError::CommitUncertain)?;
                state
            }
            Err(e) => return Err(e.into()),
        };
        validate_state(&state)?;
        let mut repo = Self {
            root,
            _lock: lock,
            trust,
            state,
            recovery: RepositoryRecovery::None,
            poisoned: false,
            #[cfg(test)]
            read_error: None,
        };
        repo.recover_selection()?;
        Ok(repo)
    }

    pub fn status(&self) -> Result<RepositoryStatus, StoreError> {
        self.ensure_usable()?;
        Ok(RepositoryStatus {
            high_water_sequence: self.state.high_water,
            selected: self.state.selected.as_ref().map(PackRef::status),
            awaiting_confirmation: self.state.trial,
            recovery: self.recovery.clone(),
        })
    }

    /// Reverify archived bytes with the current trusted keys on every load.
    /// `None` directs the caller to its compiled recovery pack.
    pub fn selected_pack(&self) -> Result<Option<VerifiedRulePack>, StoreError> {
        self.ensure_usable()?;
        self.state
            .selected
            .as_ref()
            .map(|record| self.load_ref(record))
            .transpose()
    }

    /// Authenticate before any package/state write, archive exact signed bytes,
    /// then select an unconfirmed trial. Old selection remains available for
    /// recovery. A successful return advances the durable sequence high-water.
    pub fn begin_trial(
        &mut self,
        candidate: SignedPackCandidate<'_>,
    ) -> Result<StoredPackStatus, StoreError> {
        self.ensure_usable()?;
        if self.state.trial {
            return Err(StoreError::TrialInProgress);
        }
        let pack = self
            .trust
            .verify_candidate(candidate, self.state.high_water)?;
        let record = PackRef::from_pack(&pack);
        if let Some(highest) = &self.state.highest {
            if record.tzdb_version < highest.tzdb_version {
                return Err(StoreError::ReleaseDowngrade);
            }
            if record.tzdb_version == highest.tzdb_version
                && (record.sha256 != highest.sha256
                    || record.inventory_sha256 != highest.inventory_sha256)
            {
                return Err(StoreError::ReleaseConflict);
            }
        }
        self.persist_archive(&pack)?;
        let mut state = self.state.clone();
        state.previous = state.selected.take();
        state.selected = Some(record.clone());
        state.highest = Some(record.clone());
        state.high_water = record.sequence;
        state.trial = true;
        self.commit(state)?;
        Ok(record.status())
    }

    /// Call only after the consuming runtime's health check. The supplied
    /// sequence prevents confirming a different trial by accident.
    pub fn confirm_trial(&mut self, sequence: u64) -> Result<(), StoreError> {
        self.ensure_usable()?;
        if !self.state.trial || self.state.selected.as_ref().map(|p| p.sequence) != Some(sequence) {
            return Err(StoreError::NoMatchingTrial);
        }
        self.selected_pack()?;
        let mut state = self.state.clone();
        state.trial = false;
        self.commit(state)
    }

    /// Restore a valid prior archive, or request the compiled recovery pack.
    /// This never reduces the sequence or release high-water records.
    pub fn rollback(&mut self) -> Result<RepositoryStatus, StoreError> {
        self.ensure_usable()?;
        let mut state = self.state.clone();
        state.selected = self.valid_recovery_ref(state.previous.take())?;
        state.trial = false;
        self.commit(state)?;
        self.status()
    }

    fn ensure_usable(&self) -> Result<(), StoreError> {
        if self.poisoned {
            Err(StoreError::ReopenRequired)
        } else {
            Ok(())
        }
    }

    fn commit(&mut self, mut state: State) -> Result<(), StoreError> {
        state.revision = self
            .state
            .revision
            .checked_add(1)
            .ok_or_else(|| StoreError::CorruptState("revision overflow".to_owned()))?;
        validate_state(&state)?;
        self.poisoned = true;
        persist_state(&self.root, &state).map_err(StoreError::CommitUncertain)?;
        self.state = state;
        self.poisoned = false;
        Ok(())
    }

    fn recover_selection(&mut self) -> Result<(), StoreError> {
        let Some(selected) = self.state.selected.clone() else {
            return Ok(());
        };
        if !self.state.trial {
            match self.load_ref(&selected) {
                Ok(_) => return Ok(()),
                Err(e) if invalid_archive(&e) => {}
                Err(e) => return Err(e),
            }
        }
        let interrupted_trial = self.state.trial;
        let mut state = self.state.clone();
        state.selected = self.valid_recovery_ref(state.previous.take())?;
        state.trial = false;
        let restored_sequence = state.selected.as_ref().map(|p| p.sequence);
        self.commit(state)?;
        self.recovery = if interrupted_trial {
            RepositoryRecovery::InterruptedTrial {
                rejected_sequence: selected.sequence,
                restored_sequence,
            }
        } else {
            RepositoryRecovery::InvalidSelectedPack {
                rejected_sequence: selected.sequence,
                restored_sequence,
            }
        };
        Ok(())
    }

    fn load_ref(&self, record: &PackRef) -> Result<VerifiedRulePack, StoreError> {
        #[cfg(test)]
        if let Some((sequence, kind)) = self.read_error
            && record.sequence == sequence
        {
            return Err(StoreError::Io(io::Error::new(
                kind,
                "injected archive read failure",
            )));
        }
        let pack = self.read_archive(record.sequence)?;
        if PackRef::from_pack(&pack) != *record {
            return Err(StoreError::CorruptState(
                "archive differs from recorded identity".to_owned(),
            ));
        }
        Ok(pack)
    }

    fn valid_recovery_ref(&self, record: Option<PackRef>) -> Result<Option<PackRef>, StoreError> {
        let Some(record) = record else {
            return Ok(None);
        };
        match self.load_ref(&record) {
            Ok(_) => Ok(Some(record)),
            Err(e) if invalid_archive(&e) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn archive_path(&self, sequence: u64) -> PathBuf {
        self.root.join("packs").join(format!("{sequence:020}"))
    }

    fn read_archive(&self, sequence: u64) -> Result<VerifiedRulePack, StoreError> {
        let dir = self.archive_path(sequence);
        check_directory(&dir)?;
        let metadata: PackageMetadata =
            serde_json::from_slice(&read_bounded(&dir.join("signed.json"), METADATA_LIMIT)?)
                .map_err(|e| StoreError::CorruptState(e.to_string()))?;
        if metadata.schema != 1 || metadata.sequence != sequence {
            return Err(StoreError::CorruptState(
                "archive metadata mismatch".to_owned(),
            ));
        }
        let manifest = read_bounded(&dir.join("manifest.json"), MAX_MANIFEST_BYTES)?;
        let pack = read_bounded(&dir.join("tzif-pack.bin"), MAX_PACK_BYTES)?;
        let license = read_bounded(&dir.join("IANA-LICENSE"), MAX_LICENSE_BYTES)?;
        let signature: [u8; 64] = read_bounded(&dir.join("signature.ed25519"), 64)?
            .try_into()
            .map_err(|_| StoreError::CorruptState("signature must contain 64 bytes".to_owned()))?;
        Ok(self.trust.verify_candidate(
            SignedPackCandidate {
                key_id: &metadata.key_id,
                sequence,
                boundary_sha256: &metadata.boundary_sha256,
                manifest_bytes: &manifest,
                pack_bytes: &pack,
                license_bytes: &license,
                signature,
            },
            0,
        )?)
    }

    fn persist_archive(&self, pack: &VerifiedRulePack) -> Result<(), StoreError> {
        let packs = self.root.join("packs");
        check_directory(&packs)?;
        let destination = self.archive_path(pack.sequence());
        let destination_exists = match fs::symlink_metadata(&destination) {
            Ok(_) => true,
            Err(e) if e.kind() == io::ErrorKind::NotFound => false,
            Err(e) => return Err(e.into()),
        };
        if destination_exists {
            let old = self.read_archive(pack.sequence())?;
            if PackRef::from_pack(&old) == PackRef::from_pack(pack)
                && old.manifest_bytes() == pack.manifest_bytes()
                && old.pack_bytes() == pack.pack_bytes()
                && old.license_bytes() == pack.license_bytes()
                && old.signature() == pack.signature()
            {
                // An earlier attempt may have renamed this orphan archive
                // and then failed syncing its parent. Establish durability
                // before a new state record can reference the reused slot.
                sync_directory(&destination)?;
                sync_directory(&packs)?;
                return Ok(());
            }
            return Err(StoreError::OccupiedSequence(pack.sequence()));
        }
        let stage = new_stage_directory(&packs)?;
        let metadata = PackageMetadata {
            schema: 1,
            sequence: pack.sequence(),
            key_id: pack.key_id().to_owned(),
            boundary_sha256: pack.boundary_sha256().to_owned(),
        };
        let metadata_bytes =
            serde_json::to_vec(&metadata).map_err(|e| StoreError::CorruptState(e.to_string()))?;
        let result = (|| -> Result<(), StoreError> {
            for (name, bytes) in [
                ("signed.json", metadata_bytes.as_slice()),
                ("manifest.json", pack.manifest_bytes()),
                ("tzif-pack.bin", pack.pack_bytes()),
                ("IANA-LICENSE", pack.license_bytes()),
                ("signature.ed25519", pack.signature().as_slice()),
            ] {
                write_new_synced(&stage.join(name), bytes)?;
            }
            sync_directory(&stage)?;
            fs::rename(&stage, &destination)?;
            sync_directory(&packs)?;
            Ok(())
        })();
        // Only this process's unpublished temporary directory is removed.
        let _ = fs::remove_dir_all(&stage);
        result
    }
}

fn invalid_archive(error: &StoreError) -> bool {
    match error {
        StoreError::Verification(_) | StoreError::CorruptState(_) | StoreError::UnsafePath(_) => {
            true
        }
        StoreError::Io(e) => e.kind() == io::ErrorKind::NotFound,
        _ => false,
    }
}

fn validate_state(state: &State) -> Result<(), StoreError> {
    let invalid = || StoreError::CorruptState("sequence/selection invariant failed".to_owned());
    if state.high_water == 0 {
        if state.highest.is_some()
            || state.selected.is_some()
            || state.previous.is_some()
            || state.trial
        {
            return Err(invalid());
        }
        return Ok(());
    }
    let highest = state.highest.as_ref().ok_or_else(invalid)?;
    if highest.sequence != state.high_water {
        return Err(invalid());
    }
    for record in [&state.highest, &state.selected, &state.previous]
        .into_iter()
        .flatten()
    {
        let bytes = record.tzdb_version.as_bytes();
        if record.sequence == 0
            || record.sequence > state.high_water
            || record.schema_version != 1
            || bytes.len() != 5
            || !bytes[..4].iter().all(u8::is_ascii_digit)
            || !bytes[4].is_ascii_lowercase()
            || std::str::from_utf8(&bytes[..4])
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
                != Some(record.release_year)
            || [
                &record.sha256,
                &record.inventory_sha256,
                &record.boundary_sha256,
            ]
            .iter()
            .any(|v| !is_hash(v))
            || record.key_id.is_empty()
            || record.key_id.len() > 64
            || !record
                .key_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            || (record.sequence == highest.sequence && record != highest)
        {
            return Err(invalid());
        }
    }
    if state.selected.is_none() && (state.previous.is_some() || state.trial) {
        return Err(invalid());
    }
    if state.trial && state.selected.as_ref() != Some(highest) {
        return Err(invalid());
    }
    if let (Some(previous), Some(selected)) = (&state.previous, &state.selected)
        && previous.sequence >= selected.sequence
    {
        return Err(invalid());
    }
    Ok(())
}

fn is_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn checksum(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn read_state(path: &Path) -> Result<State, StoreError> {
    let envelope: StateEnvelope = serde_json::from_slice(&read_bounded(path, STATE_LIMIT)?)
        .map_err(|e| StoreError::CorruptState(e.to_string()))?;
    let bytes =
        serde_json::to_vec(&envelope.state).map_err(|e| StoreError::CorruptState(e.to_string()))?;
    if envelope.schema != 1 || envelope.checksum != checksum(&bytes) {
        return Err(StoreError::CorruptState(
            "state checksum/schema mismatch".to_owned(),
        ));
    }
    Ok(envelope.state)
}

fn persist_state(root: &Path, state: &State) -> io::Result<()> {
    let payload = serde_json::to_vec(state).map_err(io::Error::other)?;
    let bytes = serde_json::to_vec(&StateEnvelope {
        schema: 1,
        state: state.clone(),
        checksum: checksum(&payload),
    })
    .map_err(io::Error::other)?;
    if bytes.len() > STATE_LIMIT {
        return Err(io::Error::other("state exceeds format limit"));
    }
    let (path, mut file) = new_state_file(root)?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&path, root.join("state.json"))?;
        sync_directory(root)
    })();
    let _ = fs::remove_file(&path);
    result
}

fn private_directory(path: &Path) -> Result<bool, StoreError> {
    match DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {
            check_directory(path)?;
            Ok(true)
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            check_directory(path)?;
            Ok(false)
        }
        Err(e) => Err(e.into()),
    }
}
fn check_directory(path: &Path) -> Result<(), StoreError> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err(StoreError::UnsafePath(path.to_owned()));
    }
    Ok(())
}
fn check_existing_file(path: &Path) -> Result<(), StoreError> {
    match fs::symlink_metadata(path) {
        Ok(_) => check_file(path),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
fn check_file(path: &Path) -> Result<(), StoreError> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.permissions().mode() & 0o077 != 0 || meta.nlink() != 1 {
        return Err(StoreError::UnsafePath(path.to_owned()));
    }
    Ok(())
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, StoreError> {
    check_file(path)?;
    let file = File::open(path)?;
    if file.metadata()?.len() > limit as u64 {
        return Err(StoreError::CorruptState(
            "file exceeds format limit".to_owned(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(StoreError::CorruptState(
            "file exceeds format limit".to_owned(),
        ));
    }
    Ok(bytes)
}
fn write_new_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}
fn new_state_file(root: &Path) -> io::Result<(PathBuf, File)> {
    for _ in 0..100 {
        let path = root.join(format!(
            ".state-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("cannot reserve state file"))
}
fn new_stage_directory(root: &Path) -> Result<PathBuf, StoreError> {
    for _ in 0..100 {
        let path = root.join(format!(
            ".staging-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        match DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(StoreError::Io(io::Error::other(
        "cannot reserve staging directory",
    )))
}

#[cfg(test)]
mod tests;
