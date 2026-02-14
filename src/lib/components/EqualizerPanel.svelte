<script lang="ts">
  import { equalizerStore, EQ_PRESETS } from "../stores/equalizer.svelte";

  let { onClose }: { onClose: () => void } = $props();

  const freqLabels = ["32", "64", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];

  function onPresetChange(e: Event) {
    const name = (e.target as HTMLSelectElement).value;
    equalizerStore.applyPreset(name);
    equalizerStore.saveState();
  }

  function onBandInput(index: number, e: Event) {
    const value = +(e.target as HTMLInputElement).value;
    equalizerStore.setBandGain(index, value);
    equalizerStore.saveState();
  }

  function onToggleEnabled() {
    equalizerStore.toggleEnabled();
    equalizerStore.saveState();
  }

  function formatGain(gain: number): string {
    if (gain > 0) return `+${gain}`;
    if (gain === 0) return "0";
    return String(gain);
  }
</script>

<div class="eq-panel">
  <div class="eq-header">
    <div class="eq-controls">
      <select class="preset-select" value={equalizerStore.activePreset} onchange={onPresetChange}>
        {#each EQ_PRESETS as preset}
          <option value={preset.name}>{preset.name}</option>
        {/each}
        {#if equalizerStore.activePreset === "custom"}
          <option value="custom">Custom</option>
        {/if}
      </select>
      <button
        class="toggle-btn"
        class:disabled={!equalizerStore.enabled}
        onclick={onToggleEnabled}
        title={equalizerStore.enabled ? "Disable EQ" : "Enable EQ"}
      >
        {equalizerStore.enabled ? "ON" : "OFF"}
      </button>
    </div>
    <button class="close-btn" onclick={onClose}>x</button>
  </div>
  <div class="eq-body">
    <div class="bands-container">
      {#each equalizerStore.bands as band, i}
        <div class="band">
          <span class="gain-label">{formatGain(band.gain)}</span>
          <input
            type="range"
            class="band-slider"
            min="-12"
            max="12"
            step="0.5"
            value={band.gain}
            oninput={(e: Event) => onBandInput(i, e)}
            disabled={!equalizerStore.enabled}
          />
          <span class="freq-label">{freqLabels[i]}</span>
        </div>
      {/each}
    </div>
    <div class="scale-ref">
      <span>-12</span>
      <span>0 dB</span>
      <span>+12</span>
    </div>
  </div>
</div>

<style>
  .eq-panel {
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    height: 180px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .eq-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .eq-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preset-select {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 2px 8px;
    font-size: 12px;
    text-transform: capitalize;
  }

  .toggle-btn {
    background: var(--bg-tertiary);
    color: var(--accent);
    border: none;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .toggle-btn:hover {
    background: var(--accent-tint-strong);
  }

  .toggle-btn.disabled {
    color: var(--text-secondary);
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

  .eq-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 4px 12px 2px;
    min-height: 0;
  }

  .bands-container {
    display: flex;
    justify-content: space-around;
    align-items: stretch;
    flex: 1;
    gap: 2px;
    min-height: 0;
  }

  .band {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
    min-width: 0;
    gap: 2px;
  }

  .gain-label {
    font-size: 10px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
    height: 14px;
    line-height: 14px;
  }

  .band-slider {
    writing-mode: vertical-lr;
    direction: rtl;
    -webkit-appearance: none;
    appearance: none;
    flex: 1;
    width: 20px;
    min-height: 0;
    background: transparent;
    outline: none;
    border: none;
    padding: 0;
    margin: 0;
  }

  .band-slider::-webkit-slider-runnable-track {
    width: 4px;
    background: var(--border);
    border-radius: 2px;
  }

  .band-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    background: var(--accent);
    border-radius: 50%;
    cursor: pointer;
    margin-left: -4px;
  }

  .band-slider:disabled::-webkit-slider-thumb {
    background: var(--text-secondary);
    cursor: default;
  }

  .freq-label {
    font-size: 9px;
    color: var(--text-secondary);
    flex-shrink: 0;
    height: 12px;
    line-height: 12px;
  }

  .scale-ref {
    display: flex;
    justify-content: space-between;
    font-size: 9px;
    color: var(--text-secondary);
    padding: 2px 4px 0;
    flex-shrink: 0;
  }
</style>
