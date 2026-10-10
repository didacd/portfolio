(() => {
  const root = document.documentElement;
  const storageKey = "portfolio-theme";
  const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
  let preference = null;
  let button;

  const validTheme = (value) => value === "light" || value === "dark";

  try {
    const saved = window.localStorage.getItem(storageKey);
    if (validTheme(saved)) preference = saved;
  } catch {
    // Storage can be unavailable; the switch still works for this page.
  }

  function update() {
    const dark = preference ? preference === "dark" : systemTheme.matches;
    if (preference) {
      root.dataset.theme = preference;
    } else {
      delete root.dataset.theme;
    }

    if (button) button.setAttribute("aria-checked", String(dark));
  }

  // This script runs before stylesheets so saved preferences apply immediately.
  update();

  document.addEventListener("DOMContentLoaded", () => {
    button = document.getElementById("theme-switch");
    if (!button) return;

    update();
    button.hidden = false;
    button.addEventListener("click", () => {
      const dark = preference ? preference === "dark" : systemTheme.matches;
      preference = dark ? "light" : "dark";
      update();

      try {
        window.localStorage.setItem(storageKey, preference);
      } catch {
        // Keep the in-memory preference when storage is blocked.
      }
    });
  });

  systemTheme.addEventListener("change", update);
  window.addEventListener("storage", (event) => {
    if (event.key !== storageKey && event.key !== null) return;
    preference = validTheme(event.newValue) ? event.newValue : null;
    update();
  });
})();
