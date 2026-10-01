import init, { calculate_schedule_json, calculate_schedule_with_selection_json, zone_inventory_json, lookup_timezone_json, local_clock_json } from "./pkg/salah_wasm.js";

let ready = false;
try {
  await init({ module_or_path: new URL("./pkg/salah_wasm_bg.wasm", import.meta.url) });
  ready = true;
  self.postMessage({ kind: "ready", inventory: JSON.parse(zone_inventory_json()) });
} catch {
  self.postMessage({ kind: "fatal", message: "The local engine could not load. Build the web artifact and serve this folder over HTTP." });
}

self.onmessage = ({ data }) => {
  if (!ready || !["calculate", "lookup", "clock"].includes(data?.kind) || !Number.isSafeInteger(data.id) || typeof data.request !== "string") return;
  try {
    const operation = data.kind === "lookup" ? lookup_timezone_json
      : data.kind === "clock" ? local_clock_json
        : data.selection ? calculate_schedule_with_selection_json : calculate_schedule_json;
    self.postMessage({ kind: data.kind === "calculate" ? "result" : "setup_result", id: data.id, response: JSON.parse(operation(data.request)) });
  } catch {
    ready = false;
    self.postMessage({ kind: "fatal", message: "The engine stopped unexpectedly. No replacement times were generated. Reload to start a new engine." });
  }
};
