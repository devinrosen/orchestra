<script lang="ts">
  import { shortcutsStore, ACTION_LABELS, eventToKey, type ShortcutAction } from "../stores/shortcuts.svelte";

  let { action }: { action: ShortcutAction } = $props();

  let recording = $state(false);
  let pendingKey = $state<string | null>(null);

  // Modifier-only keys that should not be captured as a binding
  const MODIFIER_KEYS = new Set(["Control", "Meta", "Alt", "Shift"]);
  // Keys that are suppressed while recording (they perform UI actions instead)
  const SUPPRESS_KEYS = new Set(["Escape", "Tab"]);

  function startRecording() {
    recording = true;
    pendingKey = null;
  }

  function cancelRecording() {
    recording = false;
    pendingKey = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();

    // Escape cancels recording
    if (e.key === "Escape") {
      cancelRecording();
      return;
    }

    // Ignore modifier-only presses and Tab
    if (MODIFIER_KEYS.has(e.key) || SUPPRESS_KEYS.has(e.key)) {
      return;
    }

    const key = eventToKey(e);
    pendingKey = key;
    recording = false;
    shortcutsStore.setBinding(action, key);
  }

  const hasConflict = $derived(shortcutsStore.hasConflict(action));
  const currentBinding = $derived(shortcutsStore.bindings[action]);
  const label = $derived(ACTION_LABELS[action]);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="keybinding-row"
  class:has-conflict={hasConflict}
  onkeydown={handleKeyDown}
>
  <span class="action-label">{label}</span>
  <div class="binding-controls">
    {#if recording}
      <span class="recording-hint">Press a key...</span>
      <button class="cancel-btn" onclick={cancelRecording} type="button">Cancel</button>
    {:else}
      <kbd class="key-badge" class:conflict={hasConflict}>{currentBinding}</kbd>
      <button class="record-btn" onclick={startRecording} type="button" title="Click to remap">
        Change
      </button>
    {/if}
  </div>
</div>

<style>
  .keybinding-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
    border: 1px solid transparent;
  }

  .keybinding-row.has-conflict {
    border-color: var(--warning);
    background: var(--warning-tint);
  }

  .action-label {
    font-size: 14px;
    color: var(--text-primary);
  }

  .binding-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .key-badge {
    display: inline-block;
    padding: 2px 8px;
    font-family: monospace;
    font-size: 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
  }

  .key-badge.conflict {
    border-color: var(--warning);
    color: var(--warning);
  }

  .recording-hint {
    font-size: 13px;
    color: var(--accent);
    font-style: italic;
  }

  .record-btn,
  .cancel-btn {
    background: var(--bg-tertiary);
    border: none;
    color: var(--text-secondary);
    padding: 4px 10px;
    font-size: 12px;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .record-btn:hover {
    background: var(--secondary-hover);
    color: var(--text-primary);
  }

  .cancel-btn {
    background: var(--accent-tint);
    color: var(--accent);
  }

  .cancel-btn:hover {
    background: var(--accent-tint-strong);
  }
</style>
