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

  try {
    const result = await api.storage.local.get(["serverUrl", "apiKey"]);
    const base = result.serverUrl || DEFAULT_SERVER_URL;
    const apiKey = result.apiKey || "";

    if (!apiKey) {
      notify("Not configured", "Set your API key in the Clipstash extension options.");
      return;
    }

    const response = await fetch(`${base}/api/articles`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${apiKey}`,
      },
      body: JSON.stringify({ url, tags: [] }),
    });

    if (response.ok) {
      const article = await response.json();
      notify("Saved!", `"${article.title}" saved to Clipstash.`);
    } else if (response.status === 401) {
      notify("Unauthorized", "Check your API key in the Clipstash extension options.");
    } else {
      const text = await response.text();
      notify("Save failed", text || `Server returned ${response.status}`);
    }
  } catch (err) {
    notify(
      "Connection error",
      `Could not reach Clipstash server. ${err.message}`
    );
  }
});

function notify(title, message) {
  api.notifications.create({
    type: "basic",
    title,
    message,
  });
}
