//! Offline verification of a signed civil-time rule-pack candidate.
//!
//! This crate never fetches, installs, or activates data. The caller must
//! compile trusted public keys into its application; a key delivered with the
//! candidate is not a trust root. The only accepted boundary artifact in this
//! revision is Salah's currently bundled one.

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;
use std::ops::Range;

use ed25519_dalek::{Signature, VerifyingKey};
use jiff::tz::TimeZone;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use salah_location::BOUNDARY_DATA_SHA256;
use salah_time::{SUPPORTED_ZONE_IDS, TZDB_PACK_INVENTORY_SHA256, TZDB_PACK_SHA256, TZDB_VERSION};

const SIGNING_DOMAIN: &[u8] = b"SALAH-TZIF-UPDATE-V1\0";
/// Maximum raw manifest length in the v1 signed-candidate format.
pub const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024;
/// Maximum raw TZif blob length in the v1 signed-candidate format.
pub const MAX_PACK_BYTES: usize = 32 * 1024 * 1024;
/// Maximum raw license length in the v1 signed-candidate format.
pub const MAX_LICENSE_BYTES: usize = 256 * 1024;
const MAX_ZONE_IDS: usize = 2048;
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

        let manifest: Manifest = serde_json::from_slice(candidate.manifest_bytes)
            .map_err(|error| UpdateError::InvalidManifest(error.to_string()))?;
        let zones = validate_payload(&manifest, &candidate)?;
        Ok(VerifiedRulePack {
            sequence: candidate.sequence,
            key_id: candidate.key_id.to_owned(),
            boundary_sha256: candidate.boundary_sha256.to_owned(),
            identity: VerifiedPackIdentity {
                schema_version: manifest.schema_version,
                tzdb_version: manifest.data_version,
                release_year: manifest.release_year,
                sha256: manifest.pack.sha256,
                inventory_sha256: manifest.pack.inventory_sha256,
            },
            manifest_bytes: candidate.manifest_bytes.to_vec(),
            pack_bytes: candidate.pack_bytes.to_vec(),
            license_bytes: candidate.license_bytes.to_vec(),
            signature: candidate.signature,
            zones,
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
/// from constructing one without passing the verifier. It is not active in
/// the prayer engine; activation needs a separate durable transaction.
pub struct VerifiedRulePack {
    sequence: u64,
    key_id: String,
    boundary_sha256: String,
    identity: VerifiedPackIdentity,
    manifest_bytes: Vec<u8>,
    pack_bytes: Vec<u8>,
    license_bytes: Vec<u8>,
    signature: [u8; 64],
    zones: BTreeMap<String, Range<usize>>,
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
        &self.pack_bytes
    }

    #[must_use]
    pub fn signature(&self) -> &[u8; 64] {
        &self.signature
    }

    #[must_use]
    pub fn zone_bytes(&self, zone_id: &str) -> Option<&[u8]> {
        self.zones
            .get(zone_id)
            .and_then(|range| self.pack_bytes.get(range.clone()))
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

fn sha256_hex(bytes: &[u8]) -> String {
    hex_lower(&Sha256::digest(bytes))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(64);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

fn invalid_payload(reason: impl Into<String>) -> UpdateError {
    UpdateError::InvalidPayload(reason.into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    data_version: String,
    release_year: i32,
    schema_version: u32,
    source_archive: SourceArchive,
    generator: Generator,
    license: License,
    pack: PackInfo,
    zones: Vec<ZoneEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceArchive {
    url: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Generator {
    command: String,
    compiler: String,
    compiler_command: String,
    host: String,
    posix_footer: String,
    tzcode_archive_sha256: String,
    tzcode_archive_url: String,
    zic_version: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct License {
    file: String,
    note: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackInfo {
    bytes: u64,
    file: String,
    sha256: String,
    inventory_sha256: String,
    unique_tzif_images: u32,
    zone_ids: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ZoneEntry {
    bytes: u64,
    id: String,
    offset: u64,
    sha256: String,
}

fn validate_payload(
    manifest: &Manifest,
    candidate: &SignedPackCandidate<'_>,
) -> Result<BTreeMap<String, Range<usize>>, UpdateError> {
    let version_bytes = manifest.data_version.as_bytes();
    if manifest.schema_version != 1
        || version_bytes.len() != 5
        || !version_bytes[..4].iter().all(u8::is_ascii_digit)
        || !version_bytes[4].is_ascii_lowercase()
        || std::str::from_utf8(&version_bytes[..4])
            .ok()
            .and_then(|year| year.parse::<i32>().ok())
            != Some(manifest.release_year)
    {
        return Err(invalid_payload(
            "unsupported schema or IANA release metadata",
        ));
    }
    if manifest.data_version.as_str() < TZDB_VERSION
        || (manifest.data_version == TZDB_VERSION
            && (manifest.pack.sha256 != TZDB_PACK_SHA256
                || manifest.pack.inventory_sha256 != TZDB_PACK_INVENTORY_SHA256))
    {
        return Err(invalid_payload(
            "candidate is older than or conflicts with bundled data",
        ));
    }
    if !valid_sha256(&manifest.source_archive.sha256)
        || !valid_sha256(&manifest.generator.tzcode_archive_sha256)
        || !manifest.source_archive.url.starts_with("https://")
        || !manifest
            .generator
            .tzcode_archive_url
            .starts_with("https://")
        || [
            &manifest.generator.command,
            &manifest.generator.compiler,
            &manifest.generator.compiler_command,
            &manifest.generator.host,
            &manifest.generator.posix_footer,
            &manifest.generator.zic_version,
            &manifest.license.note,
        ]
        .iter()
        .any(|value| value.is_empty())
    {
        return Err(invalid_payload("missing or invalid source provenance"));
    }
    if manifest.pack.file != "tzif-pack.bin"
        || manifest.license.file != "IANA-LICENSE"
        || !valid_sha256(&manifest.pack.sha256)
        || !valid_sha256(&manifest.pack.inventory_sha256)
        || !valid_sha256(&manifest.license.sha256)
        || manifest.pack.bytes != candidate.pack_bytes.len() as u64
        || manifest.pack.zone_ids as usize != manifest.zones.len()
        || manifest.zones.is_empty()
        || manifest.zones.len() > MAX_ZONE_IDS
    {
        return Err(invalid_payload("invalid pack inventory or file metadata"));
    }
    if sha256_hex(candidate.pack_bytes) != manifest.pack.sha256 {
        return Err(invalid_payload("pack SHA-256 mismatch"));
    }
    if sha256_hex(candidate.license_bytes) != manifest.license.sha256 {
        return Err(invalid_payload("license SHA-256 mismatch"));
    }

    let mut zones = BTreeMap::new();
    // IANA aliases often refer to one image. Cache the digest by exact slice
    // so verification cost scales with unique bytes, not aliases × bytes.
    let mut unique_images: BTreeMap<(usize, usize), [u8; 32]> = BTreeMap::new();
    let mut inventory_digest = Sha256::new();
    inventory_digest.update(b"SALAH-TZIF-ZONE-INVENTORY-V1\0");
    let mut previous_id = "";
    for zone in &manifest.zones {
        if zone.id.as_str() <= previous_id || !safe_zone_id(&zone.id) {
            return Err(invalid_payload(
                "zone IDs are unsafe, unsorted, or duplicated",
            ));
        }
        previous_id = &zone.id;
        let start = usize::try_from(zone.offset).map_err(|_| invalid_payload("zone offset"))?;
        let length = usize::try_from(zone.bytes).map_err(|_| invalid_payload("zone length"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| invalid_payload("zone range overflow"))?;
        let image = candidate
            .pack_bytes
            .get(start..end)
            .ok_or_else(|| invalid_payload("zone slice exceeds pack"))?;
        let image_range = (start, end);
        let is_new_image = !unique_images.contains_key(&image_range);
        let zone_digest = *unique_images
            .entry(image_range)
            .or_insert_with(|| Sha256::digest(image).into());
        if image.is_empty() || !valid_sha256(&zone.sha256) || hex_lower(&zone_digest) != zone.sha256
        {
            return Err(invalid_payload("zone SHA-256 or length mismatch"));
        }
        let id_len =
            u32::try_from(zone.id.len()).map_err(|_| invalid_payload("zone ID length overflow"))?;
        inventory_digest.update(id_len.to_be_bytes());
        inventory_digest.update(zone.id.as_bytes());
        inventory_digest.update(zone.offset.to_be_bytes());
        inventory_digest.update(zone.bytes.to_be_bytes());
        inventory_digest.update(zone_digest);
        if is_new_image {
            TimeZone::tzif(&zone.id, image)
                .map_err(|_| invalid_payload(format!("invalid TZif image for {}", zone.id)))?;
        }
        zones.insert(zone.id.clone(), start..end);
    }
    if unique_images.len() != manifest.pack.unique_tzif_images as usize {
        return Err(invalid_payload("unique image count mismatch"));
    }
    if hex_lower(&inventory_digest.finalize()) != manifest.pack.inventory_sha256 {
        return Err(invalid_payload("zone inventory SHA-256 mismatch"));
    }
    let mut cursor = 0;
    for &(start, end) in unique_images.keys() {
        if start != cursor {
            return Err(invalid_payload(
                "overlap or unreferenced bytes in TZif pack",
            ));
        }
        cursor = end;
    }
    if cursor != candidate.pack_bytes.len() {
        return Err(invalid_payload("trailing unreferenced TZif bytes"));
    }
    if SUPPORTED_ZONE_IDS
        .iter()
        .any(|required| !zones.contains_key(*required))
    {
        return Err(invalid_payload(
            "candidate omits a currently supported zone ID",
        ));
    }
    Ok(zones)
}

fn safe_zone_id(zone_id: &str) -> bool {
    !zone_id.is_empty()
        && zone_id.len() <= 255
        && zone_id.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-' | b'+')
        })
        && zone_id
            .split('/')
            .all(|component| !matches!(component, "" | "." | ".."))
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
