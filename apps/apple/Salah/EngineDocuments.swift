import Foundation

// These labels are decoded from the shared Rust JSON. Swift does no prayer or
// timezone arithmetic and does not reinterpret an event in the device's zone.
struct EngineDiagnostic: Decodable, Sendable {
    let code: String
    let message: String
}

struct RulePack: Decodable, Sendable {
    let tzdb_version: String
    let sha256: String
    let inventory_sha256: String
}

struct ZoneInventory: Decodable, Sendable {
    let schema: String
    let scope: String
    let zone_ids: [String]
    let rule_pack: RulePack
}

struct ZoneCandidates: Decodable, Sendable {
    let latitude_degrees: Double
    let longitude_degrees: Double
    let zone_ids: [String]
    let requires_confirmation: Bool
    let accuracy_footprint_checked: Bool
    let boundary_version: String
    let boundary_sha256: String
    let tzdb_version: String
}

struct LookupResponse: Decodable, Sendable {
    let schema: String
    let status: String
    let candidates: ZoneCandidates?
    let error: EngineDiagnostic?
}

struct ClockReading: Decodable, Sendable {
    let source: String
    let utc_unix_seconds: Int64
    let zone_id: String
    let local_date: String
    let local_time: String
    let offset_seconds_east: Int
    let rule_pack: RulePack
}

struct ClockResponse: Decodable, Sendable {
    let schema: String
    let status: String
    let clock: ClockReading?
    let error: EngineDiagnostic?
}

struct LookupRequest: Encodable, Sendable {
    let schema = "salah-zone-lookup-request-v1"
    let latitude_degrees: Double
    let longitude_degrees: Double
}

struct ClockRequest: Encodable, Sendable {
    let schema = "salah-local-clock-request-v1"
    let zone_id: String
    let utc_unix_seconds: Int64
}

struct ScheduleRequest: Encodable, Sendable {
    let schema = "salah-schedule-request-v2"
    let latitude_degrees: Double
    let longitude_degrees: Double
    let local_date: String
    let zone_id: String
    let method_id: String
    let asr: String
    let zone_choice: String
}

struct EventReading: Decodable, Sendable {
    let utc_unix_seconds: Int64
    let utc: String
    let local_date: String
    let local_time: String
    let offset_seconds_east: Int
    let zone_id: String
    let tzdb_version: String
}

struct EventRule: Decodable, Sendable {
    let kind: String
    let degrees: Double?
    let factor: Double?
    let seconds: Int?

    var explanation: String {
        switch kind {
        case "solar_depression":
            return "The Sun is \(degrees.map { String($0) } ?? "?")° below the horizon under the selected calculation rule."
        case "asr_shadow":
            return "The selected shadow factor is \(factor.map { String($0) } ?? "?"). The engine includes the shadow at solar noon."
        case "apparent_horizon":
            return "The engine's stated apparent-horizon rule, at assumed sea level. Terrain and local weather are not measured."
        case "solar_transit": return "The calculated solar transit."
        case "transit_with_adjustment": return "Solar transit with the method's \(seconds ?? 0)-second adjustment."
        case "sunset_with_adjustment": return "Calculated sunset with the method's \(seconds ?? 0)-second adjustment."
        default: return "Recorded engine rule: \(kind)."
        }
    }
}

struct ScheduleEvent: Decodable, Sendable, Identifiable {
    let name: String
    let status: String
    let reading: EventReading?
    let rule: EventRule?
    let reason: String?
    var id: String { name }
    var title: String { name == "dhuhr" ? "Dhuhr" : name.capitalized }
}

struct SolarCycle: Decodable, Sendable {
    let selected_transit: EventReading
    let high_latitude_rule: String
    let events: [ScheduleEvent]
}

struct CivilDate: Decodable, Sendable { let status: String }
struct KernelRecord: Decodable, Sendable { let version: String; let astronomy_model: String }
struct MethodRecord: Decodable, Sendable { let id: String; let revision: String; let source: String; let asr: String }

struct ScheduleDocument: Decodable, Sendable {
    let schema: String
    let scope: String
    let requested_local_date: String
    let zone_id: String
    let rule_pack: RulePack
    let kernel: KernelRecord
    let method: MethodRecord
    let civil_date: CivilDate
    let cycle_match_status: String
    let cycles: [SolarCycle]

    func validate() throws {
        guard schema == "salah-local-schedule-v1", scope == "research_preview",
              ["exists", "skipped"].contains(civil_date.status),
              ["zero", "one", "multiple"].contains(cycle_match_status),
              (cycle_match_status != "zero" || cycles.isEmpty),
              (cycle_match_status != "one" || cycles.count == 1),
              (cycle_match_status != "multiple" || cycles.count > 1),
              (civil_date.status != "skipped" || cycles.isEmpty) else {
            throw ClientError.invalidDocument
        }
        let names: Set<String> = ["fajr", "sunrise", "dhuhr", "asr", "sunset", "maghrib", "isha"]
        for cycle in cycles {
            guard cycle.events.count == names.count,
                  Set(cycle.events.map(\.name)) == names else { throw ClientError.invalidDocument }
            for event in cycle.events {
                switch event.status {
                case "occurs":
                    guard let reading = event.reading, event.rule != nil,
                          reading.zone_id == zone_id else { throw ClientError.invalidDocument }
                case "unavailable":
                    guard event.reading == nil, event.reason != nil else { throw ClientError.invalidDocument }
                default: throw ClientError.invalidDocument
                }
            }
        }
    }
}

struct ScheduleResponse: Decodable, Sendable {
    let schema: String
    let status: String
    let schedule: ScheduleDocument?
    let error: EngineDiagnostic?
}

enum ClientError: LocalizedError {
    case invalidDocument
    case message(String)
    var errorDescription: String? {
        switch self {
        case .invalidDocument: return "The engine returned an unsupported record. No replacement schedule was created."
        case .message(let message): return message
        }
    }
}

// Calls are serialized off the main actor. The same queue handles map/date and
// schedule operations so the UI cannot concurrently enter the native bridge.
actor EngineGateway {
    private let engine = RustEngine()
    func run<Response: Decodable & Sendable>(_ operation: EngineOperation, request: Data = Data()) throws -> Response {
        let bytes = try engine.execute(operation, request: request)
        return try JSONDecoder().decode(Response.self, from: bytes)
    }
}
