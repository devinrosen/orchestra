import * as commands from "../api/commands";

export type ShortcutAction =
  | "play-pause"
  | "next-track"
  | "prev-track"
  | "volume-up"
  | "volume-down"
  | "nav-library"
  | "nav-favorites"
  | "nav-playlists"
  | "nav-profiles"
  | "nav-devices"
  | "nav-settings"
  | "rescan-library"
  | "focus-search"
  | "show-shortcuts";

export const DEFAULT_BINDINGS: Record<ShortcutAction, string> = {
  "play-pause": "Space",
  "next-track": "ArrowRight",
  "prev-track": "ArrowLeft",
  "volume-up": "ArrowUp",
  "volume-down": "ArrowDown",
  "nav-library": "1",
  "nav-favorites": "2",
  "nav-playlists": "3",
  "nav-profiles": "4",
  "nav-devices": "5",
  "nav-settings": "6",
  "rescan-library": "Ctrl+r",
  "focus-search": "/",
  "show-shortcuts": "?",
};

export const ACTION_LABELS: Record<ShortcutAction, string> = {
  "play-pause": "Play / Pause",
  "next-track": "Next Track",
  "prev-track": "Previous Track",
  "volume-up": "Volume Up",
  "volume-down": "Volume Down",
  "nav-library": "Go to Library",
  "nav-favorites": "Go to Favorites",
  "nav-playlists": "Go to Playlists",
  "nav-profiles": "Go to Sync Profiles",
  "nav-devices": "Go to Devices",
  "nav-settings": "Go to Settings",
  "rescan-library": "Rescan Library",
  "focus-search": "Focus Search",
  "show-shortcuts": "Show Keyboard Shortcuts",
};

/** Serialize a KeyboardEvent into the canonical key string format. */
export function eventToKey(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.metaKey) parts.push("Meta");
  if (e.altKey) parts.push("Alt");

  // Normalize the key name
  let key = e.key;
  if (key === " ") key = "Space";

  // Only include Shift as a modifier for non-printable keys (arrows, function keys, etc.).
  // For printable characters like "?" (Shift+/), use the resulting character directly.
  if (e.shiftKey && key.length !== 1) parts.push("Shift");

  parts.push(key);
  return parts.join("+");
}

class ShortcutsStore {
  bindings = $state<Record<ShortcutAction, string>>({ ...DEFAULT_BINDINGS });

  async load() {
    try {
      const raw = await commands.getSetting("keyboard_shortcuts");
      if (raw) {
        const parsed = JSON.parse(raw) as Partial<Record<ShortcutAction, string>>;
        // Merge with defaults so new actions added later still have bindings
        this.bindings = { ...DEFAULT_BINDINGS, ...parsed };
      }
    } catch {
      // Malformed JSON or missing key — fall back to defaults silently
      this.bindings = { ...DEFAULT_BINDINGS };
    }
  }

  async save() {
    try {
      await commands.setSetting("keyboard_shortcuts", JSON.stringify(this.bindings));
    } catch {
      // Non-critical: ignore persistence failures
    }
  }

  async resetToDefaults() {
    this.bindings = { ...DEFAULT_BINDINGS };
    await this.save();
  }

  async setBinding(action: ShortcutAction, key: string) {
    this.bindings = { ...this.bindings, [action]: key };
    await this.save();
  }

  /** Return the action that matches a KeyboardEvent, or null. */
  match(e: KeyboardEvent): ShortcutAction | null {
    const key = eventToKey(e);
    for (const [action, binding] of Object.entries(this.bindings)) {
      if (binding === key) {
        return action as ShortcutAction;
      }
    }
    return null;
  }

  /** Return all actions that share the same binding as the given action. */
  conflictsFor(action: ShortcutAction): ShortcutAction[] {
    const binding = this.bindings[action];
    return (Object.keys(this.bindings) as ShortcutAction[]).filter(
      (a) => a !== action && this.bindings[a] === binding,
    );
  }

  /** True if any two actions share the same binding. */
  hasConflict(action: ShortcutAction): boolean {
    return this.conflictsFor(action).length > 0;
  }
}

export const shortcutsStore = new ShortcutsStore();
