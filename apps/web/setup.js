// Device readings are inputs. Rust owns timezone lookup and civil conversion.
export class OfflineSetup {
  constructor(form, request, invalidate, zoneStatus = () => {}) {
    this.form = form;
    this.request = request;
    this.invalidate = invalidate;
    this.zoneStatus = zoneStatus;
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
      this.zoneStatus({ ready: false });
    });
    this.confirm.addEventListener("change", () => {
      clearTimeout(this.lookupTimer);
      this.revision++;
      this.invalidate();
      if (this.confirm.checked) this.previewToday().then(() => this.zoneStatus({ ready: !!this.date.value, zone: this.zone.value }));
      else { this.clearToday(); this.zoneStatus({ ready: false }); }
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
    this.dateHelp.textContent = "Today’s date appears after choosing a city.";
  }

  coordinatesChanged({ immediate = false } = {}) {
    this.revision++;
    this.lookupWhenReady = true;
    this.candidates = null;
    this.confirm.checked = false;
    this.clearToday();
    clearTimeout(this.lookupTimer);
    this.zoneHelp.textContent = "Finding the time zone for this location…";
    this.zoneStatus({ ready: false });
    this.lookupTimer = setTimeout(() => this.lookup(), immediate ? 0 : 400);
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
      if (candidates.zone_ids.length === 1) {
        this.zone.value = candidates.zone_ids[0];
        this.confirm.checked = true;
        this.zoneHelp.textContent = "Time zone selected for this city. Change it below if you know your local time zone is different.";
        await this.previewToday();
        this.zoneStatus({ ready: !!this.date.value, zone: this.zone.value });
      } else if (candidates.zone_ids.length > 1) {
        this.zone.value = "";
        this.confirm.checked = false;
        this.clearToday();
        this.zoneHelp.textContent = `Choose the time zone for this location: ${candidates.zone_ids.join(", ")}.`;
        this.zoneStatus({ ready: false, needsChoice: true });
      } else {
        this.zone.value = "";
        this.confirm.checked = false;
        this.clearToday();
        this.zoneHelp.textContent = "Choose the time zone for this location.";
        this.zoneStatus({ ready: false, needsChoice: true });
      }
    } catch (error) {
      if (revision !== this.revision) return;
      this.zoneHelp.textContent = `${error.message} Choose a time zone in More settings.`;
      this.zoneStatus({ ready: false, needsChoice: true });
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
    this.dateHelp.textContent = `Today is ${clock.local_date} in your selected city.`;
    return clock.local_date;
  }

  async readToday(invalidateDate = true) {
    if (!this.confirm.checked || !this.zone.value) throw new Error("Choose the time zone for this location in More settings.");
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
      this.dateHelp.textContent = `${error.message} Choose another time zone or date.`;
    }
  }

  async prepare() {
    if (!this.confirm.checked || !this.zone.value) throw new Error("Choose the time zone for this location in More settings.");
    this.revision++; // Ignore a previous preview reading when starting a calculation.
    this.lookupWhenReady = false;
    clearTimeout(this.lookupTimer);
    const localDate = this.todayMode.checked ? await this.readToday(false) : this.date.value;
    if (!localDate) throw new Error("Choose a valid date, or use today's date after confirming the timezone.");
    if (!this.todayMode.checked) this.manualDate();
    return { local_date: localDate, zone_choice: this.candidates?.zone_ids.includes(this.zone.value) ? "confirmed_suggestion" : "manual" };
  }

  metadata() { return { zone_lookup: this.candidates, date_input: this.dateCapture }; }
}
