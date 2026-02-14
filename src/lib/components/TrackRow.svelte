<script lang="ts">
  import type { Track } from "../api/types";
  import { playerStore } from "../stores/player.svelte";
  import { formatDuration, formatSize } from "../utils/format";

  let {
    track,
    siblingTracks,
    titleFallback,
    onPlay,
    onEdit,
  }: {
    track: Track;
    siblingTracks: Track[];
    titleFallback?: string;
    onPlay?: (track: Track, siblingTracks: Track[]) => void;
    onEdit?: (track: Track) => void;
  } = $props();

  let isPlaying = $derived(playerStore.currentTrack?.file_path === track.file_path);
</script>

<div class="track-row" class:now-playing={isPlaying}>
  {#if onPlay}
    <button
      class="track-play-btn"
      onclick={(e) => { e.stopPropagation(); onPlay(track, siblingTracks); }}
      title="Play track"
    >&#9654;</button>
  {/if}
  <button
    class="track-node"
    onclick={() => onEdit?.(track)}
    title="Edit track metadata"
  >
    <span class="track-num">{track.track_number ?? "-"}</span>
    <span class="track-title">{track.title ?? titleFallback ?? track.relative_path}</span>
    <span class="track-duration">{formatDuration(track.duration_secs)}</span>
    <span class="track-format">{track.format.toUpperCase()}</span>
    <span class="track-size">{formatSize(track.file_size)}</span>
  </button>
</div>

<style>
  .track-row {
    display: flex;
    align-items: center;
    border-radius: var(--radius);
  }

  .track-row.now-playing {
    background-color: rgba(233, 69, 96, 0.1);
  }

  .track-row.now-playing .track-title {
    color: var(--accent);
    font-weight: 600;
  }

  .track-play-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    padding: 4px 4px 4px 8px;
    opacity: 0;
    transition: opacity 0.15s;
    flex-shrink: 0;
  }

  .track-row:hover .track-play-btn {
    opacity: 1;
  }

  .track-play-btn:hover {
    color: var(--accent);
  }

  .track-node {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 8px;
    border-radius: var(--radius);
    font-size: 13px;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
  }

  .track-node:hover {
    background-color: var(--bg-secondary);
  }

  .track-num {
    color: var(--text-secondary);
    width: 24px;
    text-align: right;
    flex-shrink: 0;
  }

  .track-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track-duration, .track-format, .track-size {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .track-format {
    background: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
  }
</style>
