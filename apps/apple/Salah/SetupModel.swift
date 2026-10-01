import Foundation
import SwiftUI

@MainActor
final class SetupModel: ObservableObject {
    @Published private(set) var latitude = ""
    @Published private(set) var longitude = ""
    @Published private(set) var zoneID = ""
    @Published private(set) var zoneConfirmed = false
    @Published private(set) var methodID = ""
    @Published private(set) var asr = ""
    @Published private(set) var useToday = true
    @Published private(set) var manualDate = Date()
    @Published private(set) var clock: ClockReading?
    @Published private(set) var inventory: ZoneInventory?
    @Published private(set) var candidates: ZoneCandidates?
    @Published private(set) var schedule: ScheduleDocument?
    @Published private(set) var errorMessage: String?
    @Published private(set) var zoneMessage = "Enter coordinates or request a device estimate. A supported timezone can also be chosen manually."
    @Published private(set) var locationMessage = "Salah uses coordinates on this device and does not send them to a server. Manual entry always works without a location fix."
    @Published private(set) var isLocating = false
    @Published private(set) var isCalculating = false
    @Published private(set) var isLookingUp = false
    @Published private(set) var deviceFix: DeviceFix?
    @Published private(set) var exampleLoaded = false

    private let gateway = EngineGateway()
    private let location = DeviceLocation()
    private var revision: UInt64 = 0
    private var lookupGeneration: UInt64 = 0
    private var lookupTask: Task<Void, Never>?
    private var calculationTask: Task<Void, Never>?
    private var clockTask: Task<Void, Never>?
    private var startupTask: Task<Void, Never>?

    static var dateCalendar: Calendar {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: 0)!
        return calendar
    }

    static func dateText(_ date: Date) -> String {
        let c = dateCalendar.dateComponents([.year, .month, .day], from: date)
        return String(format: "%04d-%02d-%02d", c.year ?? 0, c.month ?? 0, c.day ?? 0)
    }

    static func date(year: Int, month: Int, day: Int) -> Date {
        dateCalendar.date(from: DateComponents(year: year, month: month, day: day, hour: 12))!
    }

    var canCalculate: Bool {
        inventory != nil && coordinates != nil && zoneConfirmed && !zoneID.isEmpty &&
        !methodID.isEmpty && !asr.isEmpty && !isCalculating && !isLocating
    }

    private var coordinates: (Double, Double)? {
        guard let a = Double(latitude), let b = Double(longitude), a.isFinite, b.isFinite,
              (-90...90).contains(a), (-180...180).contains(b) else { return nil }
        return (a, b)
    }

    func start() {
        guard startupTask == nil, inventory == nil else { return }
        startupTask = Task { [weak self] in
            guard let self else { return }
            defer { self.startupTask = nil }
            do {
                let document: ZoneInventory = try await gateway.run(.inventory)
                guard document.schema == "salah-zone-inventory-v1", document.scope == "research_preview",
                      !document.zone_ids.isEmpty, Set(document.zone_ids).count == document.zone_ids.count else {
                    throw ClientError.invalidDocument
                }
                inventory = document
                if coordinates != nil { startLookup() }
            } catch { errorMessage = "The bundled engine could not start. \(error.localizedDescription)" }
        }
    }

    private func changed() {
        revision &+= 1
        calculationTask?.cancel()
        calculationTask = nil
        clockTask?.cancel()
        clockTask = nil
        schedule = nil
        errorMessage = nil
        isCalculating = false
        exampleLoaded = false
    }

    func editLatitude(_ value: String) { latitude = value; coordinatesChanged(manual: true) }
    func editLongitude(_ value: String) { longitude = value; coordinatesChanged(manual: true) }

    private func coordinatesChanged(manual: Bool) {
        changed()
        if manual {
            cancelLocation()
            deviceFix = nil
            locationMessage = "Using manually entered coordinates. Confirm the offline timezone suggestion or choose another zone."
        }
        candidates = nil
        zoneID = ""
        zoneConfirmed = false
        clock = nil
        startLookup()
    }

    private func startLookup() {
        cancelLookup()
        guard inventory != nil, let (a, b) = coordinates else {
            zoneMessage = "Enter valid latitude (−90 to 90) and longitude (−180 to 180), or choose a timezone manually."
            return
        }
        let requestedGeneration = lookupGeneration
        zoneMessage = "Checking the embedded timezone map…"
        isLookingUp = true
        lookupTask = Task { [weak self] in
            do { try await Task.sleep(for: .milliseconds(350)) } catch { return }
            guard let self else { return }
            do {
                let data = try JSONEncoder().encode(LookupRequest(latitude_degrees: a, longitude_degrees: b))
                let response: LookupResponse = try await gateway.run(.lookup, request: data)
                guard !Task.isCancelled, requestedGeneration == lookupGeneration else { return }
                guard response.schema == "salah-zone-lookup-response-v1", response.status == "ok",
                      let result = response.candidates else {
                    throw ClientError.message(response.error?.message ?? "The map could not be read.")
                }
                guard result.latitude_degrees == a, result.longitude_degrees == b,
                      result.requires_confirmation, !result.accuracy_footprint_checked,
                      result.zone_ids.allSatisfy({ inventory?.zone_ids.contains($0) == true }) else {
                    throw ClientError.invalidDocument
                }
                candidates = result
                if result.zone_ids.count == 1 {
                    zoneID = result.zone_ids[0]
                    zoneMessage = "Suggested by the offline \(result.boundary_version) map. Confirm this zone; approximate borders and location error can affect it."
                } else if result.zone_ids.isEmpty {
                    zoneMessage = "The map has no zone at this point. Choose the location's supported zone manually; ships or aircraft may follow their operator's clock."
                } else {
                    zoneMessage = "Several zones are possible: \(result.zone_ids.joined(separator: ", ")). Choose and confirm one."
                }
                isLookingUp = false
            } catch {
                guard !Task.isCancelled, requestedGeneration == lookupGeneration else { return }
                isLookingUp = false
                zoneMessage = "\(error.localizedDescription) Choose a supported zone manually."
            }
        }
    }

    private func cancelLookup() {
        lookupGeneration &+= 1
        lookupTask?.cancel()
        lookupTask = nil
        isLookingUp = false
    }

    func chooseZone(_ value: String) {
        changed()
        cancelLookup()
        zoneID = value
        zoneConfirmed = false
        clock = nil
        zoneMessage = "Confirm that this timezone applies to your selected coordinates."
    }

    func confirmZone(_ value: Bool) {
        changed()
        cancelLookup()
        zoneConfirmed = value && inventory?.zone_ids.contains(zoneID) == true
        clock = nil
        if zoneConfirmed { refreshToday() }
    }

    func chooseMethod(_ value: String) { changed(); methodID = value; refreshToday() }
    func chooseAsr(_ value: String) { changed(); asr = value; refreshToday() }
    func todayMode(_ value: Bool) { changed(); useToday = value; clock = nil; if value { refreshToday() } }
    func chooseDate(_ value: Date) { changed(); manualDate = value; useToday = false; clock = nil }

    private func readClock(zone: String) async throws -> ClockReading {
        let seconds = Date().timeIntervalSince1970.rounded(.down)
        guard seconds.isFinite, seconds >= Double(Int64.min), seconds < Double(Int64.max) else {
            throw ClientError.message("The device clock cannot be read. Select a date manually.")
        }
        let instant = Int64(seconds)
        let data = try JSONEncoder().encode(ClockRequest(zone_id: zone, utc_unix_seconds: instant))
        let response: ClockResponse = try await gateway.run(.clock, request: data)
        guard response.schema == "salah-local-clock-response-v1", response.status == "ok", let reading = response.clock else {
            throw ClientError.message(response.error?.message ?? "The device instant could not be converted. Select a date manually.")
        }
        guard reading.zone_id == zone, reading.utc_unix_seconds == instant,
              reading.source == "caller_clock_unverified" else { throw ClientError.invalidDocument }
        return reading
    }

    func refreshToday() {
        guard useToday, zoneConfirmed, inventory != nil else { return }
        clockTask?.cancel()
        let requestedRevision = revision
        let requestedZone = zoneID
        clockTask = Task { [weak self] in
            guard let self else { return }
            do {
                let reading = try await readClock(zone: requestedZone)
                guard !Task.isCancelled, requestedRevision == revision else { return }
                if let previous = clock, previous.local_date != reading.local_date { schedule = nil }
                clock = reading
            } catch {
                guard !Task.isCancelled, requestedRevision == revision else { return }
                clock = nil
                errorMessage = error.localizedDescription
            }
        }
    }

    func calculate() {
        guard canCalculate, let (a, b) = coordinates else { return }
        changed()
        cancelLookup()
        let requestedRevision = revision
        let zone = zoneID, method = methodID, criterion = asr
        let today = useToday, pickedDate = Self.dateText(manualDate)
        let origin = candidates?.zone_ids.contains(zone) == true ? "confirmed_suggestion" : "manual"
        isCalculating = true
        calculationTask = Task { [weak self] in
            guard let self else { return }
            do {
                let reading = today ? try await readClock(zone: zone) : nil
                guard !Task.isCancelled, requestedRevision == revision else { return }
                let selectedDate = reading?.local_date ?? pickedDate
                clock = reading
                let input = ScheduleRequest(latitude_degrees: a, longitude_degrees: b,
                                            local_date: selectedDate, zone_id: zone, method_id: method,
                                            asr: criterion, zone_choice: origin)
                let response: ScheduleResponse = try await gateway.run(.calculate, request: JSONEncoder().encode(input))
                guard !Task.isCancelled, requestedRevision == revision else { return }
                guard response.schema == "salah-schedule-response-v1", response.status == "ok", let document = response.schedule else {
                    throw ClientError.message(response.error?.message ?? "The schedule was not calculated.")
                }
                try document.validate()
                guard document.requested_local_date == selectedDate, document.zone_id == zone,
                      document.method.id == method, document.method.asr == criterion else { throw ClientError.invalidDocument }
                schedule = document
                isCalculating = false
            } catch {
                guard !Task.isCancelled, requestedRevision == revision else { return }
                schedule = nil
                isCalculating = false
                errorMessage = error.localizedDescription
            }
        }
    }

    func requestLocation(precise: Bool = false) {
        cancelLocation()
        changed()
        isLocating = true
        locationMessage = "Waiting for one Core Location system estimate (up to 30 seconds)…"
        location.request(precise: precise) { [weak self] result in
            guard let self else { return }
            isLocating = false
            switch result {
            case .success(let fix):
                latitude = String(fix.latitude)
                longitude = String(fix.longitude)
                deviceFix = fix
                coordinatesChanged(manual: false)
                locationMessage = "Core Location estimate; reported accuracy \(fix.horizontalAccuracyMeters.formatted(.number.precision(.fractionLength(0)))) m. Its physical source is unknown; offline acquisition depends on the device."
            case .failure(let error): locationMessage = error.localizedDescription; refreshToday()
            }
        }
    }

    func cancelLocation() { location.cancel(); isLocating = false }

    func inactive(background: Bool) {
        // Apple's permission prompt can make a scene inactive. There is no
        // sensor acquisition yet while authorization is undecided; keep that
        // prompt usable, but always cancel after actual background entry.
        if !background && location.isAwaitingPermission { location.pauseForPermissionPrompt(); return }
        cancelLocation()
        cancelLookup()
        revision &+= 1
        calculationTask?.cancel()
        calculationTask = nil
        clockTask?.cancel()
        clockTask = nil
        isCalculating = false
        locationMessage = deviceFix == nil ? "Acquisition stopped while this app was inactive. Manual coordinates remain available." : "Using the previous device estimate. Request a new fix if you have moved."
    }

    func active() { location.resume(); refreshToday() }

    func loadExample() {
        cancelLocation()
        changed()
        cancelLookup()
        deviceFix = nil
        candidates = nil
        latitude = "44.9778"
        longitude = "-93.2650"
        zoneID = "America/Chicago"
        zoneConfirmed = true
        manualDate = Self.date(year: 2026, month: 10, day: 1)
        useToday = false
        clock = nil
        methodID = "mwl-angles-18-17"
        asr = "hanafi"
        exampleLoaded = true
        zoneMessage = "Zone explicitly chosen by this labeled example."
        locationMessage = "Minneapolis research example coordinates. Choose your own location for personal use."
    }
}
