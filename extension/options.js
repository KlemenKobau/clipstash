const api = globalThis.browser ?? globalThis.chrome;
const urlInput = document.getElementById("server-url");
const keyInput = document.getElementById("api-key");
const status = document.getElementById("status");

api.storage.local.get(["serverUrl", "apiKey"]).then((result) => {
  urlInput.value = result.serverUrl || "";
  keyInput.value = result.apiKey || "";
});

function save() {
  const serverUrl = urlInput.value.replace(/\/+$/, "");
  const apiKey = keyInput.value.trim();
  api.storage.local.set({ serverUrl, apiKey });
  status.textContent = "Saved";
  setTimeout(() => (status.textContent = ""), 1500);
}

let saveTimeout;
function scheduleSave() {
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(save, 400);
}

urlInput.addEventListener("input", scheduleSave);
keyInput.addEventListener("input", scheduleSave);
