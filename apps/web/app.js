import { OfflineSetup } from "./setup.js";

const form = document.querySelector("#schedule-form");
const calculateButton = document.querySelector("#calculate");
const deviceLocationButton = document.querySelector("#device-location");
const preciseLocationButton = document.querySelector("#precise-location");
const locationStatus = document.querySelector("#location-status");
const engineState = document.querySelector("#engine-state");
const errorBox = document.querySelector("#error");
const notice = document.querySelector("#notice");
const results = document.querySelector("#results");
const provenance = document.querySelector("#provenance");
let sequence = 0;
let activeId = null;
let lastDocument = null;
let lastSetupMetadata = null;
let pendingSetupMetadata = null;
const workerJobs = new Map();
let worker;
let watchdog;
let loaded = false;
let terminated = false;
let locationRequestId = 0;
let locationRequestTimer;
let locationCapture = { source: "manual_coordinates", reportedAccuracyMeters: null };

const methods = {
  "mwl-angles-18-17": "MWL means Muslim World League. This uses the published parameter set of 18° for Fajr and 17° for Isha. The angles describe how far the Sun is below the horizon in the calculation. Our source is a secondary table; it is not an official League timetable or endorsement. Your mosque may use different settings.",
  "research-15": "A technical comparison profile: it uses a 15° Sun angle for both Fajr and Isha. It is here to compare calculations, not as a community method or mosque recommendation.",
};

const setup = new OfflineSetup(form, sendSetupRequest, () => invalidateChoices());

function sendSetupRequest(kind, request) {
  if (!loaded) return Promise.reject(new Error("The local Rust engine is not ready yet."));
  return new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => {
      workerJobs.delete(id);
      reject(new Error("The offline setup operation took too long; no replacement was inferred."));
    }, 15000);
    workerJobs.set(id, { resolve, reject, timer });
    try { worker.postMessage({ kind, id, request: JSON.stringify(request) }); }
    catch (error) { clearTimeout(timer); workerJobs.delete(id); reject(error); }
  });
}

function clearResult() {
  lastDocument = null;
  lastSetupMetadata = null;
  results.replaceChildren();
  provenance.hidden = true;
  errorBox.hidden = true;
  notice.hidden = true;
  document.querySelector("#empty").hidden = false;
  document.querySelector("#schedule-context").textContent = "Your calculated schedule will appear here.";
}

function failure(message, fatal = false) {
  clearTimeout(watchdog);
  activeId = null;
  clearResult();
  errorBox.textContent = message;
  errorBox.hidden = false;
  if (fatal) {
    loaded = false; terminated = true; worker?.terminate();
    for (const job of workerJobs.values()) { clearTimeout(job.timer); job.reject(new Error(message)); }
    workerJobs.clear();
  }
  calculateButton.disabled = !loaded;
  calculateButton.removeAttribute("aria-busy");
  engineState.textContent = fatal ? "Engine unavailable. Reload after rebuilding the local artifact." : "Ready for another explicit calculation.";
}

function setLocationMessage(message, problem = false) {
  locationStatus.textContent = message;
  locationStatus.classList.toggle("problem", problem);
}

function cancelLocationRequest(message) {
  if (locationRequestId === 0) return;
  locationRequestId = 0;
  clearTimeout(locationRequestTimer);
  deviceLocationButton.disabled = false;
  preciseLocationButton.disabled = false;
  preciseLocationButton.hidden = true;
  deviceLocationButton.removeAttribute("aria-busy");
  setLocationMessage(message);
}

function explainMethod() {
  const selected = form.elements.namedItem("method_id").value;
  document.querySelector("#method-help").textContent = methods[selected]
    ?? "Choose the calculation convention used by your community. The Sun-angle choices give estimated Fajr and Isha beginnings; there is no universal selection here.";
}

function useDeviceLocation({ highAccuracy = false } = {}) {
  if (locationRequestId !== 0) return;
  if (!window.isSecureContext) {
    setLocationMessage("Device location is available on a secure page or localhost. You can enter coordinates yourself here.", true);
    return;
  }
  if (!navigator.geolocation) {
    setLocationMessage("This browser does not provide device location. You can enter coordinates yourself.", true);
    return;
  }

  if (activeId !== null) invalidateChoices();
  const requestId = ++sequence;
  locationRequestId = requestId;
  deviceLocationButton.disabled = true;
  preciseLocationButton.disabled = true;
  preciseLocationButton.hidden = true;
  deviceLocationButton.setAttribute("aria-busy", "true");
  setLocationMessage(highAccuracy
    ? "Asking your device for a more precise location estimate… This may take longer or use more power. You can still enter coordinates yourself."
    : "Asking your device for one energy-conscious location estimate… You can still enter coordinates yourself.");

  const finish = () => {
    if (locationRequestId !== requestId) return false;
    locationRequestId = 0;
    clearTimeout(locationRequestTimer);
    deviceLocationButton.disabled = false;
    preciseLocationButton.disabled = false;
    deviceLocationButton.removeAttribute("aria-busy");
    return true;
  };

  locationRequestTimer = setTimeout(() => {
    if (!finish()) return;
    preciseLocationButton.hidden = highAccuracy;
    setLocationMessage(highAccuracy
      ? "The more precise request also timed out. Check your system location setting, try again later, or enter coordinates yourself."
      : "No location fix arrived in time. Try near a window or outdoors, request a more precise fix once, or enter coordinates yourself.", true);
  }, highAccuracy ? 30000 : 15000);

  try {
    navigator.geolocation.getCurrentPosition(position => {
      if (!finish()) return;
      const latitude = position.coords.latitude;
      const longitude = position.coords.longitude;
      const accuracy = position.coords.accuracy;
      if (!Number.isFinite(latitude) || !Number.isFinite(longitude)
          || latitude < -90 || latitude > 90 || longitude < -180 || longitude > 180) {
        setLocationMessage("Your device returned coordinates outside the valid range. Enter the coordinates yourself.", true);
        return;
      }

      // Do not silently reuse an earlier schedule under newly filled coordinates.
      invalidateChoices();
      // Seven decimal places preserve centimeter-scale coordinate resolution;
      // the separately displayed device estimate communicates measurement error.
      form.elements.namedItem("latitude_degrees").value = latitude.toFixed(7);
      form.elements.namedItem("longitude_degrees").value = longitude.toFixed(7);
      locationCapture = {
        source: "device_geolocation",
        reportedAccuracyMeters: Number.isFinite(accuracy) && accuracy >= 0 ? accuracy : null,
        reportedFixTimestampMs: Number.isFinite(position.timestamp) ? position.timestamp : null,
        providerOfflineCapability: "unknown_browser_provider",
      };
      setup.coordinatesChanged();
      preciseLocationButton.hidden = true;
      const accuracyText = locationCapture.reportedAccuracyMeters === null
        ? "The device did not provide an accuracy estimate."
        : `The device estimates its accuracy radius at about ${Math.ceil(locationCapture.reportedAccuracyMeters)} m.`;
      setLocationMessage(`Coordinates filled. ${accuracyText} The engine is checking its offline timezone map; review and confirm its suggestion.`);
    }, error => {
      if (!finish()) return;
      const message = error.code === error.PERMISSION_DENIED
        ? "Location permission is blocked. If you want to allow it, check this site's browser permission and your system Location Services setting. You can also enter coordinates yourself."
        : error.code === error.POSITION_UNAVAILABLE
          ? highAccuracy
            ? "The more precise request also got no position. Check browser and system Location Services; your device may have no usable fix here. Try again later or enter coordinates yourself."
            : "No position came from your browser or system location provider. Check Location Services for this browser and keep Wi-Fi on; some computers have no fix offline. You can try one more precise request or enter coordinates yourself."
          : highAccuracy
            ? "The more precise request timed out. Check your system location setting, try again later, or enter coordinates yourself."
            : "No location fix arrived in time. Try near a window or outdoors, request a more precise fix once, or enter coordinates yourself.";
      preciseLocationButton.hidden = error.code === error.PERMISSION_DENIED || highAccuracy;
      setLocationMessage(message, true);
  // Start with the efficient provider. The more demanding request is opt-in
  // after a failed first fix, never a background retry or continuous watch.
  }, {
    enableHighAccuracy: highAccuracy,
    maximumAge: highAccuracy ? 0 : 30_000,
    timeout: highAccuracy ? 25_000 : 12_000,
  });
  } catch {
    if (finish()) {
      preciseLocationButton.hidden = highAccuracy;
      setLocationMessage("The browser could not start a location request. You can enter coordinates yourself.", true);
    }
  }
}

function node(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined) element.textContent = text;
  if (className) element.className = className;
  return element;
}

function ruleExplanation(rule) {
  switch (rule.kind) {
    case "solar_transit": return "The selected upper solar transit.";
    case "apparent_horizon": return "The kernel’s fixed apparent-horizon model at sea level. Local terrain and weather are not modeled.";
    case "solar_depression": return `The selected method uses a solar depression of ${rule.degrees}° below the geometric horizon.`;
    case "asr_shadow": return `The selected Asr rule uses shadow factor ${rule.factor}, in addition to the noon shadow.`;
    case "sunset_with_adjustment": return `Sunset with the selected method’s adjustment of ${rule.seconds} seconds.`;
    case "transit_with_adjustment": return `Solar transit with the selected method’s adjustment of ${rule.seconds} seconds.`;
    default: throw new Error("Unsupported event rule.");
  }
}

const names = { fajr: "Fajr", sunrise: "Sunrise", dhuhr: "Dhuhr", asr: "Asr", sunset: "Sunset", maghrib: "Maghrib", isha: "Isha" };

function render(schedule) {
  if (schedule.schema !== "salah-local-schedule-v1" || schedule.scope !== "research_preview") throw new Error("Unsupported schedule document.");
  clearResult();
  document.querySelector("#empty").hidden = true;
  document.querySelector("#schedule-context").textContent = `${schedule.requested_local_date} · ${schedule.zone_id}`;
  if (schedule.civil_date.status === "skipped") {
    notice.textContent = "This civil date was skipped under the selected timezone rules. No schedule has been substituted.";
    notice.hidden = false;
  } else if (schedule.cycle_match_status === "zero") {
    notice.textContent = "This civil date exists, but no matching solar transit was selected. No cycle has been substituted.";
    notice.hidden = false;
  } else if (schedule.cycle_match_status === "multiple") {
    notice.textContent = "More than one solar cycle matches this local date. Every cycle is shown; none was chosen silently.";
    notice.hidden = false;
  }
  for (const [index, cycle] of schedule.cycles.entries()) {
    const section = node("section");
    if (schedule.cycles.length > 1) section.append(node("h3", `Cycle ${index + 1} · transit ${cycle.selected_transit.utc}`, "cycle-label"));
    for (const event of cycle.events) {
      if (!names[event.name]) throw new Error("Unsupported event name.");
      const row = node("details", undefined, `prayer${event.name === "sunrise" || event.name === "sunset" ? " observation" : ""}`);
      const summary = node("summary");
      summary.append(node("span", names[event.name], "prayer-name"));
      let explanation;
      if (event.status === "occurs") {
        const reading = event.reading;
        const time = node("span", reading.local_time, "prayer-time");
        // Use the engine's exact local labels. JavaScript Date never converts them.
        time.append(node("span", reading.local_date, "prayer-day"));
        summary.append(time);
        explanation = `${ruleExplanation(event.rule)} UTC ${reading.utc}; offset ${reading.offset_seconds_east} seconds east of UTC. ${reading.zone_id}, IANA ${reading.tzdb_version}.`;
      } else if (event.status === "unavailable" && event.reason === "no_crossing_in_solar_cycle") {
        summary.append(node("span", "Unavailable", "missing"));
        explanation = "The selected solar condition has no crossing in this solar cycle. No high-latitude alternative has been applied.";
      } else throw new Error("Unsupported event status.");
      row.append(summary, node("p", explanation, "explanation"));
      section.append(row);
    }
    results.append(section);
  }
  const list = node("dl");
  const methodLabel = schedule.method.id === "mwl-angles-18-17"
    ? "Published angles · Fajr 18° / Isha 17° (MWL parameters)"
    : "Engineering comparison · Fajr and Isha 15°";
  const asrLabel = schedule.method.asr === "hanafi"
    ? "Hanafi calculation · extra shadow ratio 2"
    : "Standard calculation · extra shadow ratio 1";
  for (const [label, value] of [
    ["Location", `${schedule.coordinates.latitude_degrees}°, ${schedule.coordinates.longitude_degrees}°`],
    ["Location source", locationCapture.source === "device_geolocation" ? "One-time device location estimate" : locationCapture.source === "example_coordinates" ? "Example coordinates" : "Manually entered coordinates"],
    ...(locationCapture.reportedAccuracyMeters === null ? [] : [["Reported accuracy", `about ${Math.ceil(locationCapture.reportedAccuracyMeters)} m`]]),
    ["Fajr and Isha profile", methodLabel], ["Asr calculation", asrLabel],
    ["Source", schedule.method.source], ["Kernel", `${schedule.kernel.version} · ${schedule.kernel.astronomy_model}`],
    ["Rules", `IANA ${schedule.rule_pack.tzdb_version} · ${schedule.runtime_source.kind}`],
    ["Pack hash", schedule.rule_pack.sha256], ["Inventory", schedule.rule_pack.inventory_sha256],
  ]) list.append(node("dt", label), node("dd", value));
  document.querySelector("#provenance-body").replaceChildren(list);
  provenance.hidden = false;
  lastDocument = schedule;
}

try {
  worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  watchdog = setTimeout(() => failure("Loading the local engine took too long. Rebuild the artifact and reload.", true), 30000);
  worker.onmessage = ({ data }) => {
    if (terminated) return;
    if (data?.kind === "ready") {
      if (data.inventory?.schema !== "salah-zone-inventory-v1" || !Array.isArray(data.inventory.zone_ids)) return failure("The engine returned an unsupported inventory.", true);
      clearTimeout(watchdog);
      const zonePlaceholder = node("option", "Choose a time zone");
      zonePlaceholder.value = "";
      const options = data.inventory.zone_ids.map(id => { const option = node("option", id); option.value = id; return option; });
      form.elements.namedItem("zone_id").replaceChildren(zonePlaceholder, ...options);
      loaded = true;
      calculateButton.disabled = false;
      deviceLocationButton.disabled = locationRequestId !== 0;
      setup.engineReady();
      engineState.textContent = `Rust engine ready · ${options.length} named zones · IANA ${data.inventory.rule_pack.tzdb_version}`;
    } else if (data?.kind === "setup_result") {
      const job = workerJobs.get(data.id);
      if (!job) return;
      clearTimeout(job.timer);
      workerJobs.delete(data.id);
      job.resolve(data.response);
    } else if (data?.kind === "fatal") {
      failure(data.message, true);
    } else if (data?.kind === "result" && data.id === activeId) {
      clearTimeout(watchdog);
      activeId = null;
      calculateButton.disabled = false;
      calculateButton.removeAttribute("aria-busy");
      const response = data.response;
      if (response?.schema !== "salah-schedule-response-v1") return failure("The engine returned an unsupported response.", true);
      if (response.status === "error") return failure(response.error.message);
      if (response.status !== "ok") return failure("The engine returned an unknown status.", true);
      try { render(response.schedule); } catch { return failure("This schedule could not be displayed safely. No replacement times were generated.", true); }
      lastSetupMetadata = pendingSetupMetadata;
      engineState.textContent = "Calculated locally. Open any event to see why this time was returned.";
    }
  };
  worker.onerror = () => failure("The local engine could not start or stopped unexpectedly. Rebuild the web artifact and reload.", true);
} catch { failure("This browser could not start the WebAssembly worker. Use a current browser and serve the page over HTTP.", true); }

form.addEventListener("submit", async event => {
  event.preventDefault();
  if (!loaded || activeId !== null) return;
  const values = new FormData(form);
  if (!values.get("latitude_degrees") || !values.get("longitude_degrees")) return failure("Enter latitude and longitude explicitly.");
  const latitude = Number(values.get("latitude_degrees"));
  const longitude = Number(values.get("longitude_degrees"));
  if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) return failure("Enter finite latitude and longitude values.");
  clearResult();
  const requestId = ++sequence;
  activeId = requestId;
  calculateButton.disabled = true;
  calculateButton.setAttribute("aria-busy", "true");
  engineState.textContent = "Calculating on your device…";
  watchdog = setTimeout(() => failure("The engine took too long. No replacement schedule was generated. Reload to start again.", true), 30000);
  try {
    const prepared = await setup.prepare();
    if (activeId !== requestId) return;
    pendingSetupMetadata = JSON.parse(JSON.stringify({ location_input: locationCapture, ...setup.metadata() }));
    worker.postMessage({ kind: "calculate", id: requestId, selection: true, request: JSON.stringify({
      schema: "salah-schedule-request-v2", latitude_degrees: latitude, longitude_degrees: longitude,
      local_date: prepared.local_date, zone_id: values.get("zone_id"), zone_choice: prepared.zone_choice,
      method_id: values.get("method_id"), asr: values.get("asr"),
    }) });
  } catch (error) { if (activeId === requestId) failure(error.message); }
});

// Editing a setting invalidates the old display and any in-flight response.
function invalidateChoices(event) {
  if (["latitude_degrees", "longitude_degrees"].includes(event?.target?.name)) {
    preciseLocationButton.hidden = true;
    cancelLocationRequest("Your coordinates were edited. They are now treated as manually entered.");
    locationCapture = { source: "manual_coordinates", reportedAccuracyMeters: null };
    setLocationMessage("Using manually entered coordinates. The engine suggests a timezone offline; please confirm it.");
  }
  if (activeId !== null) { activeId = null; clearTimeout(watchdog); calculateButton.disabled = !loaded; calculateButton.removeAttribute("aria-busy"); }
  clearResult();
  document.querySelector("#example-note").hidden = true;
  if (loaded) engineState.textContent = "Settings changed. Calculate again to use these choices.";
}
form.addEventListener("input", invalidateChoices);
form.addEventListener("change", invalidateChoices);
form.elements.namedItem("method_id").addEventListener("change", explainMethod);
deviceLocationButton.addEventListener("click", () => useDeviceLocation());
preciseLocationButton.addEventListener("click", () => useDeviceLocation({ highAccuracy: true }));

document.querySelector("#example").addEventListener("click", () => {
  if (activeId !== null) return;
  cancelLocationRequest("Using the labeled example coordinates. Choose device location or enter your own coordinates when ready.");
  preciseLocationButton.hidden = true;
  locationCapture = { source: "example_coordinates", reportedAccuracyMeters: null };
  clearResult();
  for (const [key, value] of Object.entries({ latitude_degrees: "44.9778", longitude_degrees: "-93.2650", local_date: "2026-10-01", zone_id: "America/Chicago", method_id: "mwl-angles-18-17", asr: "hanafi" })) form.elements.namedItem(key).value = value;
  setup.useExample();
  explainMethod();
  document.querySelector("#example-note").hidden = false;
  setLocationMessage("Using the labeled Minneapolis example coordinates. Choose device location or enter your own coordinates for your location.");
});

document.querySelector("#download").addEventListener("click", () => {
  if (!lastDocument) return;
  const exportRecord = {
    schema: "salah-web-schedule-export-v2",
    schedule: lastDocument,
    location_input: {
      source: locationCapture.source,
      reported_accuracy_radius_meters: locationCapture.reportedAccuracyMeters,
    },
    setup_input: lastSetupMetadata,
  };
  const url = URL.createObjectURL(new Blob([JSON.stringify(exportRecord, null, 2) + "\n"], { type: "application/json" }));
  const link = node("a"); link.href = url; link.download = `salah-${lastDocument.requested_local_date}.json`;
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
