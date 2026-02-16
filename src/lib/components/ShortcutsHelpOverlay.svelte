<script lang="ts">
  import { shortcutsStore, ACTION_LABELS, type ShortcutAction } from "../stores/shortcuts.svelte";

  let { onClose }: { onClose: () => void } = $props();

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

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" || e.key === "?") {
      e.preventDefault();
      onClose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  class="overlay-backdrop"
  onclick={onClose}
  onkeydown={handleBackdropKeydown}
  role="dialog"
  aria-modal="true"
  aria-label="Keyboard Shortcuts"
  tabindex="-1"
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay-panel" onclick={(e) => e.stopPropagation()}>
    <div class="overlay-header">
      <span class="overlay-title">Keyboard Shortcuts</span>
      <button class="close-btn" onclick={onClose} title="Close">x</button>
    </div>

    <div class="shortcuts-list">
      {#each ALL_ACTIONS as action}
        <div class="shortcut-row">
          <span class="shortcut-label">{ACTION_LABELS[action]}</span>
          <kbd class="shortcut-key">{shortcutsStore.bindings[action]}</kbd>
        </div>
      {/each}
    </div>

    <p class="hint">
      Shortcuts are ignored while a text input is focused. Change bindings in Settings.
    </p>
  </div>
</div>

<style>
  .overlay-backdrop {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: var(--overlay-bg);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .overlay-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--overlay-shadow);
    width: 420px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .overlay-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }

  .overlay-title {
    font-weight: 600;
    font-size: 15px;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 2px 6px;
    cursor: pointer;
    border-radius: var(--radius);
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .shortcuts-list {
    overflow-y: auto;
    padding: 8px 0;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 16px;
    font-size: 13px;
  }

  .shortcut-row:hover {
    background: var(--bg-tertiary);
  }

  .shortcut-label {
    color: var(--text-primary);
  }

  .shortcut-key {
    font-family: monospace;
    font-size: 12px;
    padding: 2px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-secondary);
  }

  .hint {
    font-size: 12px;
    color: var(--text-secondary);
    padding: 10px 16px;
    border-top: 1px solid var(--border);
  }
</style>
