<script lang="ts">
  import { onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { playlistStore } from "../lib/stores/playlist.svelte";
  import { playerStore } from "../lib/stores/player.svelte";
  import { exportPlaylist } from "../lib/api/commands";
  import { formatDuration } from "../lib/utils/format";
  import type { Track } from "../lib/api/types";

  let newName = $state("");
  let editingName = $state(false);
  let renameValue = $state("");
  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let dragging = $state(false);

  onMount(() => {
    playlistStore.load();
  });

  async function createPlaylist() {
    if (!newName.trim()) return;
    const result = await playlistStore.create(newName.trim());
    newName = "";
    if (result) {
      await playlistStore.select(result.playlist.id);
    }
  }

  function startRename() {
    if (!playlistStore.selectedPlaylist) return;
    renameValue = playlistStore.selectedPlaylist.playlist.name;
    editingName = true;
  }

  async function finishRename() {
    if (!playlistStore.selectedPlaylist || !renameValue.trim()) {
      editingName = false;
      return;
    }
    await playlistStore.rename(playlistStore.selectedPlaylist.playlist.id, renameValue.trim());
    editingName = false;
  }

  async function removeTrack(track: Track) {
    if (!playlistStore.selectedPlaylist || track.id == null) return;
    await playlistStore.removeTracks(playlistStore.selectedPlaylist.playlist.id, [track.id]);
  }

  function playAll() {
    if (!playlistStore.selectedPlaylist) return;
    playerStore.playPlaylist(playlistStore.selectedPlaylist.tracks);
  }

  function playTrack(track: Track) {
    if (!playlistStore.selectedPlaylist) return;
    const tracks = playlistStore.selectedPlaylist.tracks;
    const index = tracks.findIndex((t) => t.file_path === track.file_path);
    playerStore.queue = tracks;
    playerStore.queueIndex = index >= 0 ? index : 0;
    playerStore.playPlaylist(tracks.slice(index >= 0 ? index : 0));
  }

  async function handleExport(format: string) {
    if (!playlistStore.selectedPlaylist) return;
    const ext = format === "m3u" ? "m3u" : "pls";
    const path = await save({
      title: `Export Playlist as ${format.toUpperCase()}`,
      defaultPath: `${playlistStore.selectedPlaylist.playlist.name}.${ext}`,
      filters: [{ name: `${format.toUpperCase()} Playlist`, extensions: [ext] }],
    });
    if (path) {
      await exportPlaylist(playlistStore.selectedPlaylist.playlist.id, format, path);
    }
  }

  function handlePointerDown(e: PointerEvent, index: number) {
    // Only start drag from the drag handle
    const target = e.target as HTMLElement;
    if (!target.closest(".drag-handle")) return;
    e.preventDefault();
    dragIndex = index;
    dragging = true;

    const onPointerMove = (me: PointerEvent) => {
      const el = document.elementFromPoint(me.clientX, me.clientY);
      if (el) {
        const row = el.closest("[data-track-index]") as HTMLElement | null;
        if (row) {
          dragOverIndex = Number(row.dataset.trackIndex);
        }
      }
    };

    const onPointerUp = async () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      dragging = false;

      if (dragIndex !== null && dragOverIndex !== null && dragIndex !== dragOverIndex && playlistStore.selectedPlaylist) {
        const tracks = [...playlistStore.selectedPlaylist.tracks];
        const [moved] = tracks.splice(dragIndex, 1);
        tracks.splice(dragOverIndex, 0, moved);
        const trackIds = tracks.map((t) => t.id!).filter((id): id is number => id != null);
        await playlistStore.reorder(playlistStore.selectedPlaylist.playlist.id, trackIds);
      }
      dragIndex = null;
      dragOverIndex = null;
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }
</script>

<div class="playlists-page">
  <div class="playlist-sidebar">
    <div class="sidebar-header">
      <h2>Playlists</h2>
    </div>
    <div class="create-row">
      <input
        type="text"
        placeholder="New playlist..."
        bind:value={newName}
        onkeydown={(e) => { if (e.key === "Enter") createPlaylist(); }}
      />
      <button class="primary" onclick={createPlaylist} disabled={!newName.trim()}>Create</button>
    </div>
    <div class="playlist-list">
      {#each playlistStore.playlists as pl}
        <button
          class="playlist-item"
          class:active={playlistStore.selectedPlaylist?.playlist.id === pl.id}
          onclick={() => playlistStore.select(pl.id)}
        >
          <span class="pl-name">{pl.name}</span>
        </button>
      {/each}
      {#if playlistStore.playlists.length === 0 && !playlistStore.loading}
        <div class="empty-hint">No playlists yet</div>
      {/if}
    </div>
  </div>

  <div class="playlist-detail">
    {#if playlistStore.selectedPlaylist}
      <div class="detail-header">
        {#if editingName}
          <input
            class="rename-input"
            type="text"
            bind:value={renameValue}
            onkeydown={(e) => { if (e.key === "Enter") finishRename(); if (e.key === "Escape") editingName = false; }}
            onblur={finishRename}
          />
        {:else}
          <h2 class="detail-title" ondblclick={startRename}>{playlistStore.selectedPlaylist.playlist.name}</h2>
        {/if}
        <div class="detail-actions">
          <button class="secondary" onclick={playAll} disabled={playlistStore.selectedPlaylist.tracks.length === 0}>
            Play All
          </button>
          <button class="secondary" onclick={() => handleExport("m3u")}>Export M3U</button>
          <button class="secondary" onclick={() => handleExport("pls")}>Export PLS</button>
          <button class="danger-btn" onclick={() => { if (playlistStore.selectedPlaylist) playlistStore.remove(playlistStore.selectedPlaylist.playlist.id); }}>
            Delete
          </button>
        </div>
      </div>
      <div class="detail-info">
        {playlistStore.selectedPlaylist.tracks.length} tracks
      </div>
      <div class="track-list">
        {#each playlistStore.selectedPlaylist.tracks as track, i}
          <div
            class="playlist-track-row"
            class:drag-over={dragOverIndex === i && dragIndex !== null && dragIndex !== i}
            class:dragging-source={dragIndex === i && dragging}
            class:now-playing={playerStore.currentTrack?.file_path === track.file_path}
            data-track-index={i}
            onpointerdown={(e) => handlePointerDown(e, i)}
            role="listitem"
          >
            <span class="drag-handle">&#x2630;</span>
            <button class="track-play-btn" onclick={() => playTrack(track)} title="Play">&#9654;</button>
            <span class="track-pos">{i + 1}</span>
            <span class="track-title">{track.title ?? track.relative_path}</span>
            <span class="track-artist">{track.artist ?? ""}</span>
            <span class="track-duration">{formatDuration(track.duration_secs)}</span>
            <button class="remove-btn" onclick={() => removeTrack(track)} title="Remove from playlist">x</button>
          </div>
        {/each}
        {#if playlistStore.selectedPlaylist.tracks.length === 0}
          <div class="empty-hint">No tracks. Add tracks from the Library view.</div>
        {/if}
      </div>
    {:else}
      <div class="no-selection">
        <p>Select a playlist or create a new one</p>
      </div>
    {/if}
  </div>
</div>

{#if playlistStore.error}
  <div class="error-bar">{playlistStore.error}</div>
{/if}

<style>
  .playlists-page {
    display: flex;
    height: 100%;
    gap: 0;
  }

  .playlist-sidebar {
    width: 260px;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    padding: 0 12px 12px;
  }

  .sidebar-header h2 {
    font-size: 16px;
    font-weight: 600;
  }

  .create-row {
    display: flex;
    gap: 6px;
    padding: 0 12px 12px;
  }

  .create-row input {
    flex: 1;
    padding: 6px 8px;
    font-size: 13px;
  }

  .create-row button {
    padding: 6px 12px;
    font-size: 13px;
  }

  .playlist-list {
    flex: 1;
    overflow-y: auto;
  }

  .playlist-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 10px 12px;
    text-align: left;
    background: none;
    border: none;
    border-radius: 0;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
  }

  .playlist-item:hover {
    background: var(--bg-tertiary);
  }

  .playlist-item.active {
    background: var(--bg-tertiary);
    font-weight: 600;
  }

  .pl-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .playlist-detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    padding: 0 16px;
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 8px;
    flex-wrap: wrap;
  }

  .detail-title {
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
  }

  .rename-input {
    font-size: 18px;
    font-weight: 600;
    padding: 4px 8px;
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .detail-actions {
    display: flex;
    gap: 6px;
    margin-left: auto;
  }

  .detail-actions button {
    padding: 6px 12px;
    font-size: 12px;
  }

  .danger-btn {
    background: var(--danger);
    color: white;
  }

  .danger-btn:hover {
    opacity: 0.85;
  }

  .detail-info {
    font-size: 13px;
    color: var(--text-secondary);
    padding-bottom: 12px;
  }

  .track-list {
    flex: 1;
    overflow-y: auto;
  }

  .playlist-track-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .playlist-track-row:hover {
    background: var(--bg-secondary);
  }

  .playlist-track-row.drag-over {
    border-top: 2px solid var(--accent);
  }

  .playlist-track-row.dragging-source {
    opacity: 0.4;
  }

  .playlist-track-row.now-playing .track-title {
    color: var(--accent);
    font-weight: 600;
  }

  .drag-handle {
    cursor: grab;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 0 4px;
    opacity: 0.5;
    user-select: none;
    touch-action: none;
  }

  .playlist-track-row:hover .drag-handle {
    opacity: 1;
  }

  .track-play-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    padding: 2px 4px;
    opacity: 0;
    transition: opacity 0.15s;
    cursor: pointer;
  }

  .playlist-track-row:hover .track-play-btn {
    opacity: 1;
  }

  .track-play-btn:hover {
    color: var(--accent);
  }

  .track-pos {
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

  .track-artist {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track-duration {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .remove-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 2px 6px;
    opacity: 0;
    transition: opacity 0.15s;
    cursor: pointer;
  }

  .playlist-track-row:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    color: var(--danger);
  }

  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }

  .empty-hint {
    padding: 16px 12px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .error-bar {
    position: fixed;
    bottom: 60px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--danger);
    color: white;
    padding: 8px 16px;
    border-radius: var(--radius);
    font-size: 13px;
    z-index: 200;
  }
</style>
