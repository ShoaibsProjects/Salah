use salah_core::UtcInstant;
use salah_time::{
    BUNDLED_RULE_PACK_IDENTITY, SUPPORTED_ZONE_IDS, convert_utc_to_local, fixture_tzif_bytes,
};

#[test]
fn full_iana_2026d_pack_indexes_and_parses_all_named_zones() {
    assert_eq!(SUPPORTED_ZONE_IDS.len(), 598);
    assert!(SUPPORTED_ZONE_IDS.windows(2).all(|pair| pair[0] < pair[1]));

    let epoch = UtcInstant { unix_seconds: 0 };
    for zone_id in SUPPORTED_ZONE_IDS {
        let bytes = fixture_tzif_bytes(zone_id)
            .unwrap_or_else(|error| panic!("missing generated TZif bytes for {zone_id}: {error}"));
        assert!(
            bytes.starts_with(b"TZif"),
            "invalid TZif image for {zone_id}"
        );
        let converted = convert_utc_to_local(epoch, zone_id, bytes)
            .unwrap_or_else(|error| panic!("cannot parse {zone_id}: {error}"));
        assert_eq!(converted.zone_id, *zone_id);
        assert_eq!(converted.tzdb_version, "2026d");
        assert_eq!(converted.rule_pack, BUNDLED_RULE_PACK_IDENTITY);
    }
}
