<script lang="ts">
  import type { FolderNode, Track } from "../api/types";
  import TrackRow from "./TrackRow.svelte";
  import PlaylistPicker from "./PlaylistPicker.svelte";

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

  let showPicker = $state(false);
  let pickerTrackIds = $state<number[]>([]);

  function handleAddToPlaylist(track: Track) {
    if (track.id != null) {
      pickerTrackIds = [track.id];
      showPicker = true;
    }
  }

  function toggleFolder(path: string) {
    const next = new Set(expandedFolders);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    expandedFolders = next;
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
    <TrackRow
      {track}
      siblingTracks={node.tracks}
      titleFallback={track.title ? undefined : track.relative_path.split("/").pop() ?? track.relative_path}
      onPlay={onPlayTrack}
      onEdit={onEditTrack}
      onAddToPlaylist={handleAddToPlaylist}
    />
  {/each}
{/snippet}

<div class="tree-view">
  {@render folderContent(root, 0)}
</div>

{#if showPicker}
  <PlaylistPicker trackIds={pickerTrackIds} onClose={() => showPicker = false} />
{/if}

<style>
  .tree-view {
    overflow-y: auto;
    padding: 8px;
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
</style>
