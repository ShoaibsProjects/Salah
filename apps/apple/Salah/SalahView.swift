import SwiftUI

struct SalahView: View {
    @StateObject private var model = SetupModel()
    @Environment(\.scenePhase) private var scenePhase
    @State private var showZones = false
    @State private var showCities = false
    @State private var showSavePlace = false
    @State private var clearPlaces = false

    private let accent = Color(red: 0.16, green: 0.39, blue: 0.31)

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    VStack(alignment: .leading, spacing: 9) {
                        Text("A quieter moment in your day.")
                            .font(.title2.weight(.semibold))
                        Text("Prayer calculations stay on your device. Installed data works without an internet connection.")
                            .foregroundStyle(.secondary)
                        Label(model.inventory == nil ? "Preparing offline calculation…" : "Offline calculation ready", systemImage: "leaf")
                            .font(.subheadline)
                            .foregroundStyle(accent)
                    }.padding(.vertical, 6)
                }

                locationSection
                savedPlacesSection
                timezoneSection
                dateSection
                practiceSection

                Section {
                    Button(action: model.calculate) {
                        HStack {
                            Spacer()
                            if model.isCalculating { ProgressView().padding(.trailing, 5) }
                            Text(model.isCalculating ? "Calculating on your device…" : "Calculate my schedule")
                                .font(.headline)
                            Spacer()
                        }.padding(.vertical, 6)
                    }.disabled(!model.canCalculate)
                    if !model.canCalculate && !model.isCalculating {
                        Text("Choose a city, saved place or device location; confirm the timezone and choose your calculation and Asr practice.")
                            .font(.footnote).foregroundStyle(.secondary)
                    }
                    if let error = model.errorMessage {
                        Label(error, systemImage: "exclamationmark.circle")
                            .foregroundStyle(.red)
                            .accessibilityLabel("Problem: \(error)")
                        if model.inventory == nil { Button("Retry engine startup", action: model.start) }
                    }
                }

                if let schedule = model.schedule { scheduleSection(schedule) }

                Section {
                    Button { showSavePlace = true } label: {
                        Label("Save this place for offline use", systemImage: "bookmark")
                    }.disabled(!model.canSavePlace)
                    Text("Calculate once, then save the place and your choices for easy use next time.")
                        .font(.footnote).foregroundStyle(.secondary)
                }

                Section {
                    Text("Research preview. Independent astronomy and Islamic-methodology review remain open. These are calculated beginnings under selected rules; a mosque's timetable and iqamah are separate.")
                        .font(.footnote).foregroundStyle(.secondary)
                    DisclosureGroup("Try a labeled research example") {
                        Text("Minneapolis · 1 October 2026 · America/Chicago · published MWL 18°/17° angles · Hanafi. This is an example, not a recommendation for your location.")
                            .font(.footnote)
                        Button("Fill the example", action: model.loadExample)
                    }
                }
            }
            .navigationTitle("Salah")
            .tint(accent)
            .sheet(isPresented: $showZones) {
                ZonePicker(zones: model.inventory?.zone_ids ?? [], selected: model.zoneID) {
                    model.chooseZone($0)
                    showZones = false
                }
            }
            .sheet(isPresented: $showCities) {
                CityPicker { choice in model.chooseCity(choice); showCities = false }
            }
            .sheet(isPresented: $showSavePlace) {
                SavePlaceForm(initialName: model.locationName.isEmpty ? "Home" : model.locationName) { name, startup in
                    model.savePlace(name: name, useOnStartup: startup)
                    showSavePlace = false
                }
            }
            .alert("Clear all saved places?", isPresented: $clearPlaces) {
                Button("Clear saved places", role: .destructive, action: model.clearSavedPlaces)
                Button("Cancel", role: .cancel) { }
            } message: { Text("This removes the saved copies and startup choice from this app. You can add places again later.") }
            .task { model.start() }
            .onChange(of: scenePhase) { _, phase in
                if phase == .active { model.active() }
                else { model.inactive(background: phase == .background) }
            }
        }
    }

    private var locationSection: some View {
        Section("Your location") {
            if model.isLocating {
                HStack { ProgressView(); Text("Waiting for a device estimate…") }
                Button("Cancel location request") { model.cancelLocation() }
            } else {
                Button { model.requestLocation() } label: {
                    Label("Use device location", systemImage: "location")
                }
                DisclosureGroup("If the first location request fails") {
                    Text("Try outdoors. A more precise request can wait up to 90 seconds and may ask for temporary precise permission. It can use more power; Apple chooses the location sources. You can cancel and choose a city offline instead.")
                        .font(.footnote).foregroundStyle(.secondary)
                    Button("Request a more precise estimate once") { model.requestLocation(precise: true) }
                }
            }
            Button { model.cancelLocation(); showCities = true } label: {
                Label("Choose a city offline", systemImage: "building.2")
            }
            DisclosureGroup("Enter or adjust coordinates") {
                TextField("Latitude, e.g. 44.9778", text: Binding(get: { model.latitude }, set: model.editLatitude))
                    .keyboardType(.numbersAndPunctuation).textInputAutocapitalization(.never).autocorrectionDisabled()
                    .accessibilityLabel("Latitude in degrees")
                TextField("Longitude, e.g. −93.2650", text: Binding(get: { model.longitude }, set: model.editLongitude))
                    .keyboardType(.numbersAndPunctuation).textInputAutocapitalization(.never).autocorrectionDisabled()
                    .accessibilityLabel("Longitude in degrees")
                Text("Use this if you already have coordinates, or to refine an approximate city point.")
                    .font(.footnote).foregroundStyle(.secondary)
            }
            Text(model.locationMessage).font(.footnote).foregroundStyle(.secondary)
            if let fix = model.deviceFix {
                VStack(alignment: .leading, spacing: 4) {
                    Text(fix.reducedAccuracy ? "Apple permission: approximate location" : "Apple permission: full accuracy")
                    Text("Measurement: \(fix.measuredAt.formatted(date: .abbreviated, time: .standard))")
                    if fix.simulated == true { Text("Apple marks this reading as simulated by software.") }
                    if fix.accessoryProduced == true { Text("Apple marks this reading as accessory-produced.") }
                }.font(.footnote).foregroundStyle(.secondary)
            }
        }
    }

    private var savedPlacesSection: some View {
        Section("Saved on this device") {
            ForEach(model.places.places) { place in
                Button { model.useSavedPlace(place) } label: {
                    HStack {
                        VStack(alignment: .leading) {
                            Text(place.name)
                            Text(place.zoneID + (model.places.startupID == place.id ? " · used on startup" : ""))
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        if model.selectedPlaceID == place.id { Image(systemName: "checkmark") }
                    }
                }.disabled(model.inventory == nil)
                .deleteDisabled(model.isSavingPlace)
            }.onDelete { offsets in model.removeSavedPlaces(Set(offsets.map { model.places.places[$0].id })) }
            if model.places.places.isEmpty {
                Text("A saved place works without a new location fix. Choose your location and calculate once, then save it.")
                    .font(.footnote).foregroundStyle(.secondary)
            }
            if let message = model.placeMessage { Text(message).font(.footnote) }
            if !model.places.places.isEmpty || model.placeMessage != nil {
                Button("Clear saved places…", role: .destructive) { clearPlaces = true }
                    .disabled(model.isSavingPlace)
            }
        }
    }

    private var timezoneSection: some View {
        Section("Location's timezone") {
            Button { showZones = true } label: {
                HStack {
                    Text(model.zoneID.isEmpty ? "Choose from supported zones" : model.zoneID)
                    Spacer()
                    Image(systemName: "chevron.up.chevron.down")
                }
            }.disabled(model.inventory == nil)
            if model.isLookingUp { ProgressView("Reading the offline map…") }
            Text(model.zoneMessage).font(.footnote).foregroundStyle(.secondary)
            Toggle("This timezone applies to my location", isOn: Binding(get: { model.zoneConfirmed }, set: model.confirmZone))
                .disabled(model.zoneID.isEmpty)
            if let inventory = model.inventory {
                DisclosureGroup("Installed timezone data") {
                    Text("IANA \(inventory.rule_pack.tzdb_version) · \(inventory.zone_ids.count) supported names. Future political changes require updated data; sensors cannot predict them.")
                    Text("Rule pack SHA-256: \(inventory.rule_pack.sha256)").textSelection(.enabled)
                    if let candidates = model.candidates {
                        Text("Boundary map: \(candidates.boundary_version). The complete location-error footprint is not checked.")
                    }
                }.font(.footnote)
            }
        }
    }

    private var dateSection: some View {
        Section("Gregorian date") {
            Toggle("Use today's date in this location", isOn: Binding(get: { model.useToday }, set: model.todayMode))
            if model.useToday {
                Text(model.clock?.local_date ?? "Confirm the timezone to derive today.")
                Text("Your device supplies a UTC clock instant. Rust derives the date under the selected timezone's installed rules. The device clock is not independently verified.")
                    .font(.footnote).foregroundStyle(.secondary)
            } else {
                DatePicker("Choose a date", selection: Binding(get: { model.manualDate }, set: model.chooseDate),
                           in: SetupModel.date(year: 1900, month: 1, day: 1)...SetupModel.date(year: 2100, month: 12, day: 31),
                           displayedComponents: [.date])
                    .environment(\.calendar, SetupModel.dateCalendar)
                    .environment(\.timeZone, TimeZone(secondsFromGMT: 0)!)
                Text("Manual date: \(SetupModel.dateText(model.manualDate)).")
                    .font(.footnote).foregroundStyle(.secondary)
            }
        }
    }

    private var practiceSection: some View {
        Section("Your calculation practice") {
            Picker("Calculation profile", selection: Binding(get: { model.methodID }, set: model.chooseMethod)) {
                Text("Please choose").tag("")
                Text("Published MWL angles · 18° / 17°").tag("mwl-angles-18-17")
                if model.methodID == "research-15" { Text("Engineering comparison · 15° / 15°").tag("research-15") }
            }
            Text(model.methodID == "research-15"
                 ? "This is an engineering comparison profile, not a community practice or recommendation."
                 : "Fajr and Isha use selected Sun angles below the horizon. This published 18°/17° parameter set is sourced to a secondary table; it is not an official League timetable or endorsement. Ask a trusted local mosque which practice it uses.")
                .font(.footnote).foregroundStyle(.secondary)
            Picker("Asr practice", selection: Binding(get: { model.asr }, set: model.chooseAsr)) {
                Text("Please choose").tag("")
                Text("Standard · one shadow length").tag("standard")
                Text("Hanafi · two shadow lengths").tag("hanafi")
            }
            Text("Both include the shadow already present at solar noon. Hanafi usually places Asr later. Choose the practice you follow; this app does not decide it from your country or location.")
                .font(.footnote).foregroundStyle(.secondary)
            DisclosureGroup("Engineering comparison") {
                Text("research-15 has no institutional attribution. It is for comparing the engine's 15° Fajr/Isha angle outputs, not for a recommended consumer timetable.")
                    .font(.footnote)
                Button("Select research-15 for technical comparison") { model.chooseMethod("research-15") }
            }
        }
    }

    @ViewBuilder private func scheduleSection(_ schedule: ScheduleDocument) -> some View {
        Section("\(schedule.requested_local_date) · \(schedule.zone_id)") {
            if schedule.civil_date.status == "skipped" {
                Text("This civil date did not exist in the selected timezone. No schedule was synthesized.")
            } else if schedule.cycle_match_status == "zero" {
                Text("The civil date exists, but no matching solar cycle was found. This differs from a skipped civil date.")
            } else {
                if schedule.cycle_match_status == "multiple" {
                    Text("Several solar cycles match this civil date. All are shown; the app has not selected one silently.")
                        .foregroundStyle(.secondary)
                }
                ForEach(Array(schedule.cycles.enumerated()), id: \.offset) { index, cycle in
                    if schedule.cycles.count > 1 { Text("Solar cycle \(index + 1)").font(.headline) }
                    ForEach(cycle.events) { event in
                        DisclosureGroup {
                            if let reading = event.reading {
                                Text("Event date: \(reading.local_date). UTC: \(reading.utc). Offset: \(reading.offset_seconds_east) seconds east of UTC.")
                                    .font(.footnote).textSelection(.enabled)
                                if let rule = event.rule { Text(rule.explanation).font(.footnote) }
                            } else {
                                Text("The selected solar condition does not occur in this cycle. No high-latitude alternative has been applied.")
                                    .font(.footnote)
                                Text("Engine reason: \(event.reason ?? "unavailable")").font(.caption)
                            }
                        } label: {
                            HStack {
                                Text(event.title)
                                Spacer()
                                Text(event.reading?.local_time ?? "Unavailable")
                                    .monospacedDigit()
                                    .foregroundStyle(event.reading == nil ? .secondary : .primary)
                            }
                        }
                    }
                }
            }
            DisclosureGroup("Calculation record") {
                Text("\(schedule.kernel.astronomy_model) · kernel \(schedule.kernel.version)")
                Text("\(schedule.method.id) revision \(schedule.method.revision) · Asr \(schedule.method.asr)")
                Text("Method source: \(schedule.method.source)")
                Text("IANA \(schedule.rule_pack.tzdb_version) · pack SHA-256 \(schedule.rule_pack.sha256)")
                    .textSelection(.enabled)
                Text("Second-precision research output. Local terrain/weather, approved high-latitude alternatives, notifications, and institutional certification are not supplied by this preview.")
            }.font(.footnote)
        }
    }
}

struct SavePlaceForm: View {
    let initialName: String
    let save: (String, Bool) -> Void
    @State private var name = ""
    @State private var startup = true
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                TextField("Place name, e.g. Home", text: $name)
                Toggle("Use this place when Salah opens", isOn: $startup)
                Text("Saves the chosen location, timezone and calculation practice on this device. It can be reused without internet or a fresh fix. Saved copies are excluded from backup; removing the app removes them. You can delete them here.")
                    .font(.footnote).foregroundStyle(.secondary)
                Button("Save place") { save(name, startup) }
                    .disabled(name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || name.count > 80)
            }
            .navigationTitle("Save for offline use")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } } }
            .onAppear { name = String(initialName.prefix(80)) }
        }
    }
}

struct ZonePicker: View {
    let zones: [String]
    let selected: String
    let choose: (String) -> Void
    @State private var search = ""
    @Environment(\.dismiss) private var dismiss

    private var filtered: [String] {
        search.isEmpty ? zones : zones.filter { $0.localizedCaseInsensitiveContains(search) }
    }

    var body: some View {
        NavigationStack {
            List(filtered, id: \.self) { zone in
                Button { choose(zone) } label: {
                    HStack { Text(zone); Spacer(); if zone == selected { Image(systemName: "checkmark") } }
                }
            }
            .searchable(text: $search, prompt: "Search a city or IANA zone")
            .overlay {
                if filtered.isEmpty { ContentUnavailableView.search(text: search) }
            }
            .navigationTitle("Supported timezones")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } } }
        }
    }
}
