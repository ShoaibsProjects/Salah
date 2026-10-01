// Developer-only acceptance executable. No location request or network call.
import Foundation

@main
struct ApplePlaceCheck {
    @MainActor static func wait(_ label: String, until condition: () -> Bool) async throws {
        for _ in 0..<240 {
            if condition() { return }
            try await Task.sleep(for: .milliseconds(25))
        }
        throw ClientError.message("Acceptance timed out: " + label)
    }

    static func assert(_ condition: Bool, _ label: String) throws {
        if !condition { throw ClientError.message("Acceptance failed: " + label) }
    }

    @MainActor static func main() async throws {
        guard CommandLine.arguments.count == 3 else { throw ClientError.message("Expected catalogue directory and disposable storage directory") }
        let assets = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
        let storage = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
        let directory = CityDirectory()
        let start = ContinuousClock.now
        let minneapolis = try await directory.search("Minneapolis US", directory: assets)
        let elapsed = start.duration(to: .now)
        guard let city = minneapolis.first else { throw ClientError.message("Minneapolis city not found") }
        try assert(city.city.countryCode == "US", "country matching")
        let makkah = try await directory.search("Makkah", directory: assets)
        try assert(makkah.contains { $0.city.countryCode == "SA" }, "source alias Makkah")
        let accent = try await directory.search("sao paulo brazil", directory: assets)
        try assert(accent.contains { $0.city.name == "São Paulo" }, "accent/country search")
        let paris = try await directory.search("Paris France", directory: assets)
        try assert(paris.allSatisfy { $0.city.countryCode == "FR" } && !paris.isEmpty, "same-name country distinction")
        print("Offline city search: countries, aliases, diacritics; first load+search \(elapsed)")
        await directory.unload()

        let repository = SavedPlacesRepository(directory: storage)
        let model = SetupModel(placeRepository: repository)
        model.start()
        try await wait("bundled inventory") { model.inventory != nil }
        model.chooseCity(city)
        model.chooseMethod("mwl-angles-18-17")
        model.chooseAsr("hanafi")
        try await wait("city map") { !model.isLookingUp && !model.zoneID.isEmpty }
        try assert(model.zoneID == "America/Chicago" && !model.zoneConfirmed, "city suggestion still needs confirmation")
        model.confirmZone(true)
        model.chooseDate(SetupModel.date(year: 2026, month: 10, day: 1))
        model.calculate()
        try await wait("city schedule") { !model.isCalculating }
        try assert(model.schedule != nil && model.errorMessage == nil, "city calculations")
        model.savePlace(name: "Acceptance Home", useOnStartup: true)
        try await wait("private save") { !model.isSavingPlace }
        let reopened = SavedPlacesRepository(directory: storage)
        let document = try await reopened.load()
        guard let place = document.places.first else { throw ClientError.message("Place was not persisted: \(model.placeMessage ?? "unknown")") }
        try assert(document.startupID == place.id && place.source.geonameID == city.city.id,
                   "startup choice and city provenance")
        try assert(place.latitude == city.city.latitude && place.longitude == city.city.longitude,
                   "persisted source coordinates")
        let restored = SetupModel(placeRepository: reopened)
        restored.start()
        try await wait("restored today schedule") { restored.schedule != nil || restored.errorMessage != nil }
        try assert(restored.schedule != nil && restored.useToday && restored.selectedPlaceID == place.id,
                   "offline restart retains practice/place and computes today's date")
        try assert(restored.clock?.source == "caller_clock_unverified" && restored.deviceFix == nil,
                   "saved place is not relabeled as fresh device fix")
        let edited = SetupModel(placeRepository: reopened)
        edited.start()
        edited.editLatitude("27.7172")
        edited.editLongitude("85.324")
        try await wait("manual edit during startup") { edited.inventory != nil && !edited.isLookingUp }
        try assert(edited.latitude == "27.7172" && edited.selectedPlaceID == nil, "startup cannot overwrite newer input")

        let changedRules = SavedPlace(id: UUID(), name: "Old rules", latitude: place.latitude,
            longitude: place.longitude, zoneID: place.zoneID, methodID: place.methodID,
            methodRevision: place.methodRevision, asr: place.asr, rulePackSHA: String(repeating: "0", count: 64),
            boundarySHA: place.boundarySHA, source: place.source)
        restored.useSavedPlace(changedRules)
        try assert(!restored.zoneConfirmed && restored.schedule == nil, "changed rules require confirmation")
        let changedMap = SavedPlace(id: UUID(), name: "Old map", latitude: place.latitude,
            longitude: place.longitude, zoneID: place.zoneID, methodID: place.methodID,
            methodRevision: place.methodRevision, asr: place.asr, rulePackSHA: place.rulePackSHA,
            boundarySHA: String(repeating: "0", count: 64), source: place.source)
        restored.useSavedPlace(changedMap)
        try await wait("changed map") { restored.zoneMessage.contains("changed or") }
        try assert(!restored.zoneConfirmed && restored.schedule == nil, "changed map requires confirmation")
        let changedMethod = SavedPlace(id: UUID(), name: "Old method", latitude: place.latitude,
            longitude: place.longitude, zoneID: place.zoneID, methodID: place.methodID,
            methodRevision: "unrecognized-revision", asr: place.asr, rulePackSHA: place.rulePackSHA,
            boundarySHA: nil, source: place.source)
        restored.useSavedPlace(changedMethod)
        try await wait("changed method") { restored.errorMessage != nil }
        try assert(restored.methodID.isEmpty && restored.schedule == nil, "changed method cannot reuse approval")
        print("Private save/reopen/startup, explicit provenance, edit race and rule/map/method invalidation passed.")

        let removed = try await reopened.remove([place.id])
        try assert(removed.places.isEmpty && removed.startupID == nil, "deletion removes startup reference")
        let file = storage.appendingPathComponent("places-v1.json")
        let corrupt = Data("{broken".utf8)
        try corrupt.write(to: file)
        do { _ = try await reopened.save(place, useOnStartup: false); throw ClientError.message("Corrupt store unexpectedly replaced") }
        catch let error as ClientError { throw error }
        catch { }
        try assert(try Data(contentsOf: file) == corrupt, "corrupt data preserved")
        try Data(repeating: 65, count: 65_537).write(to: file)
        do { _ = try await reopened.load(); throw ClientError.message("Oversized store accepted") }
        catch let error as ClientError {
            if case .invalidDocument = error { } else { throw error }
        }
        try await reopened.clear()
        let empty = try await reopened.load()
        try assert(empty.places.isEmpty, "explicit reset")
        print("Corrupt/oversized data fails safely; deletion and explicit reset passed. Simulator protection flags do not prove physical encryption or GNSS.")
    }
}
