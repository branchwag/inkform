"use client";

import { useSyncExternalStore } from "react";

type Theme = "light" | "dark";

const storageKey = "inkform-theme";

function preferredTheme(): Theme {
  const storedTheme = window.localStorage.getItem(storageKey);
  if (storedTheme === "light" || storedTheme === "dark") {
    return storedTheme;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function currentTheme(): Theme {
  const activeTheme = document.documentElement.dataset.theme;
  return activeTheme === "dark" || activeTheme === "light" ? activeTheme : preferredTheme();
}

function subscribeToThemeChanges(onStoreChange: () => void): () => void {
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  window.addEventListener("inkform-theme-change", onStoreChange);
  window.addEventListener("storage", onStoreChange);
  mediaQuery.addEventListener("change", onStoreChange);

  return () => {
    window.removeEventListener("inkform-theme-change", onStoreChange);
    window.removeEventListener("storage", onStoreChange);
    mediaQuery.removeEventListener("change", onStoreChange);
  };
}

function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;
  window.localStorage.setItem(storageKey, theme);
  window.dispatchEvent(new Event("inkform-theme-change"));
}

export function ThemeToggle() {
  const theme = useSyncExternalStore(subscribeToThemeChanges, currentTheme, () => "light");

  const nextTheme = theme === "dark" ? "light" : "dark";

  return (
    <button
      className="theme-toggle"
      type="button"
      onClick={() => applyTheme(nextTheme)}
      aria-label={`Switch to ${nextTheme} mode`}
      aria-pressed={theme === "dark"}
    >
      <span aria-hidden="true" className="theme-toggle__mark">
        {theme === "dark" ? "Light" : "Dark"}
      </span>
      <span className="theme-toggle__label">mode</span>
    </button>
  );
}
