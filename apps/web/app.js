const form = document.querySelector("#schedule-form");
const calculateButton = document.querySelector("#calculate");
const engineState = document.querySelector("#engine-state");
const errorBox = document.querySelector("#error");
const notice = document.querySelector("#notice");
const results = document.querySelector("#results");
const provenance = document.querySelector("#provenance");
let sequence = 0;
let activeId = null;
let lastDocument = null;
let worker;
let watchdog;
let loaded = false;
let terminated = false;

function clearResult() {
  lastDocument = null;
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
  if (fatal) { loaded = false; terminated = true; worker?.terminate(); }
  calculateButton.disabled = !loaded;
  calculateButton.removeAttribute("aria-busy");
  engineState.textContent = fatal ? "Engine unavailable. Reload after rebuilding the local artifact." : "Ready for another explicit calculation.";
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
  for (const [label, value] of [
    ["Location", `${schedule.coordinates.latitude_degrees}°, ${schedule.coordinates.longitude_degrees}°`],
    ["Method", `${schedule.method.id} v${schedule.method.revision}`], ["Asr", schedule.method.asr],
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
      const options = data.inventory.zone_ids.map(id => { const option = node("option"); option.value = id; return option; });
      document.querySelector("#zone-list").replaceChildren(...options);
      loaded = true;
      calculateButton.disabled = false;
      engineState.textContent = `Rust engine ready · ${options.length} named zones · IANA ${data.inventory.rule_pack.tzdb_version}`;
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
      engineState.textContent = "Calculated locally. Open any event to see why this time was returned.";
    }
  };
  worker.onerror = () => failure("The local engine could not start or stopped unexpectedly. Rebuild the web artifact and reload.", true);
} catch { failure("This browser could not start the WebAssembly worker. Use a current browser and serve the page over HTTP.", true); }

form.addEventListener("submit", event => {
  event.preventDefault();
  if (!loaded || activeId !== null) return;
  const values = new FormData(form);
  if (!values.get("latitude_degrees") || !values.get("longitude_degrees")) return failure("Enter latitude and longitude explicitly.");
  const latitude = Number(values.get("latitude_degrees"));
  const longitude = Number(values.get("longitude_degrees"));
  if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) return failure("Enter finite latitude and longitude values.");
  clearResult();
  activeId = ++sequence;
  calculateButton.disabled = true;
  calculateButton.setAttribute("aria-busy", "true");
  engineState.textContent = "Calculating on your device…";
  watchdog = setTimeout(() => failure("The engine took too long. No replacement schedule was generated. Reload to start again.", true), 30000);
  worker.postMessage({ kind: "calculate", id: activeId, request: JSON.stringify({
    schema: "salah-schedule-request-v1", latitude_degrees: latitude, longitude_degrees: longitude,
    local_date: values.get("local_date"), zone_id: values.get("zone_id"),
    method_id: values.get("method_id"), asr: values.get("asr"),
  }) });
});

// Editing a setting invalidates the old display and any in-flight response.
function invalidateChoices() {
  if (activeId !== null) { activeId = null; clearTimeout(watchdog); calculateButton.disabled = !loaded; calculateButton.removeAttribute("aria-busy"); }
  clearResult();
  document.querySelector("#example-note").hidden = true;
  if (loaded) engineState.textContent = "Settings changed. Calculate again to use these choices.";
}
form.addEventListener("input", invalidateChoices);
form.addEventListener("change", invalidateChoices);

document.querySelector("#example").addEventListener("click", () => {
  if (activeId !== null) return;
  clearResult();
  for (const [key, value] of Object.entries({ latitude_degrees: "44.9778", longitude_degrees: "-93.2650", local_date: "2026-09-30", zone_id: "America/Chicago", method_id: "mwl-angles-18-17", asr: "hanafi" })) form.elements.namedItem(key).value = value;
  document.querySelector("#example-note").hidden = false;
});

document.querySelector("#download").addEventListener("click", () => {
  if (!lastDocument) return;
  const url = URL.createObjectURL(new Blob([JSON.stringify(lastDocument, null, 2) + "\n"], { type: "application/json" }));
  const link = node("a"); link.href = url; link.download = `salah-${lastDocument.requested_local_date}.json`;
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
