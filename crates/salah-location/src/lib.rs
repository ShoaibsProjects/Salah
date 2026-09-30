//! Offline, source-versioned suggestions for mapping geographic coordinates
//! to IANA time-zone identifiers.
//!
//! The bundled boundary data is community-maintained and approximate. A map
//! match is always a suggestion: this crate never silently turns a coordinate
//! into an authoritative time-zone selection. Callers must explicitly confirm
//! a suggestion or supply a manual override.

use std::fmt;
use std::sync::OnceLock;

use salah_core::Coordinates;
use salah_time::{SUPPORTED_ZONE_IDS, TZDB_VERSION};
use tzf_rs::EmbeddedFinder;

/// Version of the bundled time-zone boundary database.
pub const BOUNDARY_DATA_VERSION: &str = "2026d";

/// Exact crate release supplying the embedded boundary artifact.
pub const BOUNDARY_DATA_DISTRIBUTION_VERSION: &str = "0.0.2026-d-fix1";

/// SHA-256 of the exact `lite.tzb` artifact from the pinned distribution crate.
pub const BOUNDARY_DATA_SHA256: &str =
    "c1ee211d87027ebc87904d05e52942a644908d1e4ad2d062b0dbbc032eda22e8";

/// Exact `tzf-rs` implementation version used to interpret the bundled data.
pub const LOOKUP_IMPLEMENTATION_VERSION: &str = "2.1.2";

static FINDER: OnceLock<Result<EmbeddedFinder, String>> = OnceLock::new();

fn finder() -> Result<&'static EmbeddedFinder, ZoneLookupError> {
    match FINDER.get_or_init(|| {
        EmbeddedFinder::from_tzb(tzf_dist::load_lite_tzb()).map_err(|error| error.to_string())
    }) {
        Ok(finder) => Ok(finder),
        Err(error) => Err(ZoneLookupError::InvalidBundledData(error.clone())),
    }
}

fn validate_data_version(boundary_data_version: &str) -> Result<(), ZoneLookupError> {
    if boundary_data_version != TZDB_VERSION || boundary_data_version != BOUNDARY_DATA_VERSION {
        return Err(ZoneLookupError::DataVersionMismatch {
            boundary_data_version: boundary_data_version.to_owned(),
            expected_boundary_data_version: BOUNDARY_DATA_VERSION,
            timezone_database_version: TZDB_VERSION,
        });
    }
    Ok(())
}

/// Outcome cardinality for the approximate boundary lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateCardinality {
    /// No polygon lookup was requested. This does not mean no coverage.
    NotLookedUp,
    /// No mapped polygon covers these coordinates.
    NoCoverage,
    /// One polygon in the data pack covers these coordinates. This is still a
    /// suggestion that the user must confirm.
    OneSuggestion,
    /// Multiple polygons cover the point. The library does not choose one.
    MultipleSuggestions,
}

/// A boundary lookup result, or an explicitly unperformed lookup retained for
/// a direct manual choice. Inspect [`Self::lookup_performed`] before treating
/// installed map metadata as lookup evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneCandidates {
    coordinates: Coordinates,
    lookup_performed: bool,
    boundary_data_version: String,
    boundary_distribution_version: &'static str,
    boundary_data_sha256: &'static str,
    lookup_implementation_version: &'static str,
    timezone_database_version: &'static str,
    zone_ids: Vec<String>,
}

impl ZoneCandidates {
    /// Whether the boundary map was actually consulted. Versions on an
    /// unperformed lookup identify the compatible installed map, not evidence.
    #[must_use]
    pub const fn lookup_performed(&self) -> bool {
        self.lookup_performed
    }

    /// Coordinates to which the lookup or manual choice applies.
    #[must_use]
    pub const fn coordinates(&self) -> Coordinates {
        self.coordinates
    }

    /// Installed boundary dataset compatible with this selection. It produced
    /// matches only when [`Self::lookup_performed`] is true.
    #[must_use]
    pub fn boundary_data_version(&self) -> &str {
        &self.boundary_data_version
    }

    /// Pinned crate release that supplies the boundary artifact.
    #[must_use]
    pub const fn boundary_distribution_version(&self) -> &'static str {
        self.boundary_distribution_version
    }

    /// SHA-256 of the exact embedded boundary artifact.
    #[must_use]
    pub const fn boundary_data_sha256(&self) -> &'static str {
        self.boundary_data_sha256
    }

    /// Pinned point-in-polygon implementation version.
    #[must_use]
    pub const fn lookup_implementation_version(&self) -> &'static str {
        self.lookup_implementation_version
    }

    /// Version of the IANA time-zone rules expected by this mapping release.
    #[must_use]
    pub const fn timezone_database_version(&self) -> &'static str {
        self.timezone_database_version
    }

    /// Sorted mapped candidates. An empty slice indicates no coverage only
    /// when [`Self::lookup_performed`] is true.
    #[must_use]
    pub fn zone_ids(&self) -> &[String] {
        &self.zone_ids
    }

    /// Number of mapped candidates, without implying that a unique match is
    /// authoritative.
    #[must_use]
    pub fn cardinality(&self) -> CandidateCardinality {
        if !self.lookup_performed {
            return CandidateCardinality::NotLookedUp;
        }
        match self.zone_ids.len() {
            0 => CandidateCardinality::NoCoverage,
            1 => CandidateCardinality::OneSuggestion,
            _ => CandidateCardinality::MultipleSuggestions,
        }
    }

    /// Record explicit user confirmation of one of the mapped suggestions.
    ///
    /// This is deliberately separate from lookup so callers cannot mistake a
    /// single polygon hit for a user-approved time zone.
    pub fn confirm_suggestion(&self, zone_id: &str) -> Result<ZoneSelection, SelectionError> {
        if !self.zone_ids.iter().any(|candidate| candidate == zone_id) {
            return Err(SelectionError::NotAMappedCandidate(zone_id.to_owned()));
        }

        Ok(ZoneSelection {
            candidates: self.clone(),
            selected_zone_id: zone_id.to_owned(),
            origin: SelectionOrigin::UserConfirmedSuggestion,
        })
    }

    /// Record an explicit user-selected IANA time zone, including when the
    /// boundary data returns no match or several candidates.
    pub fn manual_override(&self, zone_id: &str) -> Result<ZoneSelection, SelectionError> {
        if !SUPPORTED_ZONE_IDS.contains(&zone_id) {
            return Err(SelectionError::UnsupportedTimeZoneId(zone_id.to_owned()));
        }

        Ok(ZoneSelection {
            candidates: self.clone(),
            selected_zone_id: zone_id.to_owned(),
            origin: if self.lookup_performed {
                SelectionOrigin::ManualOverride
            } else {
                SelectionOrigin::ManualWithoutLookup
            },
        })
    }
}

/// How a selected IANA zone became associated with the coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionOrigin {
    /// The user confirmed a zone returned by the approximate boundary map.
    UserConfirmedSuggestion,
    /// The user chose a supported IANA zone independently of the map result.
    ManualOverride,
    /// The user chose a supported zone without requesting a polygon lookup.
    ManualWithoutLookup,
}

/// Select a supported zone directly, without loading or querying the polygon
/// map. The explicit choice applies to the supplied checked coordinates;
/// membership in the inventory does not prove jurisdictional correctness.
pub fn select_manual_zone(
    coordinates: Coordinates,
    zone_id: &str,
) -> Result<ZoneSelection, SelectionError> {
    if !SUPPORTED_ZONE_IDS.contains(&zone_id) {
        return Err(SelectionError::UnsupportedTimeZoneId(zone_id.to_owned()));
    }
    Ok(ZoneSelection {
        candidates: ZoneCandidates {
            coordinates,
            lookup_performed: false,
            boundary_data_version: BOUNDARY_DATA_VERSION.to_owned(),
            boundary_distribution_version: BOUNDARY_DATA_DISTRIBUTION_VERSION,
            boundary_data_sha256: BOUNDARY_DATA_SHA256,
            lookup_implementation_version: LOOKUP_IMPLEMENTATION_VERSION,
            timezone_database_version: TZDB_VERSION,
            zone_ids: Vec::new(),
        },
        selected_zone_id: zone_id.to_owned(),
        origin: SelectionOrigin::ManualWithoutLookup,
    })
}

/// An explicit time-zone choice paired with lookup evidence when requested.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSelection {
    candidates: ZoneCandidates,
    selected_zone_id: String,
    origin: SelectionOrigin,
}

impl ZoneSelection {
    /// Selected IANA identifier.
    #[must_use]
    pub fn zone_id(&self) -> &str {
        &self.selected_zone_id
    }

    /// Coordinates to which this explicit choice applies.
    #[must_use]
    pub const fn coordinates(&self) -> Coordinates {
        self.candidates.coordinates
    }

    /// How this time zone was selected.
    #[must_use]
    pub const fn origin(&self) -> SelectionOrigin {
        self.origin
    }

    /// Boundary candidates and data versions retained as selection evidence.
    #[must_use]
    pub const fn candidates(&self) -> &ZoneCandidates {
        &self.candidates
    }
}

/// Failure while validating the requested selection against this engine's
/// bundled time-zone pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    /// The requested suggestion was not returned for these coordinates.
    NotAMappedCandidate(String),
    /// The identifier is not present in the pinned IANA time-zone pack.
    UnsupportedTimeZoneId(String),
}

impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAMappedCandidate(zone_id) => {
                write!(
                    f,
                    "{zone_id:?} was not a mapped candidate for these coordinates"
                )
            }
            Self::UnsupportedTimeZoneId(zone_id) => write!(
                f,
                "time-zone identifier {zone_id:?} is not in the bundled IANA {TZDB_VERSION} pack"
            ),
        }
    }
}

impl std::error::Error for SelectionError {}

/// Failure while running the bundled coordinate-to-zone lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneLookupError {
    /// The embedded database could not be parsed. This is normally a build or
    /// packaging defect; no host or network fallback is attempted.
    InvalidBundledData(String),
    /// The boundary database and civil-time rule pack must share one IANA
    /// release so every returned identifier can be resolved deterministically.
    DataVersionMismatch {
        boundary_data_version: String,
        expected_boundary_data_version: &'static str,
        timezone_database_version: &'static str,
    },
    /// A boundary dataset identifier is absent from the bundled IANA rule pack.
    UnsupportedMappedZone(String),
}

impl fmt::Display for ZoneLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBundledData(reason) => {
                write!(f, "bundled timezone boundary data is invalid: {reason}")
            }
            Self::DataVersionMismatch {
                boundary_data_version,
                expected_boundary_data_version,
                timezone_database_version,
            } => write!(
                f,
                "boundary data {boundary_data_version} does not match pinned boundary data {expected_boundary_data_version} and IANA rules {timezone_database_version}"
            ),
            Self::UnsupportedMappedZone(zone_id) => write!(
                f,
                "boundary data returned {zone_id:?}, which is absent from the bundled IANA {TZDB_VERSION} pack"
            ),
        }
    }
}

impl std::error::Error for ZoneLookupError {}

/// Find every approximate boundary polygon covering a validated point.
///
/// The boundary dataset and time-zone rules are pinned to IANA `2026d`. If a
/// later dependency update changes either version without updating the other,
/// this function fails closed. Longitude is passed to the mapping engine
/// before latitude; the public `Coordinates` constructor takes latitude first.
pub fn lookup_timezone_candidates(
    coordinates: Coordinates,
) -> Result<ZoneCandidates, ZoneLookupError> {
    let finder = finder()?;
    let boundary_data_version = finder.data_version();
    validate_data_version(boundary_data_version)?;

    let mut zone_ids: Vec<String> = finder
        .get_tz_names(
            coordinates.longitude_degrees(),
            coordinates.latitude_degrees(),
        )
        .into_iter()
        .map(str::to_owned)
        .collect();
    zone_ids.sort_unstable();
    zone_ids.dedup();

    if let Some(unsupported) = zone_ids
        .iter()
        .find(|zone_id| !SUPPORTED_ZONE_IDS.contains(&zone_id.as_str()))
    {
        return Err(ZoneLookupError::UnsupportedMappedZone(unsupported.clone()));
    }

    Ok(ZoneCandidates {
        coordinates,
        lookup_performed: true,
        boundary_data_version: boundary_data_version.to_owned(),
        boundary_distribution_version: BOUNDARY_DATA_DISTRIBUTION_VERSION,
        boundary_data_sha256: BOUNDARY_DATA_SHA256,
        lookup_implementation_version: LOOKUP_IMPLEMENTATION_VERSION,
        timezone_database_version: TZDB_VERSION,
        zone_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateCardinality, SelectionError, SelectionOrigin, ZoneCandidates, ZoneLookupError,
        validate_data_version,
    };
    use salah_core::Coordinates;

    #[test]
    fn empty_candidate_set_never_confirms_a_zone_but_allows_override() {
        let coordinates = Coordinates::new(0.0, 0.0).expect("valid coordinates");
        let candidates = ZoneCandidates {
            coordinates,
            lookup_performed: true,
            boundary_data_version: super::BOUNDARY_DATA_VERSION.to_owned(),
            boundary_distribution_version: super::BOUNDARY_DATA_DISTRIBUTION_VERSION,
            boundary_data_sha256: super::BOUNDARY_DATA_SHA256,
            lookup_implementation_version: super::LOOKUP_IMPLEMENTATION_VERSION,
            timezone_database_version: super::TZDB_VERSION,
            zone_ids: Vec::new(),
        };

        assert_eq!(candidates.cardinality(), CandidateCardinality::NoCoverage);
        assert_eq!(
            candidates.confirm_suggestion("Etc/UTC"),
            Err(SelectionError::NotAMappedCandidate("Etc/UTC".to_owned()))
        );
        let override_choice = candidates
            .manual_override("Etc/UTC")
            .expect("supported manual zone is accepted");
        assert_eq!(override_choice.origin(), SelectionOrigin::ManualOverride);
    }

    #[test]
    fn boundary_and_rule_data_versions_must_remain_aligned() {
        assert!(validate_data_version(super::BOUNDARY_DATA_VERSION).is_ok());
        assert_eq!(
            validate_data_version("2026c"),
            Err(ZoneLookupError::DataVersionMismatch {
                boundary_data_version: "2026c".to_owned(),
                expected_boundary_data_version: super::BOUNDARY_DATA_VERSION,
                timezone_database_version: super::TZDB_VERSION,
            })
        );
    }
}
