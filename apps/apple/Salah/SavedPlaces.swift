import Foundation

struct PlaceSource: Codable, Sendable {
    let kind: String
    var geonameID: Int?
    var catalogueSHA: String?
    var horizontalAccuracyMeters: Double?
    var measuredAt: Date?
    var reducedAccuracy: Bool?
    var simulated: Bool?
    var accessoryProduced: Bool?

    func validate() throws {
        guard ["manual", "city", "device"].contains(kind) else { throw ClientError.invalidDocument }
        if kind == "city" {
            guard let geonameID, geonameID > 0, let catalogueSHA, catalogueSHA.count == 64 else { throw ClientError.invalidDocument }
        }
        if kind == "device" {
            guard let horizontalAccuracyMeters, horizontalAccuracyMeters.isFinite, horizontalAccuracyMeters >= 0,
                  let measuredAt, measuredAt.timeIntervalSince1970.isFinite else { throw ClientError.invalidDocument }
        }
    }
}

struct SavedPlace: Codable, Identifiable, Sendable {
    let id: UUID
    let name: String
    let latitude: Double
    let longitude: Double
    let zoneID: String
    let methodID: String
    let methodRevision: String
    let asr: String
    let rulePackSHA: String
    let boundarySHA: String?
    let source: PlaceSource

    func validate() throws {
        guard !name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty, name.count <= 80,
              latitude.isFinite, longitude.isFinite, (-90...90).contains(latitude), (-180...180).contains(longitude),
              !zoneID.isEmpty, zoneID.count <= 128,
              ["research-15", "mwl-angles-18-17"].contains(methodID),
              !methodRevision.isEmpty, methodRevision.count <= 32, ["standard", "hanafi"].contains(asr),
              rulePackSHA.count == 64, boundarySHA == nil || boundarySHA?.count == 64 else { throw ClientError.invalidDocument }
        try source.validate()
    }
}

struct SavedPlacesDocument: Codable, Sendable {
    var schema = "salah-saved-places-v1"
    var places: [SavedPlace] = []
    var startupID: UUID?

    func validate() throws {
        guard schema == "salah-saved-places-v1", places.count <= 20,
              Set(places.map(\.id)).count == places.count,
              startupID == nil || places.contains(where: { $0.id == startupID }) else { throw ClientError.invalidDocument }
        try places.forEach { try $0.validate() }
    }
}

// One app-private repository. Personal data is opt-in, bounded, atomically
// replaced with iOS complete protection, and excluded from backups.
actor SavedPlacesRepository {
    static let shared = SavedPlacesRepository()
    private let directory: URL?
    init(directory: URL? = nil) { self.directory = directory }

    private func root() throws -> URL {
        if let directory { return directory }
        guard let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else { throw ClientError.message("Private saved-place storage is unavailable.") }
        return support.appendingPathComponent("SalahPlaces", isDirectory: true)
    }

    private func checkType(_ path: URL, expected: FileAttributeType) throws {
        let values = try FileManager.default.attributesOfItem(atPath: path.path)
        guard values[.type] as? FileAttributeType == expected else { throw ClientError.message("Saved-place storage has an unexpected file type.") }
    }

    func load() throws -> SavedPlacesDocument {
        let root = try root()
        do { try checkType(root, expected: .typeDirectory) }
        catch let error as CocoaError where error.code == .fileReadNoSuchFile { return SavedPlacesDocument() }
        let fileURL = root.appendingPathComponent("places-v1.json")
        do { try checkType(fileURL, expected: .typeRegular) }
        catch let error as CocoaError where error.code == .fileReadNoSuchFile { return SavedPlacesDocument() }
        let file = try FileHandle(forReadingFrom: fileURL)
        defer { try? file.close() }
        let bytes = try file.read(upToCount: 65_537) ?? Data()
        guard !bytes.isEmpty, bytes.count <= 65_536 else { throw ClientError.invalidDocument }
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let document = try decoder.decode(SavedPlacesDocument.self, from: bytes)
        try document.validate()
        return document
    }

    private func write(_ document: SavedPlacesDocument) throws {
        try document.validate()
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.outputFormatting = [.sortedKeys]
        let bytes = try encoder.encode(document)
        guard bytes.count <= 65_536 else { throw ClientError.message("Saved places exceed this app's storage limit.") }
        var root = try root()
        #if os(iOS)
        let attributes: [FileAttributeKey: Any] = [.protectionKey: FileProtectionType.complete, .posixPermissions: 0o700]
        #else
        let attributes: [FileAttributeKey: Any] = [.posixPermissions: 0o700] // Developer host check only.
        #endif
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true, attributes: attributes)
        try checkType(root, expected: .typeDirectory)
        try FileManager.default.setAttributes(attributes, ofItemAtPath: root.path)
        var exclusion = URLResourceValues()
        exclusion.isExcludedFromBackup = true
        try root.setResourceValues(exclusion)
        guard try root.resourceValues(forKeys: [.isExcludedFromBackupKey]).isExcludedFromBackup == true else { throw ClientError.message("The OS could not exclude saved places from backup.") }
        let fileURL = root.appendingPathComponent("places-v1.json")
        #if os(iOS)
        try bytes.write(to: fileURL, options: [.atomic, .completeFileProtection])
        #else
        try bytes.write(to: fileURL, options: [.atomic])
        #endif
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: fileURL.path)
        #if os(iOS) && !targetEnvironment(simulator)
        let fileAttributes = try FileManager.default.attributesOfItem(atPath: fileURL.path)
        let rawProtection = fileAttributes[.protectionKey]
        let protection = rawProtection as? FileProtectionType ?? (rawProtection as? String).map(FileProtectionType.init(rawValue:))
        guard protection == .complete else { throw ClientError.message("The OS could not confirm saved-place protection.") }
        #endif
    }

    func save(_ place: SavedPlace, useOnStartup: Bool) throws -> SavedPlacesDocument {
        var document = try load() // Invalid existing data is never silently replaced.
        guard document.places.count < 20 else { throw ClientError.message("You can save up to 20 places. Remove one before adding another.") }
        document.places.append(place)
        if useOnStartup { document.startupID = place.id }
        try write(document)
        return document
    }

    func remove(_ ids: Set<UUID>) throws -> SavedPlacesDocument {
        var document = try load()
        document.places.removeAll { ids.contains($0.id) }
        if let startupID = document.startupID, ids.contains(startupID) { document.startupID = nil }
        try write(document)
        return document
    }

    func clear() throws {
        let root = try root()
        do { try checkType(root, expected: .typeDirectory) }
        catch let error as CocoaError where error.code == .fileReadNoSuchFile { return }
        let path = root.appendingPathComponent("places-v1.json")
        do { try checkType(path, expected: .typeRegular) }
        catch let error as CocoaError where error.code == .fileReadNoSuchFile { return }
        do { try FileManager.default.removeItem(at: path) }
        catch let error as CocoaError where error.code == .fileNoSuchFile || error.code == .fileReadNoSuchFile { return }
    }
}
