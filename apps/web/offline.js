const button = document.querySelector("#prepare-offline");
const message = document.querySelector("#offline-status");
let registration;
let applying = false;
const scope = new URL("./", import.meta.url).href;

function rpc(worker, kind) {
  return new Promise((resolve, reject) => {
    const channel = new MessageChannel();
    const timer = setTimeout(() => { channel.port1.close(); reject(new Error("Saving the offline package took too long. Keep the local files available and try again.")); }, 60000);
    channel.port1.onmessage = ({ data }) => { clearTimeout(timer); channel.port1.close(); data?.status === "ok" ? resolve(data) : reject(new Error(data?.message ?? "The offline package returned an unsupported response.")); };
    try { worker.postMessage({ kind }, [channel.port2]); }
    catch (error) { clearTimeout(timer); channel.port1.close(); channel.port2.close(); reject(error); }
  });
}
function showStatus(status) {
  message.textContent = status.ready
    ? `App and engine files saved for offline reopening · ${status.build_id.slice(0, 12)}. Browser storage can be cleared or evicted; this does not guarantee a device location fix.`
    : "The complete offline package is not saved. Keep the local files available and save the package again.";
  if (registration?.waiting) showWaiting();
}
function showWaiting() {
  message.textContent = "A complete new offline version is saved. Apply it when ready; applying reloads the page.";
  button.textContent = "Apply saved offline update";
}
function waitForInstall(worker) {
  return new Promise((resolve, reject) => {
    const finish = () => {
      if (["installed", "activated"].includes(worker.state)) { clearTimeout(timer); worker.removeEventListener("statechange", finish); resolve(); }
      else if (worker.state === "redundant") { clearTimeout(timer); worker.removeEventListener("statechange", finish); reject(new Error("The offline package did not install. Rebuild its files and try again.")); }
    };
    const timer = setTimeout(() => { worker.removeEventListener("statechange", finish); reject(new Error("Offline installation took too long.")); }, 60000);
    worker.addEventListener("statechange", finish);
    finish();
  });
}

if (!window.isSecureContext || !("serviceWorker" in navigator)) {
  button.disabled = true;
  message.textContent = "Offline browser storage requires localhost or HTTPS and service-worker support. The Rust engine can still run from locally served files without internet.";
} else {
  navigator.serviceWorker.getRegistration(scope).then(found => {
    registration ??= found;
    if (registration?.waiting) showWaiting();
  }).catch(error => { message.textContent = error.message; });
  navigator.serviceWorker.addEventListener("controllerchange", () => {
    if (applying) location.reload();
    else if (navigator.serviceWorker.controller) rpc(navigator.serviceWorker.controller, "status").then(showStatus).catch(error => { message.textContent = error.message; });
  });
  if (navigator.serviceWorker.controller) rpc(navigator.serviceWorker.controller, "status").then(showStatus).catch(error => { message.textContent = error.message; });
  button.addEventListener("click", async () => {
    button.disabled = true;
    try {
      registration ??= await navigator.serviceWorker.getRegistration(scope);
      if (registration?.waiting) {
        applying = true;
        message.textContent = "Applying the saved version. The page will reload and clear the current calculation.";
        registration.waiting.postMessage({ kind: "activate_update" });
        return;
      }
      message.textContent = "Saving the app, Rust engine, embedded maps, and notices from the available local files…";
      registration = await navigator.serviceWorker.register("./offline-worker.js", { updateViaCache: "none" });
      if (registration.active && !registration.installing && !registration.waiting) await registration.update();
      const installer = registration.installing;
      if (installer) await waitForInstall(installer);
      if (registration.waiting) {
        showWaiting();
      } else {
        const active = registration.active ?? (await navigator.serviceWorker.ready).active;
        showStatus(await rpc(active, installer ? "status" : "prepare"));
        button.textContent = "Refresh saved offline copy";
      }
    } catch (error) { applying = false; message.textContent = error.message; }
    finally { button.disabled = applying; }
  });
}
