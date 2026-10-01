// Device readings are inputs. Rust owns timezone lookup and civil conversion.
export class OfflineSetup {
  constructor(form, request, invalidate) {
    this.form = form;
    this.request = request;
    this.invalidate = invalidate;
    this.zone = form.elements.namedItem("zone_id");
    this.date = form.elements.namedItem("local_date");
    this.confirm = document.querySelector("#zone-confirmed");
    this.todayMode = document.querySelector("#device-today");
    this.zoneHelp = document.querySelector("#zone-help");
    this.dateHelp = document.querySelector("#date-help");
    this.revision = 0;
    this.lookupWhenReady = false;
    this.candidates = null;
    this.dateCapture = { source: "not_selected" };
    for (const name of ["latitude_degrees", "longitude_degrees"]) {
      form.elements.namedItem(name).addEventListener("input", () => this.coordinatesChanged());
    }
    this.zone.addEventListener("change", () => {
      clearTimeout(this.lookupTimer);
      this.lookupWhenReady = false;
      this.revision++;
      this.confirm.checked = false;
      this.clearToday();
      this.zoneHelp.textContent = "Timezone edited. Confirm that it applies to your selected location; manual correction is always available.";
    });
    this.confirm.addEventListener("change", () => {
      clearTimeout(this.lookupTimer);
      this.revision++;
      this.invalidate();
      if (this.confirm.checked) this.previewToday(); else this.clearToday();
    });
    this.todayMode.addEventListener("change", () => {
      this.revision++;
      this.invalidate();
      if (this.todayMode.checked) this.previewToday(); else this.manualDate();
    });
    this.date.addEventListener("input", () => {
      if (this.todayMode.checked) {
        this.todayMode.checked = false;
        this.revision++;
        this.invalidate();
      }
      this.manualDate();
    });
    document.addEventListener("visibilitychange", () => {
      if (!document.hidden && this.todayMode.checked && this.confirm.checked) this.previewToday();
    });
  }

  coordinates() {
    const a = this.form.elements.namedItem("latitude_degrees").value;
    const b = this.form.elements.namedItem("longitude_degrees").value;
    const latitude = Number(a), longitude = Number(b);
    if (!a || !b || !Number.isFinite(latitude) || !Number.isFinite(longitude)
        || latitude < -90 || latitude > 90 || longitude < -180 || longitude > 180) return null;
    return { latitude_degrees: latitude, longitude_degrees: longitude };
  }

  clearToday() {
    if (!this.todayMode.checked) return;
    this.date.value = "";
    this.dateCapture = { source: "not_selected" };
    this.dateHelp.textContent = "Confirm the location's timezone first. Rust will then derive its date from your device's UTC clock.";
  }

  coordinatesChanged() {
    this.revision++;
    this.lookupWhenReady = true;
    this.candidates = null;
    this.confirm.checked = false;
    this.clearToday();
    clearTimeout(this.lookupTimer);
    this.zoneHelp.textContent = "Checking the embedded timezone map after you finish entering coordinates…";
    this.lookupTimer = setTimeout(() => this.lookup(), 400);
  }

  engineReady() {
    if (!this.lookupWhenReady) return;
    clearTimeout(this.lookupTimer);
    this.lookupWhenReady = false;
    this.lookup();
  }

  async lookup() {
    const point = this.coordinates();
    if (!point) {
      this.zoneHelp.textContent = "Enter valid coordinates or use an available device fix. Timezone suggestions are calculated locally.";
      return;
    }
    const revision = this.revision;
    try {
      const response = await this.request("lookup", { schema: "salah-zone-lookup-request-v1", ...point });
      if (revision !== this.revision) return;
      if (response.schema !== "salah-zone-lookup-response-v1" || response.status !== "ok") {
        throw new Error(response.error?.message ?? "The timezone map could not be read.");
      }
      const candidates = response.candidates;
      if (!Array.isArray(candidates?.zone_ids) || candidates.latitude_degrees !== point.latitude_degrees
          || candidates.longitude_degrees !== point.longitude_degrees || !candidates.requires_confirmation) {
        throw new Error("The engine returned an unsupported timezone suggestion.");
      }
      this.candidates = candidates;
      this.lookupWhenReady = false;
      this.confirm.checked = false;
      if (candidates.zone_ids.length === 1) {
        this.zone.value = candidates.zone_ids[0];
        this.zoneHelp.textContent = `Filled from the offline ${candidates.boundary_version} map. Confirm this zone below; approximate borders and location error can change the result.`;
      } else if (candidates.zone_ids.length > 1) {
        this.zone.value = "";
        this.zoneHelp.textContent = `The offline map returns several possible zones: ${candidates.zone_ids.join(", ")}. Choose and confirm one; the app will not choose the first.`;
      } else {
        this.zone.value = "";
        this.zoneHelp.textContent = "The offline map has no zone for this point. Select and confirm a timezone yourself. Ship and aircraft clocks may follow the operator's choice.";
      }
      this.clearToday();
    } catch (error) {
      if (revision !== this.revision) return;
      this.zoneHelp.textContent = `${error.message} Select a supported timezone manually and confirm it.`;
    }
  }

  manualDate() {
    this.dateCapture = { source: "manual_date", local_date: this.date.value };
    this.dateHelp.textContent = "Using your selected Gregorian date. You can return to today's date from the device clock at any time.";
  }

  applyClock(response, instant, zone, invalidateDate) {
    if (response.schema !== "salah-local-clock-response-v1" || response.status !== "ok") {
      throw new Error(response.error?.message ?? "The device date could not be converted.");
    }
    const clock = response.clock;
    if (clock?.zone_id !== zone || clock.utc_unix_seconds !== instant || !/^\d{4}-\d{2}-\d{2}$/.test(clock.local_date)) {
      throw new Error("The engine returned an unsupported clock reading.");
    }
    const changedDay = this.date.value && this.date.value !== clock.local_date;
    if (changedDay && invalidateDate) this.invalidate();
    this.date.value = clock.local_date;
    this.dateCapture = { source: "device_clock_in_selected_zone", clock };
    this.dateHelp.textContent = `Today is ${clock.local_date} in ${zone}, using IANA ${clock.rule_pack.tzdb_version}. Your device clock supplies the instant; check that its clock is correct.`;
    return clock.local_date;
  }

  async readToday(invalidateDate = true) {
    if (!this.confirm.checked || !this.zone.value) throw new Error("Confirm the timezone for this location first.");
    const revision = this.revision;
    const zone = this.zone.value;
    const instant = Math.floor(Date.now() / 1000); // UTC epoch only; no host timezone conversion.
    if (!Number.isSafeInteger(instant)) throw new Error("The device clock could not be read. Choose a date manually.");
    const response = await this.request("clock", { schema: "salah-local-clock-request-v1", zone_id: zone, utc_unix_seconds: instant });
    if (revision !== this.revision) throw new Error("Settings changed while reading the clock. Calculate again.");
    return this.applyClock(response, instant, zone, invalidateDate);
  }

  async previewToday() {
    if (!this.todayMode.checked || !this.confirm.checked) return this.clearToday();
    const revision = this.revision;
    try { await this.readToday(); }
    catch (error) {
      if (revision !== this.revision) return;
      this.date.value = "";
      this.dateCapture = { source: "not_selected" };
      this.dateHelp.textContent = `${error.message} You can turn off today's date and enter one manually.`;
    }
  }

  async prepare() {
    if (!this.confirm.checked || !this.zone.value) throw new Error("Confirm that the timezone applies to your selected location.");
    this.revision++; // Ignore a previous preview reading when starting a calculation.
    this.lookupWhenReady = false;
    clearTimeout(this.lookupTimer);
    const localDate = this.todayMode.checked ? await this.readToday(false) : this.date.value;
    if (!localDate) throw new Error("Choose a valid date, or use today's date after confirming the timezone.");
    if (!this.todayMode.checked) this.manualDate();
    return { local_date: localDate, zone_choice: this.candidates?.zone_ids.includes(this.zone.value) ? "confirmed_suggestion" : "manual" };
  }

  useExample() {
    this.revision++;
    this.lookupWhenReady = false;
    clearTimeout(this.lookupTimer);
    this.candidates = null;
    this.confirm.checked = true;
    this.todayMode.checked = false;
    this.dateCapture = { source: "example_date", local_date: this.date.value };
    this.dateHelp.textContent = "Using the labeled research example date. Enable today's date to use your device clock.";
    this.zoneHelp.textContent = "Timezone explicitly selected by the labeled research example. This is not a regional method recommendation.";
  }

  metadata() { return { zone_lookup: this.candidates, date_input: this.dateCapture }; }
}
