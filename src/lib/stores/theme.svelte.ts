import * as commands from "../api/commands";

export type ThemePreference = "system" | "light" | "dark";
type ResolvedTheme = "light" | "dark";

class ThemeStore {
  preference = $state<ThemePreference>("system");
  osTheme = $state<ResolvedTheme>("dark");

  resolved = $derived<ResolvedTheme>(
    this.preference === "system" ? this.osTheme : this.preference
  );

  private mediaQuery: MediaQueryList | null = null;

  async init() {
    // Listen to OS preference
    this.mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    this.osTheme = this.mediaQuery.matches ? "dark" : "light";
    this.mediaQuery.addEventListener("change", (e) => {
      this.osTheme = e.matches ? "dark" : "light";
    });

    // Load persisted preference
    try {
      const saved = await commands.getSetting("theme");
      if (saved === "light" || saved === "dark" || saved === "system") {
        this.preference = saved;
      }
    } catch {
      // default to system
    }
  }

  async setPreference(pref: ThemePreference) {
    this.preference = pref;
    try {
      await commands.setSetting("theme", pref);
    } catch {
      // non-critical
    }
  }
}

export const themeStore = new ThemeStore();
