<script lang="ts">
  import type { FolderNode, Track } from "../api/types";
  import { playerStore } from "../stores/player.svelte";

  let {
    root,
    onEditTrack,
    onPlayTrack,
    onPlayFolder,
  }: {
    root: FolderNode;
    onEditTrack?: (track: Track) => void;
    onPlayTrack?: (track: Track, albumTracks: Track[]) => void;
    onPlayFolder?: (tracks: Track[]) => void;
  } = $props();

  let expandedFolders = $state<Set<string>>(new Set());

  function toggleFolder(path: string) {
    const next = new Set(expandedFolders);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    expandedFolders = next;
  }

  function formatDuration(secs: number | null): string {
    if (secs == null) return "--:--";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getAllTracks(node: FolderNode): Track[] {
    const tracks: Track[] = [...node.tracks];
    for (const child of node.children) {
      tracks.push(...getAllTracks(child));
    }
    return tracks;
  }
</script>

{#snippet folderContent(node: FolderNode, depth: number)}
  {#each node.children as child}
    <div class="folder-node">
      <div class="folder-header">
        <button class="tree-toggle" onclick={() => toggleFolder(child.path)}>
          <span class="chevron" class:expanded={expandedFolders.has(child.path)}>&#9654;</span>
          <span class="folder-icon">&#128193;</span>
          <span class="folder-name">{child.name}</span>
          <span class="count">{getAllTracks(child).length} track{getAllTracks(child).length !== 1 ? "s" : ""}</span>
        </button>
        {#if onPlayFolder}
          <button
            class="action-btn play-folder-btn"
            onclick={(e) => { e.stopPropagation(); onPlayFolder(getAllTracks(child)); }}
            title="Play folder"
          >&#9654;</button>
        {/if}
      </div>

      {#if expandedFolders.has(child.path)}
        <div class="children">
          {@render folderContent(child, depth + 1)}
        </div>
      {/if}
    </div>
  {/each}

  {#each node.tracks as track}
    {@const isPlaying = playerStore.currentTrack?.file_path === track.file_path}
    <div class="track-row" class:now-playing={isPlaying}>
      {#if onPlayTrack}
        <button
          class="track-play-btn"
          onclick={(e) => { e.stopPropagation(); onPlayTrack(track, node.tracks); }}
          title="Play track"
        >&#9654;</button>
      {/if}
      <button
        class="track-node"
        onclick={() => onEditTrack?.(track)}
        title="Edit track metadata"
      >
        <span class="track-num">{track.track_number ?? "-"}</span>
        <span class="track-title">{track.title ?? track.relative_path.split("/").pop()}</span>
        <span class="track-duration">{formatDuration(track.duration_secs)}</span>
        <span class="track-format">{track.format.toUpperCase()}</span>
        <span class="track-size">{formatSize(track.file_size)}</span>
      </button>
    </div>
  {/each}
{/snippet}

<div class="tree-view">
  {@render folderContent(root, 0)}
</div>

<style>
  .tree-view {
    overflow-y: auto;
    padding: 8px;
  }

  .tree-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 6px 8px;
    border-radius: var(--radius);
    text-align: left;
    font-size: 14px;
  }

  .tree-toggle:hover {
    background-color: var(--bg-tertiary);
  }

  .chevron {
    font-size: 10px;
    transition: transform 0.15s;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .folder-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .folder-header {
    display: flex;
    align-items: center;
  }

  .folder-header .tree-toggle {
    flex: 1;
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 4px 8px;
    border-radius: var(--radius);
    opacity: 0;
    transition: opacity 0.15s;
  }

  .folder-header:hover .action-btn {
    opacity: 1;
  }

  .action-btn:hover {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .play-folder-btn {
    font-size: 12px;
  }

  .folder-name {
    font-weight: 500;
  }

  .count {
    color: var(--text-secondary);
    font-size: 12px;
    margin-left: auto;
  }

  .children {
    padding-left: 20px;
  }

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
