//! Offline verification of a signed civil-time rule-pack candidate.
//!
//! This crate never fetches, installs, or activates data. The caller must
//! compile trusted public keys into its application; a key delivered with the
//! candidate is not a trust root. The only accepted boundary artifact in this
//! revision is Salah's currently bundled one.

use std::collections::BTreeMap;
use std::fmt;

use ed25519_dalek::{Signature, VerifyingKey};

use salah_location::BOUNDARY_DATA_SHA256;
pub use salah_time::{MAX_LICENSE_BYTES, MAX_MANIFEST_BYTES, MAX_PACK_BYTES};
use salah_time::{RulePackError, RulePackPayload, ValidatedRulePack};
#[cfg(test)]
use salah_time::{TZDB_PACK_INVENTORY_SHA256, TZDB_PACK_SHA256, TZDB_VERSION};

const SIGNING_DOMAIN: &[u8] = b"SALAH-TZIF-UPDATE-V1\0";
const MAX_KEY_ID_BYTES: usize = 64;

/// Public key installed by a trusted application release, never read from an
/// update candidate. Private signing keys do not belong in this repository.
#[derive(Debug, Clone)]
pub struct TrustedPublicKey {
    pub key_id: String,
    pub bytes: [u8; 32],
}

/// A nonempty set of trusted public keys supplied outside the update package.
pub struct TrustStore {
    keys: BTreeMap<String, VerifyingKey>,
}

impl TrustStore {
    /// Construct from keys pinned by the application build. Duplicate IDs or
    /// malformed public keys fail closed.
    pub fn new(keys: impl IntoIterator<Item = TrustedPublicKey>) -> Result<Self, UpdateError> {
        let mut trusted = BTreeMap::new();
        for key in keys {
            if !valid_key_id(&key.key_id) {
                return Err(UpdateError::InvalidKeyId);
            }
            let verifying_key =
                VerifyingKey::from_bytes(&key.bytes).map_err(|_| UpdateError::InvalidPublicKey)?;
            if trusted.insert(key.key_id, verifying_key).is_some() {
                return Err(UpdateError::DuplicateKeyId);
            }
        }
        if trusted.is_empty() {
            return Err(UpdateError::NoTrustedKeys);
        }
        Ok(Self { keys: trusted })
    }

    /// Verify signature first, then inspect manifest, hashes, inventory, and
    /// TZif images. `last_accepted_sequence` must come from trusted durable
    /// state; this verifier cannot protect against rollback if the caller
    /// forgets that high-water mark after restart.
    pub fn verify_candidate(
        &self,
        candidate: SignedPackCandidate<'_>,
        last_accepted_sequence: u64,
    ) -> Result<VerifiedRulePack, UpdateError> {
        check_candidate_bounds(&candidate)?;
        if candidate.sequence <= last_accepted_sequence {
            return Err(UpdateError::SequenceNotNewer);
        }
        if candidate.boundary_sha256 != BOUNDARY_DATA_SHA256 {
            return Err(UpdateError::IncompatibleBoundary);
        }
        let key = self
            .keys
            .get(candidate.key_id)
            .ok_or(UpdateError::UnknownKeyId)?;
        let message = signing_message(&candidate);
        let signature = Signature::from_bytes(&candidate.signature);
        key.verify_strict(&message, &signature)
            .map_err(|_| UpdateError::InvalidSignature)?;

        let payload = ValidatedRulePack::from_payload(RulePackPayload {
            manifest_bytes: candidate.manifest_bytes,
            pack_bytes: candidate.pack_bytes,
            license_bytes: candidate.license_bytes,
        })
        .map_err(|error| match error {
            RulePackError::PayloadTooLarge => UpdateError::CandidateTooLarge,
            RulePackError::InvalidManifest(reason) => UpdateError::InvalidManifest(reason),
            RulePackError::InvalidPayload(reason) => UpdateError::InvalidPayload(reason),
        })?;
        let identity = payload.identity();
        let identity = VerifiedPackIdentity {
            schema_version: identity.schema_version,
            tzdb_version: identity.tzdb_version.to_string(),
            release_year: identity.release_year,
            sha256: identity.sha256.to_string(),
            inventory_sha256: identity.inventory_sha256.to_string(),
        };
        Ok(VerifiedRulePack {
            sequence: candidate.sequence,
            key_id: candidate.key_id.to_owned(),
            boundary_sha256: candidate.boundary_sha256.to_owned(),
            identity,
            payload,
            manifest_bytes: candidate.manifest_bytes.to_vec(),
            license_bytes: candidate.license_bytes.to_vec(),
            signature: candidate.signature,
        })
    }
}

/// Detached signature and raw payload bytes. All fields except `signature`
/// are covered by the domain-separated signature message.
pub struct SignedPackCandidate<'a> {
    pub key_id: &'a str,
    pub sequence: u64,
    pub boundary_sha256: &'a str,
    pub manifest_bytes: &'a [u8],
    pub pack_bytes: &'a [u8],
    pub license_bytes: &'a [u8],
    pub signature: [u8; 64],
}

/// Exact identity of a signature-verified, internally consistent rule pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPackIdentity {
    pub schema_version: u32,
    pub tzdb_version: String,
    pub release_year: i32,
    pub sha256: String,
    pub inventory_sha256: String,
}

/// An authenticated and validated candidate. Private fields prevent callers
/// from constructing one without passing the verifier. The engine can consume
/// it as an explicit immutable runtime snapshot. Verification alone does not
/// install a package or confirm a durable repository trial.
pub struct VerifiedRulePack {
    sequence: u64,
    key_id: String,
    boundary_sha256: String,
    identity: VerifiedPackIdentity,
    manifest_bytes: Vec<u8>,
    payload: ValidatedRulePack,
    license_bytes: Vec<u8>,
    signature: [u8; 64],
}

impl VerifiedRulePack {
    #[must_use]
    pub fn identity(&self) -> &VerifiedPackIdentity {
        &self.identity
    }

    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    #[must_use]
    pub fn boundary_sha256(&self) -> &str {
        &self.boundary_sha256
    }

    #[must_use]
    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }

    #[must_use]
    pub fn license_bytes(&self) -> &[u8] {
        &self.license_bytes
    }

    #[must_use]
    pub fn pack_bytes(&self) -> &[u8] {
        self.payload.pack_bytes()
    }

    #[must_use]
    pub fn signature(&self) -> &[u8; 64] {
        &self.signature
    }

    /// Immutable validated payload used by the authenticated runtime adapter.
    /// Access cannot change its identity, bytes, or inventory.
    #[must_use]
    pub fn runtime_payload(&self) -> &ValidatedRulePack {
        &self.payload
    }

    #[must_use]
    pub fn zone_bytes(&self, zone_id: &str) -> Option<&[u8]> {
        self.payload.zone_bytes(zone_id)
    }
}

/// Verification failures are distinct from prayer or civil-time calculation
/// errors; no failure falls back to installing a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateError {
    NoTrustedKeys,
    InvalidKeyId,
    DuplicateKeyId,
    InvalidPublicKey,
    UnknownKeyId,
    SequenceNotNewer,
    IncompatibleBoundary,
    CandidateTooLarge,
    InvalidSignature,
    InvalidManifest(String),
    InvalidPayload(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTrustedKeys => f.write_str("no trusted update keys are configured"),
            Self::InvalidKeyId => f.write_str("invalid trusted or candidate key ID"),
            Self::DuplicateKeyId => f.write_str("duplicate trusted key ID"),
            Self::InvalidPublicKey => f.write_str("invalid trusted Ed25519 public key"),
            Self::UnknownKeyId => f.write_str("candidate key ID is not trusted"),
            Self::SequenceNotNewer => f.write_str("candidate sequence is not newer"),
            Self::IncompatibleBoundary => {
                f.write_str("candidate does not target the bundled boundary artifact")
            }
            Self::CandidateTooLarge => f.write_str("candidate exceeds update format limits"),
            Self::InvalidSignature => f.write_str("candidate Ed25519 signature is invalid"),
            Self::InvalidManifest(reason) => write!(f, "invalid update manifest: {reason}"),
            Self::InvalidPayload(reason) => write!(f, "invalid update payload: {reason}"),
        }
    }
}

impl std::error::Error for UpdateError {}

fn valid_key_id(key_id: &str) -> bool {
    !key_id.is_empty()
        && key_id.len() <= MAX_KEY_ID_BYTES
        && key_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn check_candidate_bounds(candidate: &SignedPackCandidate<'_>) -> Result<(), UpdateError> {
    if !valid_key_id(candidate.key_id) {
        return Err(UpdateError::InvalidKeyId);
    }
    if candidate.manifest_bytes.is_empty()
        || candidate.manifest_bytes.len() > MAX_MANIFEST_BYTES
        || candidate.pack_bytes.is_empty()
        || candidate.pack_bytes.len() > MAX_PACK_BYTES
        || candidate.license_bytes.is_empty()
        || candidate.license_bytes.len() > MAX_LICENSE_BYTES
    {
        return Err(UpdateError::CandidateTooLarge);
    }
    if !valid_sha256(candidate.boundary_sha256) {
        return Err(UpdateError::IncompatibleBoundary);
    }
    Ok(())
}

/// Exact domain-separated bytes that a release signer must sign with Ed25519.
/// The `signature` field itself is excluded. This function checks the same
/// size and identifier bounds as verification, so publisher and verifier use
/// one encoding implementation.
pub fn candidate_signing_bytes(
    candidate: &SignedPackCandidate<'_>,
) -> Result<Vec<u8>, UpdateError> {
    check_candidate_bounds(candidate)?;
    Ok(signing_message(candidate))
}

fn signing_message(candidate: &SignedPackCandidate<'_>) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        SIGNING_DOMAIN.len()
            + candidate.key_id.len()
            + candidate.boundary_sha256.len()
            + candidate.manifest_bytes.len()
            + candidate.pack_bytes.len()
            + candidate.license_bytes.len()
            + 6 * 8,
    );
    message.extend_from_slice(SIGNING_DOMAIN);
    message.extend_from_slice(&candidate.sequence.to_be_bytes());
    for field in [
        candidate.key_id.as_bytes(),
        candidate.boundary_sha256.as_bytes(),
        candidate.manifest_bytes,
        candidate.pack_bytes,
        candidate.license_bytes,
    ] {
        message.extend_from_slice(&(field.len() as u64).to_be_bytes());
        message.extend_from_slice(field);
    }
    message
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const MANIFEST: &[u8] = include_bytes!("../../salah-time/fixtures/global/manifest.json");
    const PACK: &[u8] = include_bytes!("../../salah-time/fixtures/global/tzif-pack.bin");
    const LICENSE: &[u8] = include_bytes!("../../salah-time/fixtures/global/IANA-LICENSE");
    const TEST_KEY_ID: &str = "test-only-key";
    const OTHER_BOUNDARY_HASH: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";

    fn test_key() -> SigningKey {
        // Deterministic fixture key. Never configure it as a production trust root.
        SigningKey::from_bytes(&[7; 32])
    }

    fn trust_store() -> TrustStore {
        TrustStore::new([TrustedPublicKey {
            key_id: TEST_KEY_ID.to_owned(),
            bytes: test_key().verifying_key().to_bytes(),
        }])
        .unwrap()
    }

    fn signed_candidate<'a>(
        manifest_bytes: &'a [u8],
        pack_bytes: &'a [u8],
        boundary_sha256: &'a str,
        sequence: u64,
    ) -> SignedPackCandidate<'a> {
        let mut candidate = SignedPackCandidate {
            key_id: TEST_KEY_ID,
            sequence,
            boundary_sha256,
            manifest_bytes,
            pack_bytes,
            license_bytes: LICENSE,
            signature: [0; 64],
        };
        candidate.signature = test_key()
            .sign(&candidate_signing_bytes(&candidate).unwrap())
            .to_bytes();
        candidate
    }

    #[test]
    fn signed_bundled_snapshot_verifies_as_a_candidate() {
        let verified = trust_store()
            .verify_candidate(signed_candidate(MANIFEST, PACK, BOUNDARY_DATA_SHA256, 1), 0)
            .unwrap();
        assert_eq!(verified.identity().tzdb_version, TZDB_VERSION);
        assert_eq!(verified.identity().sha256, TZDB_PACK_SHA256);
        assert_eq!(
            verified.identity().inventory_sha256,
            TZDB_PACK_INVENTORY_SHA256
        );
        assert_eq!(verified.sequence(), 1);
        assert_eq!(verified.key_id(), TEST_KEY_ID);
        assert_eq!(verified.boundary_sha256(), BOUNDARY_DATA_SHA256);
        assert_eq!(
            verified.zone_bytes("America/Chicago").unwrap(),
            salah_time::fixture_tzif_bytes("America/Chicago").unwrap()
        );
        assert!(verified.zone_bytes("Unknown/Zone").is_none());
    }

    #[test]
    fn tampered_payload_and_unknown_key_fail_before_parsing() {
        let mut candidate = signed_candidate(MANIFEST, PACK, BOUNDARY_DATA_SHA256, 1);
        let mut changed = PACK.to_vec();
        changed[0] ^= 1;
        candidate.pack_bytes = &changed;
        assert!(matches!(
            trust_store().verify_candidate(candidate, 0),
            Err(UpdateError::InvalidSignature)
        ));

        let mut unknown = signed_candidate(MANIFEST, PACK, BOUNDARY_DATA_SHA256, 1);
        unknown.key_id = "untrusted-key";
        assert!(matches!(
            trust_store().verify_candidate(unknown, 0),
            Err(UpdateError::UnknownKeyId)
        ));
    }

    #[test]
    fn sequence_and_boundary_are_fail_closed() {
        assert!(matches!(
            trust_store()
                .verify_candidate(signed_candidate(MANIFEST, PACK, BOUNDARY_DATA_SHA256, 1), 1),
            Err(UpdateError::SequenceNotNewer)
        ));
        assert!(matches!(
            trust_store()
                .verify_candidate(signed_candidate(MANIFEST, PACK, OTHER_BOUNDARY_HASH, 2), 1),
            Err(UpdateError::IncompatibleBoundary)
        ));
        assert!(matches!(
            TrustStore::new(Vec::new()),
            Err(UpdateError::NoTrustedKeys)
        ));
    }

    #[test]
    fn a_valid_signature_does_not_bypass_pack_hash_validation() {
        let mut changed = PACK.to_vec();
        changed[0] ^= 1;
        let candidate = signed_candidate(MANIFEST, &changed, BOUNDARY_DATA_SHA256, 1);
        assert!(matches!(
            trust_store().verify_candidate(candidate, 0),
            Err(UpdateError::InvalidPayload(reason)) if reason.contains("pack SHA-256")
        ));
    }

    #[test]
    fn signed_invalid_schema_and_duplicate_fields_are_rejected() {
        let text = std::str::from_utf8(MANIFEST).unwrap();
        let changed = text.replace("\"schema_version\": 1", "\"schema_version\": 2");
        assert!(matches!(
            trust_store().verify_candidate(
                signed_candidate(changed.as_bytes(), PACK, BOUNDARY_DATA_SHA256, 1),
                0
            ),
            Err(UpdateError::InvalidPayload(_))
        ));
        let duplicate = text.replace(
            "\"schema_version\": 1,",
            "\"schema_version\": 1, \"schema_version\": 1,",
        );
        assert!(matches!(
            trust_store().verify_candidate(
                signed_candidate(duplicate.as_bytes(), PACK, BOUNDARY_DATA_SHA256, 1),
                0
            ),
            Err(UpdateError::InvalidManifest(_))
        ));
    }

    #[test]
    fn signed_non_ascii_release_is_rejected_without_panicking() {
        let text = std::str::from_utf8(MANIFEST).unwrap();
        let changed = text.replace("\"data_version\": \"2026d\"", "\"data_version\": \"ééa\"");
        assert!(matches!(
            trust_store().verify_candidate(
                signed_candidate(changed.as_bytes(), PACK, BOUNDARY_DATA_SHA256, 1),
                0
            ),
            Err(UpdateError::InvalidPayload(_))
        ));
    }

    #[test]
    fn signed_zone_inventory_mismatch_is_rejected() {
        let text = std::str::from_utf8(MANIFEST).unwrap();
        let changed = text
            .replace("\"data_version\": \"2026d\"", "\"data_version\": \"2027a\"")
            .replace("\"release_year\": 2026", "\"release_year\": 2027")
            .replace(TZDB_PACK_INVENTORY_SHA256, &"0".repeat(64));
        assert!(matches!(
            trust_store().verify_candidate(
                signed_candidate(changed.as_bytes(), PACK, BOUNDARY_DATA_SHA256, 1),
                0
            ),
            Err(UpdateError::InvalidPayload(reason)) if reason.contains("zone inventory SHA-256")
        ));
    }
}
