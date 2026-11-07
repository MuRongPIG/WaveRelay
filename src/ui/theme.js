(() => {
  const key = "waverelay.theme";
  const system = window.matchMedia("(prefers-color-scheme: dark)");
  let preference;
  try {
    const saved = localStorage.getItem(key);
    if (saved === "light" || saved === "dark") preference = saved;
  } catch {
    // The toggle still works for the session if storage is unavailable.
  }

  const root = document.documentElement;
  function applyTheme() {
    const theme = preference ?? (system.matches ? "dark" : "light");
    root.dataset.theme = theme;
    document.getElementById("theme-toggle")?.setAttribute("aria-pressed", String(theme === "dark"));
  }

  applyTheme();
  system.addEventListener("change", applyTheme);
  document.addEventListener("DOMContentLoaded", () => {
    const toggle = document.getElementById("theme-toggle");
    applyTheme();
    toggle.addEventListener("click", () => {
      preference = root.dataset.theme === "dark" ? "light" : "dark";
      applyTheme();
      try {
        localStorage.setItem(key, preference);
      } catch {
        // Keep the user's selection active even if it cannot be saved.
      }
    });
  });
})();
