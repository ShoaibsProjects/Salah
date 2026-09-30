use salah_core::Coordinates;
use salah_location::{
    BOUNDARY_DATA_DISTRIBUTION_VERSION, BOUNDARY_DATA_SHA256, CandidateCardinality,
    LOOKUP_IMPLEMENTATION_VERSION, SelectionError, SelectionOrigin, lookup_timezone_candidates,
};
use salah_time::{SUPPORTED_ZONE_IDS, TZDB_VERSION};
use tzf_rs::EmbeddedFinder;

fn coordinates(latitude: f64, longitude: f64) -> Coordinates {
    Coordinates::new(latitude, longitude).expect("test coordinates are valid")
}

#[test]
fn city_lookup_uses_latitude_longitude_input_and_matches_the_pinned_rules() {
    let result = lookup_timezone_candidates(coordinates(39.9289, 116.3883)).unwrap();

    assert_eq!(result.zone_ids(), &["Asia/Shanghai"]);
    assert_eq!(result.cardinality(), CandidateCardinality::OneSuggestion);
    assert_eq!(result.boundary_data_version(), "2026d");
    assert_eq!(
        result.boundary_distribution_version(),
        BOUNDARY_DATA_DISTRIBUTION_VERSION
    );
    assert_eq!(result.boundary_data_sha256(), BOUNDARY_DATA_SHA256);
    assert_eq!(
        result.lookup_implementation_version(),
        LOOKUP_IMPLEMENTATION_VERSION
    );
    assert_eq!(result.timezone_database_version(), TZDB_VERSION);
}

#[test]
fn ocean_polygon_assignment_is_explicit_and_can_be_manually_overridden() {
    let result = lookup_timezone_candidates(coordinates(0.0, 0.0)).unwrap();

    assert_eq!(result.zone_ids(), &["Etc/GMT"]);
    assert_eq!(result.cardinality(), CandidateCardinality::OneSuggestion);

    let selection = result.manual_override("Etc/UTC").unwrap();
    assert_eq!(selection.zone_id(), "Etc/UTC");
    assert_eq!(selection.origin(), SelectionOrigin::ManualOverride);
    assert_eq!(selection.candidates().zone_ids(), &["Etc/GMT"]);
}

#[test]
fn a_single_polygon_hit_still_requires_explicit_confirmation() {
    let result = lookup_timezone_candidates(coordinates(39.9289, 116.3883)).unwrap();
    let selection = result
        .confirm_suggestion("Asia/Shanghai")
        .expect("the user can confirm the mapped suggestion");

    assert_eq!(selection.zone_id(), "Asia/Shanghai");
    assert_eq!(selection.origin(), SelectionOrigin::UserConfirmedSuggestion);
}

#[test]
fn candidate_confirmation_and_manual_override_validate_differently() {
    let result = lookup_timezone_candidates(coordinates(39.9289, 116.3883)).unwrap();

    assert_eq!(
        result.confirm_suggestion("America/Chicago"),
        Err(SelectionError::NotAMappedCandidate(
            "America/Chicago".to_owned()
        ))
    );
    assert_eq!(
        result.manual_override("Mars/Olympus_Mons"),
        Err(SelectionError::UnsupportedTimeZoneId(
            "Mars/Olympus_Mons".to_owned()
        ))
    );
    assert_eq!(
        result.manual_override("America/Chicago").unwrap().origin(),
        SelectionOrigin::ManualOverride
    );
}

#[test]
fn every_zone_in_the_boundary_dataset_is_supported_by_the_same_iana_pack() {
    let finder = EmbeddedFinder::from_tzb(tzf_dist::load_lite_tzb()).unwrap();

    assert_eq!(finder.data_version(), TZDB_VERSION);
    for zone_id in finder.timezonenames() {
        assert!(
            SUPPORTED_ZONE_IDS.contains(&zone_id),
            "boundary zone {zone_id:?} is not available in the pinned IANA pack"
        );
    }
}

#[test]
fn points_on_a_timezone_border_are_reported_as_multiple_candidates() {
    // Upstream's exact shared-boundary regression vector: both adjacent
    // nautical polygons claim this point. Keep it explicit so no first-match
    // tie-break is introduced here.
    let result = lookup_timezone_candidates(coordinates(54.5, 7.5)).unwrap();

    assert_eq!(
        result.cardinality(),
        CandidateCardinality::MultipleSuggestions
    );
    assert_eq!(result.zone_ids(), ["Etc/GMT", "Etc/GMT-1"]);
}
