// Developer-only integration probe; never bundled into the application.
import Foundation

@main
struct AppleBridgeCheck {
    static func main() throws {
        let engine = RustEngine()
        let decoder = JSONDecoder()
        let inventory = try decoder.decode(ZoneInventory.self, from: engine.execute(.inventory))
        guard inventory.schema == "salah-zone-inventory-v1", inventory.zone_ids.count == 598 else {
            throw ClientError.invalidDocument
        }
        let lookup = try decoder.decode(LookupResponse.self, from: engine.execute(.lookup,
            request: JSONEncoder().encode(LookupRequest(latitude_degrees: 44.9778, longitude_degrees: -93.265))))
        guard lookup.candidates?.zone_ids == ["America/Chicago"] else { throw ClientError.invalidDocument }
        let clock = try decoder.decode(ClockResponse.self, from: engine.execute(.clock,
            request: JSONEncoder().encode(ClockRequest(zone_id: "Pacific/Kiritimati", utc_unix_seconds: 1790856000))))
        guard clock.clock?.offset_seconds_east == 50_400,
              clock.clock?.local_date == "2026-10-02" else { throw ClientError.invalidDocument }
        let cases: [(String, Double, Double, String, String, String)] = [
            ("minneapolis", 44.9778, -93.265, "2026-10-01", "America/Chicago", "hanafi"),
            ("london_dst", 51.5072, -0.1276, "2026-03-29", "Europe/London", "hanafi"),
            ("apia_skipped", -13.8333, -171.75, "2011-12-30", "Pacific/Apia", "standard"),
            ("tromso_polar", 69.6492, 18.9553, "2026-06-21", "Europe/Oslo", "hanafi"),
            ("kathmandu", 27.7172, 85.324, "2026-01-01", "Asia/Kathmandu", "standard"),
            ("kiritimati", 1.8721, -157.4278, "2026-10-01", "Pacific/Kiritimati", "standard"),
        ]
        var fixtures: [String: Any] = [:]
        for (name, latitude, longitude, date, zone, asr) in cases {
            let input = ScheduleRequest(latitude_degrees: latitude, longitude_degrees: longitude,
                                        local_date: date, zone_id: zone, method_id: "mwl-angles-18-17",
                                        asr: asr, zone_choice: "manual")
            let data = try engine.execute(.calculate, request: JSONEncoder().encode(input))
            let response = try decoder.decode(ScheduleResponse.self, from: data)
            guard response.status == "ok", let schedule = response.schedule else { throw ClientError.invalidDocument }
            try schedule.validate()
            guard schedule.requested_local_date == date, schedule.zone_id == zone else { throw ClientError.invalidDocument }
            fixtures[name] = (try JSONSerialization.jsonObject(with: data) as? [String: Any])?["schedule"]
            if name == "apia_skipped" && schedule.civil_date.status != "skipped" { throw ClientError.invalidDocument }
            if name == "tromso_polar" && !schedule.cycles.flatMap(\.events).contains(where: { $0.status == "unavailable" }) {
                throw ClientError.invalidDocument
            }
        }
        for data in [Data("{}".utf8), Data([0xff])] {
            let result = try JSONSerialization.jsonObject(with: engine.execute(.calculate, request: data)) as? [String: Any]
            guard result?["status"] as? String == "error", result?["schedule"] == nil else { throw ClientError.invalidDocument }
        }
        if CommandLine.arguments.count == 2 {
            try JSONSerialization.data(withJSONObject: fixtures, options: [.sortedKeys]).write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
        }
        print("Apple C ABI + Swift decoding: 6 schedules, lookup, date-line clock, skipped/polar status, malformed/UTF-8 rejection passed.")
    }
}
