const api = globalThis.browser ?? globalThis.chrome;
const DEFAULT_SERVER_URL = "http://localhost:3000";

api.contextMenus.create({
  id: "save-to-clipstash",
  title: "Save to Clipstash",
  contexts: ["page"],
});

api.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== "save-to-clipstash") return;

  const url = tab.url;
  if (!url) return;

  const result = await api.storage.local.get(["serverUrl", "apiKey"]);
  const base = result.serverUrl || DEFAULT_SERVER_URL;
  const apiKey = result.apiKey || "";

  if (!apiKey) {
    api.runtime.openOptionsPage();
    return;
  }

  try {
    const response = await fetch(`${base}/api/articles`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${apiKey}`,
      },
      body: JSON.stringify({ url }),
    });

    if (response.ok) {
      notify(tab.id, "success", "Saved to Clipstash!");
    } else if (response.status === 401) {
      notify(tab.id, "error", "Clipstash: invalid API key.");
    } else {
      const text = await response.text();
      notify(tab.id, "error", `Clipstash: ${text || `server error ${response.status}`}`);
    }
  } catch (err) {
    notify(tab.id, "error", `Clipstash: could not reach server. ${err.message}`);
  }
});

function notify(tabId, type, message) {
  api.scripting.executeScript({
    target: { tabId },
    func: (type, message) => {
      const el = document.createElement("div");
      el.textContent = message;
      Object.assign(el.style, {
        position: "fixed",
        top: "16px",
        right: "16px",
        zIndex: "2147483647",
        padding: "10px 16px",
        borderRadius: "6px",
        fontFamily: "sans-serif",
        fontSize: "14px",
        color: "#fff",
        background: type === "success" ? "#2563eb" : "#dc2626",
        boxShadow: "0 2px 8px rgba(0,0,0,0.3)",
        transition: "opacity 0.4s",
      });
      document.body.appendChild(el);
      setTimeout(() => {
        el.style.opacity = "0";
        setTimeout(() => el.remove(), 400);
      }, 3000);
    },
    args: [type, message],
  });
}
