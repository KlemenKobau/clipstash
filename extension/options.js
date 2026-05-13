const api = globalThis.browser ?? globalThis.chrome;
const input = document.getElementById("server-url");
const status = document.getElementById("status");

api.storage.local.get("serverUrl").then((result) => {
  input.value = result.serverUrl || "";
});

let saveTimeout;
input.addEventListener("input", () => {
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(() => {
    const serverUrl = input.value.replace(/\/+$/, "");
    api.storage.local.set({ serverUrl });
    status.textContent = "Saved";
    setTimeout(() => (status.textContent = ""), 1500);
  }, 400);
});
