//! Immutable, integrity-checked timezone payloads. Authentication is owned
//! by `salah-update`; a successful load here does not establish a trusted signer.

use crate::{
    RulePackIdentity, RuntimeZone, SUPPORTED_ZONE_IDS, TZDB_PACK_INVENTORY_SHA256,
    TZDB_PACK_SHA256, TZDB_VERSION, TimeError,
};
use jiff::tz::TimeZone;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;
use std::ops::Range;

/// Maximum manifest bytes in the schema 1 rule-pack format.
pub const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024;
/// Maximum TZif blob bytes in the schema 1 rule-pack format.
pub const MAX_PACK_BYTES: usize = 32 * 1024 * 1024;
/// Maximum license bytes in the schema 1 rule-pack format.
pub const MAX_LICENSE_BYTES: usize = 256 * 1024;
const MAX_ZONE_IDS: usize = 2048;

/// Raw bounded payload. A signature must be checked separately before using
/// this as an application update; this type carries no authentication claim.
pub struct RulePackPayload<'a> {
    pub manifest_bytes: &'a [u8],
    pub pack_bytes: &'a [u8],
    pub license_bytes: &'a [u8],
}

/// Payload-integrity errors, separate from signer authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulePackError {
    PayloadTooLarge,
    InvalidManifest(String),
    InvalidPayload(String),
}
impl fmt::Display for RulePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge => f.write_str("rule pack exceeds format limits"),
            Self::InvalidManifest(reason) => write!(f, "invalid rule-pack manifest: {reason}"),
            Self::InvalidPayload(reason) => write!(f, "invalid rule-pack payload: {reason}"),
        }
    }
}
impl std::error::Error for RulePackError {}

/// Exact immutable blob and zone inventory after bounded integrity validation.
/// This is not a signature verification result. The engine's authenticated
/// path requires a `salah_update::VerifiedRulePack`, not this type alone.
pub struct ValidatedRulePack {
    identity: RulePackIdentity,
    pack_bytes: Vec<u8>,
    zones: BTreeMap<String, Range<usize>>,
}
impl ValidatedRulePack {
    /// Validate the same schema, checksums, parser and name-coverage rules as
    /// the signed-update verifier. No filesystem or host database is read.
    pub fn from_payload(payload: RulePackPayload<'_>) -> Result<Self, RulePackError> {
        if payload.manifest_bytes.is_empty()
            || payload.manifest_bytes.len() > MAX_MANIFEST_BYTES
            || payload.pack_bytes.is_empty()
            || payload.pack_bytes.len() > MAX_PACK_BYTES
            || payload.license_bytes.is_empty()
            || payload.license_bytes.len() > MAX_LICENSE_BYTES
        {
            return Err(RulePackError::PayloadTooLarge);
        }
        let manifest: Manifest = serde_json::from_slice(payload.manifest_bytes)
            .map_err(|error| RulePackError::InvalidManifest(error.to_string()))?;
        let zones = validate_payload(&manifest, &payload)?;
        Ok(Self {
            identity: RulePackIdentity {
                schema_version: manifest.schema_version,
                tzdb_version: Cow::Owned(manifest.data_version),
                release_year: manifest.release_year,
                sha256: Cow::Owned(manifest.pack.sha256),
                inventory_sha256: Cow::Owned(manifest.pack.inventory_sha256),
            },
            pack_bytes: payload.pack_bytes.to_vec(),
            zones,
        })
    }
    #[must_use]
    pub fn identity(&self) -> &RulePackIdentity {
        &self.identity
    }
    #[must_use]
    pub fn pack_bytes(&self) -> &[u8] {
        &self.pack_bytes
    }
    #[must_use]
    pub fn zone_bytes(&self, zone_id: &str) -> Option<&[u8]> {
        self.zones
            .get(zone_id)
            .and_then(|range| self.pack_bytes.get(range.clone()))
    }
    /// Prepare one zone only from its validated inventory entry. The caller
    /// cannot supply different bytes or overwrite the recorded pack identity.
    pub fn zone(&self, zone_id: &str) -> Result<RuntimeZone, TimeError> {
        let bytes = self
            .zone_bytes(zone_id)
            .ok_or_else(|| TimeError::InvalidZoneId(zone_id.to_owned()))?;
        let zone = TimeZone::tzif(zone_id, bytes)
            .map_err(|error| TimeError::MalformedTzif(error.to_string()))?;
        Ok(RuntimeZone {
            zone_id: zone_id.to_owned(),
            zone,
            identity: self.identity.clone(),
        })
    }
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

fn invalid_payload(reason: impl Into<String>) -> RulePackError {
    RulePackError::InvalidPayload(reason.into())
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
    candidate: &RulePackPayload<'_>,
) -> Result<BTreeMap<String, Range<usize>>, RulePackError> {
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
