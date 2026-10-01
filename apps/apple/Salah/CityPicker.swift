import SwiftUI

struct CityPicker: View {
    let choose: (CityChoice) -> Void
    @State private var query = ""
    @State private var results: [CityChoice] = []
    @State private var error: String?
    @State private var loading = false
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Section {
                    Text("Search works without internet. A city uses its approximate reference point, which may differ from your exact location. Choose your city or a nearby town, then confirm the timezone.")
                        .font(.footnote).foregroundStyle(.secondary)
                }
                if loading { ProgressView("Searching installed city data…") }
                if let error { Text(error).foregroundStyle(.red) }
                ForEach(results) { choice in
                    Button { choose(choice) } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(choice.city.name)
                            Text("\(choice.country) · \(choice.city.countryCode)").font(.subheadline).foregroundStyle(.secondary)
                            Text("Approximate city point: \(choice.city.latitude.formatted())°, \(choice.city.longitude.formatted())°")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
                if !loading && !query.isEmpty && results.isEmpty && error == nil {
                    Text("No matching city in the installed directory. Try another spelling or a nearby town; device location and manual entry remain available.")
                }
                Section {
                    Text("City points and country names © GeoNames, CC BY 4.0. This snapshot includes larger towns and capitals, with a bounded set of source aliases. It does not contain every village or street.")
                        .font(.footnote).foregroundStyle(.secondary)
                    Link("GeoNames attribution and license", destination: URL(string: "https://www.geonames.org/about.html")!)
                }
            }
            .navigationTitle("Choose a city offline")
            .navigationBarTitleDisplayMode(.inline)
            .searchable(text: $query, prompt: "City, country, or country code")
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } } }
            .onDisappear { Task { await CityDirectory.shared.unload() } }
            .task(id: query) {
                error = nil
                results = []
                loading = !query.isEmpty
                do {
                    try await Task.sleep(for: .milliseconds(250))
                    let found = try await CityDirectory.shared.search(query)
                    guard !Task.isCancelled else { return }
                    results = found
                    loading = false
                } catch {
                    guard !Task.isCancelled else { return }
                    self.error = error.localizedDescription
                    loading = false
                }
            }
        }
    }
}
