import { setupCitySearch } from "./cities.js";
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
const solarEvidenceBox = document.querySelector("#solar-evidence");
const solarEvidenceBody = document.querySelector("#solar-evidence-body");
let sequence = 0;
let activeId = null;
let lastDocument = null;
let solarEvidence = null;
let lastSetupMetadata = null;
let pendingSetupMetadata = null;
const workerJobs = new Map();
let worker;
let watchdog;
let loaded = false;
let terminated = false;
let locationReady = false;
let calculateWhenReady = false;
let locationRequestId = 0;
let locationRequestTimer;
let locationCapture = { source: "manual_coordinates", reportedAccuracyMeters: null };

const methods = {
  "mwl-angles-18-17": "This published setting is one option. Your local mosque may use another.",
  "research-15": "This comparison setting may differ from your local mosque’s timetable.",
};

const setup = new OfflineSetup(form, sendSetupRequest, () => invalidateChoices(), ({ ready, zone, needsChoice } = {}) => {
  locationReady = !!ready;
  calculateButton.disabled = !loaded || !locationReady;
  if (locationReady) setLocationMessage("Ready for today" + (locationCapture.cityName ? " in " + locationCapture.cityName : "") + ".");
  else if (needsChoice) setLocationMessage("Please choose your time zone in More settings.", true);
  else if (zone) setLocationMessage("Checking today’s date for this time zone…");
  if (locationReady && calculateWhenReady) {
    calculateWhenReady = false;
    queueMicrotask(() => form.requestSubmit());
  }
});

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
  document.querySelector("#schedule-context").textContent = "Choose a city to see today’s times.";
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
  calculateButton.disabled = !loaded || !locationReady;
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
    ?? "The local mosque’s profile is a useful starting point; its imam can help with the choice.";
}

function useDeviceLocation({ highAccuracy = false } = {}) {
  if (locationRequestId !== 0) return;
  if (!window.isSecureContext) {
    setLocationMessage("Location isn’t available here. Search for your city instead.", true);
    return;
  }
  if (!navigator.geolocation) {
    setLocationMessage("Location isn’t available in this browser. Search for your city instead.", true);
    return;
  }

  if (activeId !== null) invalidateChoices();
  const requestId = ++sequence;
  locationRequestId = requestId;
  deviceLocationButton.disabled = true;
  preciseLocationButton.disabled = true;
  preciseLocationButton.hidden = true;
  deviceLocationButton.setAttribute("aria-busy", "true");
  setLocationMessage(highAccuracy ? "Finding a more precise location…" : "Finding your location…");

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
    setLocationMessage("Location took too long. Search for your city or try again.", true);
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
      locationReady = false;
      calculateButton.disabled = true;
      calculateWhenReady = true;
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
      setLocationMessage("Finding the time zone for your location…");
    }, error => {
      if (!finish()) return;
      const message = error.code === error.PERMISSION_DENIED
        ? "Location permission is off. Search for your city or allow location in browser settings."
        : "We couldn’t find your location. Search for your city or try again.";
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
      setLocationMessage("We couldn’t find your location. Search for your city or try again.", true);
    }
  }
}

function node(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined) element.textContent = text;
  if (className) element.className = className;
  return element;
}

function renderSolarEvidence(evidence) {
  if (evidence?.schema !== "salah-solar-validation-evidence-v1"
      || evidence.audit_id !== "solar-reference-audit-2026-10-01"
      || evidence.scope?.selected_case_count !== 15
      || evidence.scope?.event_status_count !== 30
      || evidence.scope?.matching_event_status_count !== 30
      || evidence.scope?.ordinary_timed_event_count !== 20
      || !Array.isArray(evidence.comparisons) || evidence.comparisons.length !== 2
      || !Array.isArray(evidence.references) || evidence.references.length !== 4
      || !Array.isArray(evidence.limitations) || evidence.limitations.length === 0
      || evidence.grazing_case?.latitude_degrees !== 65.72
      || evidence.comparisons.some(item => !Number.isFinite(item.median_absolute_difference_seconds)
        || !Number.isFinite(item.maximum_absolute_difference_seconds) || item.timed_event_count !== 20)) {
    solarEvidenceBody.replaceChildren(node("p", "The selected research note could not be read. This does not change schedule calculation; the Rust engine remains the only calculator."));
    solarEvidenceBox.hidden = false;
    return;
  }
  solarEvidence = evidence;
  const children = [node("p", `This is a check of the solar model, not a verification of your selected day or location. Sunrise and Sunset outputs were compared across ${evidence.scope.selected_case_count} selected 2026 cases. The models agreed on all ${evidence.scope.matching_event_status_count} of ${evidence.scope.event_status_count} event-existence outcomes.`)];
  for (const comparison of evidence.comparisons) {
    const card = node("div", undefined, "evidence-result");
    card.append(node("strong", comparison.label));
    card.append(node("p", `${comparison.timed_event_count} ordinary event times · median absolute difference ${comparison.median_absolute_difference_seconds.toFixed(2)} s · largest ${comparison.maximum_absolute_difference_seconds.toFixed(3)} s`));
    card.append(node("p", comparison.source_artifact, "field-help"));
    children.push(card);
  }
  children.push(node("p", `Near a very shallow crossing at ${evidence.grazing_case.latitude_degrees}°N, the references differed from Salah by about 50 seconds. A small difference in modeled Sun height near the horizon can move an event time substantially. No universal correction follows from this case.`));
  children.push(node("p", "The installed app still calculates with NOAA-MEEUS-SOLAR-2. JPL and NREL were offline research references; they are not loaded to calculate your schedule."));
  const limits = node("details", undefined, "evidence-limitations");
  limits.append(node("summary", "What this comparison does not prove"));
  const list = node("ul");
  for (const limitation of evidence.limitations) list.append(node("li", limitation));
  limits.append(list, node("p", `Next research: ${evidence.next_research}`));
  const sources = node("details", undefined, "evidence-limitations");
  sources.append(node("summary", "Reference editions and data identity"));
  for (const reference of evidence.references) {
    const item = node("p", undefined, "evidence-reference");
    item.append(node("strong", reference.id));
    if (reference.implementation) item.append(node("span", ` · ${reference.implementation}`));
    if (reference.generated_date) item.append(node("span", ` · generated ${reference.generated_date}`));
    if (reference.publication_doi) item.append(node("span", ` · DOI ${reference.publication_doi}`));
    if (reference.sha256) item.append(node("span", ` · SHA-256 ${reference.sha256}`));
    if (reference.source_file_sha256) {
      for (const [name, hash] of Object.entries(reference.source_file_sha256)) item.append(node("span", ` · ${name} SHA-256 ${hash}`));
    }
    if (reference.url) item.append(node("span", ` · ${reference.url}`));
    sources.append(item);
  }
  children.push(limits, sources, node("p", `Evidence ${evidence.audit_id} · ${evidence.review_status.replaceAll("_", " ")}`, "evidence-footnote"));
  solarEvidenceBody.replaceChildren(...children);
  solarEvidenceBox.hidden = false;
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
  document.querySelector("#schedule-context").textContent = (locationCapture.cityName || schedule.zone_id) + " · " + schedule.requested_local_date;
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
    ["Location source", locationCapture.source === "device_geolocation" ? "One-time device location estimate" : locationCapture.source === "example_coordinates" ? "Example coordinates" : locationCapture.source === "offline_city" ? `Offline city point · ${locationCapture.cityName}` : "Manually entered coordinates"],
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
      renderSolarEvidence(data.solarEvidence);
      clearTimeout(watchdog);
      const zonePlaceholder = node("option", "Choose a time zone");
      zonePlaceholder.value = "";
      const options = data.inventory.zone_ids.map(id => { const option = node("option", id); option.value = id; return option; });
      form.elements.namedItem("zone_id").replaceChildren(zonePlaceholder, ...options);
      loaded = true;
      calculateButton.disabled = !locationReady;
      deviceLocationButton.disabled = locationRequestId !== 0;
      setup.engineReady();
      engineState.textContent = "Prayer times are ready to calculate.";
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
      engineState.textContent = "Today’s estimated prayer times are ready.";
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
    locationReady = false;
    calculateButton.disabled = true;
    preciseLocationButton.hidden = true;
    cancelLocationRequest("Your coordinates were edited. They are now treated as manually entered.");
    locationCapture = { source: "manual_coordinates", reportedAccuracyMeters: null };
    setLocationMessage("Finding the time zone for these coordinates…");
  }
  if (activeId !== null) { activeId = null; clearTimeout(watchdog); calculateButton.disabled = !loaded || !locationReady; calculateButton.removeAttribute("aria-busy"); }
  clearResult();
  if (loaded) engineState.textContent = "Settings changed. Calculate again to use these choices.";
}
form.addEventListener("input", invalidateChoices);
form.addEventListener("change", invalidateChoices);
document.querySelector('[name="method_id-choice"]').addEventListener("change", event => {
  form.elements.namedItem("method_id").value = event.target.value;
  explainMethod(); invalidateChoices();
});
document.querySelector('[name="asr-choice"]').addEventListener("change", event => {
  form.elements.namedItem("asr").value = event.target.value;
  invalidateChoices();
});
deviceLocationButton.addEventListener("click", () => useDeviceLocation());
preciseLocationButton.addEventListener("click", () => useDeviceLocation({ highAccuracy: true }));

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

document.querySelector("#download-audit").addEventListener("click", () => {
  if (!solarEvidence) return;
  const url = URL.createObjectURL(new Blob([JSON.stringify(solarEvidence, null, 2) + "\n"], { type: "application/json" }));
  const link = node("a"); link.href = url; link.download = `${solarEvidence.audit_id}.json`;
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});

setupCitySearch(city => {
  locationReady = false;
  calculateWhenReady = true;
  calculateButton.disabled = true;
  cancelLocationRequest("Using your selected city.");
  invalidateChoices();
  form.elements.namedItem("latitude_degrees").value = city.latitude;
  form.elements.namedItem("longitude_degrees").value = city.longitude;
  locationCapture = { source: "offline_city", cityName: `${city.name}, ${city.country}`, geonamesId: city.id, reportedAccuracyMeters: null };
  setLocationMessage("Finding today’s time zone…");
  setup.coordinatesChanged({ immediate: true });
});
