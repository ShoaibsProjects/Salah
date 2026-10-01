import init, { calculate_schedule_json, zone_inventory_json } from "./pkg/salah_wasm.js";

let ready = false;
try {
  await init({ module_or_path: new URL("./pkg/salah_wasm_bg.wasm", import.meta.url) });
  ready = true;
  self.postMessage({ kind: "ready", inventory: JSON.parse(zone_inventory_json()) });
} catch {
  self.postMessage({ kind: "fatal", message: "The local engine could not load. Build the web artifact and serve this folder over HTTP." });
}

self.onmessage = ({ data }) => {
  if (!ready || data?.kind !== "calculate" || !Number.isSafeInteger(data.id) || typeof data.request !== "string") return;
  try {
    self.postMessage({ kind: "result", id: data.id, response: JSON.parse(calculate_schedule_json(data.request)) });
  } catch {
    ready = false;
    self.postMessage({ kind: "fatal", message: "The engine stopped unexpectedly. No replacement times were generated. Reload to start a new engine." });
  }
};
