import CryptoKit
import Foundation

struct OfflineCity: Decodable, Sendable, Identifiable {
    let id: Int
    let name: String
    let asciiName: String
    let countryCode: String
    let latitude: Double
    let longitude: Double
    let population: Int
    let aliases: [String]

    init(from decoder: Decoder) throws {
        var row = try decoder.unkeyedContainer()
        id = try row.decode(Int.self)
        name = try row.decode(String.self)
        asciiName = try row.decode(String.self)
        countryCode = try row.decode(String.self)
        latitude = try row.decode(Double.self)
        longitude = try row.decode(Double.self)
        population = try row.decode(Int.self)
        aliases = try row.decode([String].self)
        guard row.isAtEnd, id > 0, !name.isEmpty, name.count <= 300, asciiName.count <= 300,
              countryCode.count == 2, latitude.isFinite, longitude.isFinite,
              (-90...90).contains(latitude), (-180...180).contains(longitude),
              population >= 0, aliases.count <= 24, aliases.allSatisfy({ $0.count <= 80 }) else {
            throw ClientError.invalidDocument
        }
    }
}

struct CityChoice: Identifiable, Sendable {
    let city: OfflineCity
    let country: String
    let catalogueSHA: String
    var id: Int { city.id }
}

actor CityDirectory {
    static let shared = CityDirectory()
    private struct Catalogue: Decodable {
        let schema: String
        let version: String
        let countries: [String: String]
        let cities: [OfflineCity]
    }
    private struct Manifest: Decodable {
        let schema: String
        let version: String
        let bytes: Int
        let rows: Int
        let sha256: String
    }
    private struct Entry {
        let choice: CityChoice
        let names: String
        let country: String
    }
    private var entries: [Entry]?

    private static func normalized(_ text: String) -> String {
        text.folding(options: [.diacriticInsensitive, .caseInsensitive], locale: Locale(identifier: "en_US_POSIX"))
    }

    private func load(directory: URL?) throws -> [Entry] {
        if let entries { return entries }
        let dataURL = directory?.appendingPathComponent("cities-v1.json") ?? Bundle.main.url(forResource: "cities-v1", withExtension: "json")
        let manifestURL = directory?.appendingPathComponent("cities-manifest-v1.json") ?? Bundle.main.url(forResource: "cities-manifest-v1", withExtension: "json")
        guard let dataURL, let manifestURL else { throw ClientError.message("The offline city directory is missing from this installation.") }
        let manifestBytes = try Data(contentsOf: manifestURL)
        guard manifestBytes.count <= 16_384 else { throw ClientError.invalidDocument }
        let manifest = try JSONDecoder().decode(Manifest.self, from: manifestBytes)
        guard manifest.schema == "salah-offline-cities-manifest-v1", manifest.bytes > 0,
              manifest.bytes <= 8_000_000, (20_000...50_000).contains(manifest.rows) else { throw ClientError.invalidDocument }
        let file = try FileHandle(forReadingFrom: dataURL)
        defer { try? file.close() }
        let bytes = try file.read(upToCount: manifest.bytes + 1) ?? Data()
        guard bytes.count == manifest.bytes,
              SHA256.hash(data: bytes).map({ String(format: "%02x", $0) }).joined() == manifest.sha256 else { throw ClientError.invalidDocument }
        let catalogue = try JSONDecoder().decode(Catalogue.self, from: bytes)
        guard catalogue.schema == "salah-offline-cities-v1", catalogue.version == manifest.version,
              catalogue.cities.count == manifest.rows, Set(catalogue.cities.map(\.id)).count == manifest.rows else { throw ClientError.invalidDocument }
        let result = try catalogue.cities.map { city in
            guard let country = catalogue.countries[city.countryCode] else { throw ClientError.invalidDocument }
            let names = ([city.name, city.asciiName] + city.aliases).joined(separator: "\n")
            return Entry(choice: CityChoice(city: city, country: country, catalogueSHA: manifest.sha256),
                         names: Self.normalized(names), country: Self.normalized(country + " " + city.countryCode))
        }
        entries = result
        return result
    }

    func search(_ query: String, directory: URL? = nil) throws -> [CityChoice] {
        let terms = Self.normalized(query.trimmingCharacters(in: .whitespacesAndNewlines)).split(whereSeparator: \.isWhitespace).map(String.init)
        guard !terms.isEmpty, query.count <= 160 else { return [] }
        let rows = try load(directory: directory)
        var result: [CityChoice] = []
        for entry in rows where terms.allSatisfy({ entry.names.contains($0) || entry.country.contains($0) }) {
            result.append(entry.choice)
            if result.count == 40 { break }
        }
        return result // Pinned source population order, never an inferred user location.
    }

    func unload() { entries = nil }
}
