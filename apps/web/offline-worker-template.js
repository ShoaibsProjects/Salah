// Generated into offline-worker.js with an exact package identity by build_web.py.
const BUILD_ID = "__SALAH_OFFLINE_BUILD_ID__";
const ROOT = self.registration.scope;
const PREFIX = "salah-offline-v1-" + new URL(ROOT).pathname + "-";
const CACHE = PREFIX + BUILD_ID;
const MANIFEST_URL = new URL("pkg/offline-manifest.json", ROOT).href;

async function digest(bytes) {
  const hash = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(hash), byte => byte.toString(16).padStart(2, "0")).join("");
}

async function fillCache() {
  const response = await fetch(MANIFEST_URL, { cache: "no-store" });
  if (!response.ok || response.redirected) throw new Error("The offline package manifest could not be loaded.");
  const manifest = await response.json();
  if (manifest.schema !== "salah-offline-package-v1" || manifest.build_id !== BUILD_ID
      || !Array.isArray(manifest.entries) || manifest.entries.length === 0 || manifest.entries.length > 512) {
    throw new Error("The offline package manifest has an unsupported identity.");
  }
  if (await digest(new TextEncoder().encode(JSON.stringify(manifest.entries))) !== BUILD_ID) {
    throw new Error("The offline package inventory does not match this application version.");
  }
  let total = 0;
  const paths = new Set();
  for (const entry of manifest.entries) {
    if (typeof entry.path !== "string" || !/^[A-Za-z0-9._+/-]+$/.test(entry.path)
        || entry.path.startsWith("/") || entry.path.split("/").some(part => !part || part === "." || part === "..")
        || !/^[a-f0-9]{64}$/.test(entry.sha256) || !Number.isSafeInteger(entry.bytes) || entry.bytes < 0
        || paths.has(entry.path)) throw new Error("Invalid offline package inventory.");
    paths.add(entry.path);
    total += entry.bytes;
  }
  if (total > 64 * 1024 * 1024) throw new Error("The offline package exceeds this preview's size limit.");
  const cache = await caches.open(CACHE);
  for (const entry of manifest.entries) {
    const url = new URL(entry.path, ROOT);
    if (url.origin !== new URL(ROOT).origin) throw new Error("The offline package must stay on the same origin.");
    const asset = await fetch(url.href, { cache: "no-store" });
    if (!asset.ok || asset.redirected) throw new Error(`Could not load ${entry.path}.`);
    const bytes = await asset.clone().arrayBuffer();
    if (bytes.byteLength !== entry.bytes || await digest(bytes) !== entry.sha256) {
      throw new Error(`The bytes of ${entry.path} differ from the built offline package. Rebuild before installing.`);
    }
    await cache.put(url.href, asset);
  }
  await cache.put(MANIFEST_URL, new Response(JSON.stringify(manifest), { headers: { "Content-Type": "application/json" } }));
}

async function status() {
  const cache = await caches.open(CACHE);
  const response = await cache.match(MANIFEST_URL);
  if (!response) return { ready: false, build_id: BUILD_ID };
  const manifest = await response.json();
  for (const entry of manifest.entries) {
    if (!await cache.match(new URL(entry.path, ROOT).href)) return { ready: false, build_id: BUILD_ID };
  }
  return { ready: true, build_id: BUILD_ID, asset_count: manifest.entries.length };
}

self.addEventListener("install", event => {
  event.waitUntil(fillCache().catch(async error => { await caches.delete(CACHE); throw error; }));
});
self.addEventListener("activate", event => {
  event.waitUntil((async () => {
    for (const name of await caches.keys()) {
      if (name.startsWith(PREFIX) && name !== CACHE) await caches.delete(name);
    }
    await self.clients.claim();
  })());
});
self.addEventListener("fetch", event => {
  const url = new URL(event.request.url);
  if (event.request.method !== "GET" || url.origin !== new URL(ROOT).origin || !url.href.startsWith(ROOT)) return;
  event.respondWith((async () => {
    const root = new URL(ROOT);
    const path = url.pathname === root.pathname ? new URL("index.html", ROOT).href : url.origin + url.pathname;
    const cached = await (await caches.open(CACHE)).match(path);
    return cached ?? new Response("This file is not in the saved Salah package. Reinstall the complete package when its local files are available.", {
      status: 503, headers: { "Content-Type": "text/plain; charset=utf-8" },
    });
  })());
});
self.addEventListener("message", event => {
  if (event.data?.kind === "activate_update") { event.waitUntil(self.skipWaiting()); return; }
  if (!["status", "prepare"].includes(event.data?.kind) || !event.ports[0]) return;
  event.waitUntil((async () => {
    try {
      if (event.data.kind === "prepare") await fillCache();
      event.ports[0].postMessage({ status: "ok", ...await status() });
    } catch (error) { event.ports[0].postMessage({ status: "error", message: error.message }); }
  })());
});
