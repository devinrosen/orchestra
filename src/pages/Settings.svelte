<script lang="ts">
  import * as commands from "../lib/api/commands";
  import { themeStore, type ThemePreference } from "../lib/stores/theme.svelte";
  import KeybindingRow from "../lib/components/KeybindingRow.svelte";
  import { shortcutsStore, type ShortcutAction } from "../lib/stores/shortcuts.svelte";

  const ALL_ACTIONS: ShortcutAction[] = [
    "play-pause",
    "next-track",
    "prev-track",
    "volume-up",
    "volume-down",
    "nav-library",
    "nav-favorites",
    "nav-playlists",
    "nav-profiles",
    "nav-devices",
    "nav-settings",
    "rescan-library",
    "focus-search",
    "show-shortcuts",
  ];

  let settings = $state<Record<string, string>>({});
  let loading = $state(true);
  let saved = $state(false);

  $effect(() => {
    loadSettings();
  });

  async function loadSettings() {
    loading = true;
    try {
      const all = await commands.getAllSettings();
      settings = Object.fromEntries(all);
    } catch {
      // defaults
    } finally {
      loading = false;
    }
  }

  async function saveSetting(key: string, value: string) {
    settings[key] = value;
    await commands.setSetting(key, value);
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }
</script>

<div class="settings-page">
  <h2>Settings</h2>

  {#if loading}
    <p>Loading settings...</p>
  {:else}
    <div class="settings-group">
      <h3>Appearance</h3>

      <div class="setting-row">
        <div class="setting-info">
          <label for="setting-theme">Theme</label>
          <p class="setting-desc">Choose light or dark mode, or follow the OS setting</p>
        </div>
        <select
          id="setting-theme"
          value={themeStore.preference}
          onchange={(e) => themeStore.setPreference((e.target as HTMLSelectElement).value as ThemePreference)}
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </div>
    </div>

    <div class="settings-group">
      <h3>General</h3>

      <div class="setting-row">
        <div class="setting-info">
          <label for="setting-sync-mode">Default Sync Mode</label>
          <p class="setting-desc">Default sync mode for new profiles</p>
        </div>
        <select
          id="setting-sync-mode"
          value={settings["default_sync_mode"] ?? "one_way"}
          onchange={(e) => saveSetting("default_sync_mode", (e.target as HTMLSelectElement).value)}
        >
          <option value="one_way">One-Way</option>
          <option value="two_way">Two-Way</option>
        </select>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label for="setting-hash-mode">Hash Verification</label>
          <p class="setting-desc">Use content hashing to detect changes (slower but more accurate)</p>
        </div>
        <select
          id="setting-hash-mode"
          value={settings["hash_mode"] ?? "auto"}
          onchange={(e) => saveSetting("hash_mode", (e.target as HTMLSelectElement).value)}
        >
          <option value="auto">Auto (size + mtime first)</option>
          <option value="always">Always hash</option>
          <option value="never">Never hash (fastest)</option>
        </select>
      </div>
    </div>

    <div class="settings-group">
      <h3>Keyboard Shortcuts</h3>

      <div class="shortcut-rows">
        {#each ALL_ACTIONS as action}
          <KeybindingRow {action} />
        {/each}
      </div>

      <button class="secondary reset-btn" onclick={() => shortcutsStore.resetToDefaults()}>
        Reset to Defaults
      </button>
    </div>

    {#if saved}
      <div class="save-indicator">Settings saved</div>
    {/if}
  {/if}
</div>

<style>
  .settings-page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 600px;
  }

  .settings-page h2 {
    font-size: 20px;
    font-weight: 600;
  }

  .settings-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .settings-group h3 {
    font-size: 14px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
  }

  .setting-info label {
    font-weight: 500;
  }

  .setting-desc {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .save-indicator {
    color: var(--success);
    font-size: 13px;
  }

  .shortcut-rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .reset-btn {
    margin-top: 4px;
    align-self: flex-start;
    font-size: 13px;
    padding: 6px 14px;
  }
</style>
