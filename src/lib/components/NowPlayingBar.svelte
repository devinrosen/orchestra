<script lang="ts">
  import { onMount } from "svelte";
  import { playerStore } from "../stores/player.svelte";
  import QueuePanel from "./QueuePanel.svelte";

  let showQueue = $state(false);

  let audioEl: HTMLAudioElement;

  onMount(() => {
    playerStore.bindAudio(audioEl);
  });

  function formatTime(secs: number): string {
    if (!secs || !isFinite(secs)) return "0:00";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function onSeek(e: Event) {
    const value = +(e.target as HTMLInputElement).value;
    playerStore.seek(value);
  }

  function onVolume(e: Event) {
    const value = +(e.target as HTMLInputElement).value;
    playerStore.setVolume(value);
  }
</script>

<div class="now-playing-bar">
  <audio bind:this={audioEl}></audio>

  <div class="track-info">
    {#if playerStore.artwork}
      <img
        class="album-art"
        src="data:{playerStore.artwork.mime_type};base64,{playerStore.artwork.data}"
        alt="Album art"
      />
    {:else}
      <div class="album-art placeholder">
        <span>&#9835;</span>
      </div>
    {/if}
    <div class="track-text">
      <div class="track-title">{playerStore.currentTrack?.title ?? "Unknown"}</div>
      <div class="track-artist">{playerStore.currentTrack?.artist ?? "Unknown"}</div>
    </div>
  </div>

  <div class="controls">
    <div class="control-buttons">
      <button class="ctrl-btn" onclick={() => playerStore.previous()} title="Previous">
        &#9198;
      </button>
      <button class="ctrl-btn play-btn" onclick={() => playerStore.togglePlayPause()} title={playerStore.playing ? "Pause" : "Play"}>
        {playerStore.playing ? "\u23F8" : "\u25B6"}
      </button>
      <button class="ctrl-btn" onclick={() => playerStore.next()} disabled={!playerStore.hasNext} title="Next">
        &#9197;
      </button>
    </div>
    <div class="seek-row">
      <span class="time">{formatTime(playerStore.currentTime)}</span>
      <input
        type="range"
        class="seek-bar"
        min="0"
        max={playerStore.duration || 0}
        step="0.5"
        value={playerStore.currentTime}
        oninput={onSeek}
      />
      <span class="time">{formatTime(playerStore.duration)}</span>
    </div>
  </div>

  {#if playerStore.error}
    <div class="error-section">
      <span class="error-icon">&#9888;</span>
      <span class="error-text">{playerStore.error}</span>
    </div>
  {/if}

  <div class="volume-section">
    <button class="ctrl-btn eq-btn" class:active={playerStore.equalizerActive} onclick={() => playerStore.toggleEqualizer()} title="Equalizer">
      EQ
    </button>
    <button class="ctrl-btn viz-btn" class:active={playerStore.visualizerActive} onclick={() => playerStore.toggleVisualizer()} title="Visualizer">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor"><rect x="1" y="6" width="3" height="7" rx="0.5"/><rect x="5.5" y="2" width="3" height="11" rx="0.5"/><rect x="10" y="4" width="3" height="9" rx="0.5"/></svg>
    </button>
    <button class="ctrl-btn queue-btn" class:active={showQueue} onclick={() => showQueue = !showQueue} title="Play Queue">
      &#x2630;
    </button>
    <button class="ctrl-btn volume-btn" onclick={() => playerStore.setMuted(!playerStore.muted)} title={playerStore.muted ? "Unmute" : "Mute"}>
      {playerStore.muted || playerStore.volume === 0 ? "\u{1F507}" : "\u{1F509}"}
    </button>
    <input
      type="range"
      class="volume-bar"
      min="0"
      max="1"
      step="0.02"
      value={playerStore.muted ? 0 : playerStore.volume}
      oninput={onVolume}
    />
  </div>
</div>

{#if showQueue}
  <QueuePanel onClose={() => showQueue = false} />
{/if}

<style>
  .now-playing-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 16px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
    height: 72px;
  }

  .track-info {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 200px;
    max-width: 280px;
    flex-shrink: 0;
  }

  .album-art {
    width: 48px;
    height: 48px;
    border-radius: 4px;
    object-fit: cover;
    flex-shrink: 0;
  }

  .album-art.placeholder {
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
    color: var(--text-secondary);
  }

  .track-text {
    overflow: hidden;
  }

  .track-title {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-artist {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .controls {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .control-buttons {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .ctrl-btn {
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 16px;
    padding: 4px;
    border-radius: 50%;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ctrl-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
  }

  .ctrl-btn:disabled {
    color: var(--text-secondary);
    opacity: 0.4;
  }

  .play-btn {
    font-size: 20px;
    width: 36px;
    height: 36px;
    background: var(--accent);
    color: var(--on-accent);
  }

  .play-btn:hover {
    background: var(--accent-hover);
  }

  .seek-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 500px;
  }

  .time {
    font-size: 11px;
    color: var(--text-secondary);
    min-width: 36px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .seek-bar {
    flex: 1;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    outline: none;
    border: none;
    padding: 0;
  }

  .seek-bar::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    background: var(--accent);
    border-radius: 50%;
    cursor: pointer;
  }

  .volume-section {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    min-width: 120px;
  }

  .eq-btn {
    font-size: 11px;
    font-weight: 700;
  }

  .eq-btn.active {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .viz-btn {
    font-size: 14px;
  }

  .viz-btn svg {
    width: 14px;
    height: 14px;
  }

  .viz-btn.active {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .queue-btn {
    font-size: 14px;
  }

  .queue-btn.active {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .volume-btn {
    font-size: 14px;
  }

  .volume-bar {
    width: 80px;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    outline: none;
    border: none;
    padding: 0;
  }

  .volume-bar::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 10px;
    height: 10px;
    background: var(--text-secondary);
    border-radius: 50%;
    cursor: pointer;
  }

  .error-section {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--warning-tint-strong);
    padding: 4px 10px;
    border-radius: var(--radius);
    flex-shrink: 0;
    max-width: 200px;
  }

  .error-icon {
    color: var(--warning);
    font-size: 14px;
    flex-shrink: 0;
  }

  .error-text {
    color: var(--warning);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
