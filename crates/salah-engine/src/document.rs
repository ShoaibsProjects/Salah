//! Versioned, read-only serialization boundary for CLI and future clients.
//! No calculation, timezone selection, rounding, or update activation occurs.

use core::fmt;

use salah_core::{AsrCriterion, EventRule, UnavailableReason};
use salah_location::{CandidateCardinality, SelectionOrigin};
use salah_time::{
    LocalCivilTime, LocalDatePrayerScheduleCandidate, LocalDatePrayerScheduleMatches,
    LocalDateStatus, LocalizedEvent, RulePackIdentity,
};
use serde_json::{Value, json};

use crate::{RuntimeRuleSource, SelectedLocalDaySchedule};

/// Additive fields may appear within this schema; semantic changes require a
/// new schema identity. Consumers must check this field before interpreting.
pub const SCHEDULE_DOCUMENT_SCHEMA: &str = "salah-local-schedule-v1";

/// Reject non-finite floating values rather than silently encoding JSON null.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleDocumentError {
    pub field: &'static str,
}

impl fmt::Display for ScheduleDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot export non-finite schedule field: {}", self.field)
    }
}

impl std::error::Error for ScheduleDocumentError {}

fn finite(value: f64, field: &'static str) -> Result<f64, ScheduleDocumentError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ScheduleDocumentError { field })
    }
}

fn policy(id: &str, revision: &str) -> Value {
    json!({"id": id, "revision": revision})
}

fn identity(pack: &RulePackIdentity) -> Value {
    json!({
        "schema_version": pack.schema_version,
        "tzdb_version": pack.tzdb_version,
        "release_year": pack.release_year,
        "sha256": pack.sha256,
        "inventory_sha256": pack.inventory_sha256,
    })
}

fn local_reading(local: &LocalCivilTime) -> Value {
    json!({
        "utc_unix_seconds": local.utc.unix_seconds,
        "utc": format!("{}Z", local.utc.to_utc()).replace(' ', "T"),
        "local_date": local.local.date.to_string(),
        "local_time": format!("{:02}:{:02}:{:02}", local.local.hour, local.local.minute, local.local.second),
        "offset_seconds_east": local.offset_seconds_east,
        "zone_id": local.zone_id,
        "tzdb_version": local.tzdb_version,
        "rule_pack": identity(&local.rule_pack),
    })
}

fn event_rule(rule: EventRule) -> Result<Value, ScheduleDocumentError> {
    Ok(match rule {
        EventRule::SolarTransit => json!({"kind": "solar_transit"}),
        EventRule::ApparentHorizon => json!({"kind": "apparent_horizon"}),
        EventRule::SolarDepression { degrees } => json!({
            "kind": "solar_depression", "degrees": finite(degrees, "rule.degrees")?,
        }),
        EventRule::AsrShadow { factor } => json!({
            "kind": "asr_shadow", "factor": finite(factor, "rule.factor")?,
        }),
        EventRule::SunsetWithAdjustment { seconds } => {
            json!({"kind": "sunset_with_adjustment", "seconds": seconds})
        }
        EventRule::TransitWithAdjustment { seconds } => {
            json!({"kind": "transit_with_adjustment", "seconds": seconds})
        }
    })
}

fn event(name: &str, event: &LocalizedEvent) -> Result<Value, ScheduleDocumentError> {
    Ok(match event {
        LocalizedEvent::Occurs {
            local,
            unrounded_utc_unix_seconds,
            rule,
        } => json!({
            "name": name,
            "status": "occurs",
            "reading": local_reading(local),
            "unrounded_utc_unix_seconds": finite(*unrounded_utc_unix_seconds, "event.unrounded_utc_unix_seconds")?,
            "rule": event_rule(*rule)?,
        }),
        LocalizedEvent::Unavailable { reason } => {
            let code = match reason {
                UnavailableReason::NoCrossingInSolarCycle => "no_crossing_in_solar_cycle",
            };
            json!({"name": name, "status": "unavailable", "reason": code})
        }
    })
}

fn cycle(candidate: &LocalDatePrayerScheduleCandidate) -> Result<Value, ScheduleDocumentError> {
    let record = &candidate.cycle.record;
    let e = &candidate.events;
    let events = [
        ("fajr", &e.fajr),
        ("sunrise", &e.sunrise),
        ("dhuhr", &e.dhuhr),
        ("asr", &e.asr),
        ("sunset", &e.sunset),
        ("maghrib", &e.maghrib),
        ("isha", &e.isha),
    ]
    .into_iter()
    .map(|(name, value)| event(name, value))
    .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "anchor_utc_unix_seconds": record.anchor.unix_seconds,
        "selected_transit": local_reading(&candidate.local_transit),
        "unrounded_selected_transit_unix_seconds": finite(record.unrounded_selected_transit_unix_seconds, "cycle.unrounded_selected_transit_unix_seconds")?,
        "anchor_selection_policy": policy(record.selection_policy_id, record.selection_policy_revision),
        "high_latitude_rule": record.high_latitude_rule,
        "assumed_elevation_meters": finite(record.assumed_elevation_meters, "cycle.assumed_elevation_meters")?,
        "events": events,
    }))
}

/// Export a complete local schedule with explicit cardinality and provenance.
/// Missing events have no time fields; a skipped civil date is separate from
/// zero solar-cycle matches. Event-local dates and second-precision offsets
/// are preserved. No display-minute or notification policy is applied.
///
/// This encodes a result; it does not authenticate caller-constructed records
/// or certify the religious/astronomical suitability of the selected method.
pub fn schedule_document(
    selected: &SelectedLocalDaySchedule,
) -> Result<Value, ScheduleDocumentError> {
    let schedule = &selected.schedule;
    let record = &schedule.record;
    let selection = &selected.zone_selection;
    let candidates = selection.candidates();
    let origin = match selection.origin() {
        SelectionOrigin::UserConfirmedSuggestion => "user_confirmed_suggestion",
        SelectionOrigin::ManualOverride => "manual_override",
        SelectionOrigin::ManualWithoutLookup => "manual_without_lookup",
    };
    let cardinality = match candidates.cardinality() {
        CandidateCardinality::NotLookedUp => "not_looked_up",
        CandidateCardinality::NoCoverage => "no_coverage",
        CandidateCardinality::OneSuggestion => "one_suggestion",
        CandidateCardinality::MultipleSuggestions => "multiple_suggestions",
    };
    let boundary_lookup = if candidates.lookup_performed() {
        json!({
            "status": "performed", "cardinality": cardinality, "zone_ids": candidates.zone_ids(),
            "data_version": candidates.boundary_data_version(),
            "distribution_version": candidates.boundary_distribution_version(),
            "sha256": candidates.boundary_data_sha256(),
            "implementation_version": candidates.lookup_implementation_version(),
        })
    } else {
        json!({"status": "not_performed", "cardinality": cardinality})
    };
    let runtime_source = match &selected.runtime_source {
        RuntimeRuleSource::Bundled => json!({"kind": "bundled"}),
        RuntimeRuleSource::SignedPackage {
            sequence,
            key_id,
            boundary_sha256,
        } => json!({
            "kind": "signed_package", "sequence": sequence, "key_id": key_id, "boundary_sha256": boundary_sha256,
        }),
    };
    let (civil_status, intervals) = match &schedule.civil_date.status {
        LocalDateStatus::Skipped => ("skipped", Vec::new()),
        LocalDateStatus::Exists(intervals) => (
            "exists",
            intervals
                .iter()
                .map(|interval| {
                    json!({
                        "start_utc_unix_seconds": interval.start.unix_seconds,
                        "end_utc_unix_seconds_inclusive": interval.end.unix_seconds,
                        "offset_seconds_east": interval.offset_seconds_east,
                    })
                })
                .collect(),
        ),
    };
    let (match_status, cycles) = match &schedule.matches {
        LocalDatePrayerScheduleMatches::Zero => ("zero", Vec::new()),
        LocalDatePrayerScheduleMatches::One(candidate) => ("one", vec![cycle(candidate)?]),
        LocalDatePrayerScheduleMatches::Multiple(candidates) => (
            "multiple",
            candidates
                .iter()
                .map(cycle)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    };
    Ok(json!({
        "schema": SCHEDULE_DOCUMENT_SCHEMA,
        "scope": "research_preview",
        "requested_local_date": record.requested_date.to_string(),
        "coordinates": {
            "latitude_degrees": finite(record.coordinates.latitude_degrees(), "coordinates.latitude_degrees")?,
            "longitude_degrees": finite(record.coordinates.longitude_degrees(), "coordinates.longitude_degrees")?,
        },
        "zone_id": record.zone_id,
        "zone_selection": {"origin": origin, "boundary_lookup": boundary_lookup},
        "rule_pack": identity(&record.rule_pack),
        "runtime_source": runtime_source,
        "kernel": {"version": record.engine_version, "astronomy_model": record.astronomy_model},
        "method": {
            "id": record.method.id, "revision": record.method.revision, "source": record.method.source,
            "fajr_depression_degrees": finite(record.method.fajr_depression_degrees, "method.fajr_depression_degrees")?,
            "isha_depression_degrees": finite(record.method.isha_depression_degrees, "method.isha_depression_degrees")?,
            "dhuhr_adjustment_seconds": record.method.dhuhr_adjustment_seconds,
            "maghrib_adjustment_seconds": record.method.maghrib_adjustment_seconds,
            "asr": match record.asr_criterion {AsrCriterion::Standard => "standard", AsrCriterion::Hanafi => "hanafi"},
        },
        "schedule_policy": policy(schedule.policy_id, schedule.policy_revision),
        "transit_selection_policy": policy(record.policy_id, record.policy_revision),
        "civil_date": {
            "status": civil_status,
            "policy": policy(schedule.civil_date.policy_id, schedule.civil_date.policy_revision),
            "utc_intervals": intervals,
        },
        "cycle_match_status": match_status,
        "cycles": cycles,
    }))
}
