<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Track } from "../lib/api/types";
  import TreeView from "../lib/components/TreeView.svelte";
  import MetadataEditor from "../lib/components/MetadataEditor.svelte";
  import AlbumEditor from "../lib/components/AlbumEditor.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";
  import { libraryStore } from "../lib/stores/library.svelte";
  import { playerStore } from "../lib/stores/player.svelte";

  let editingTrack = $state<Track | null>(null);
  let editingAlbum = $state<{ tracks: Track[]; albumName: string; artistName: string } | null>(null);

  async function pickDirectory() {
    const selected = await open({ directory: true, multiple: false, title: "Select Music Directory" });
    if (selected && typeof selected === "string") {
      await libraryStore.scan(selected);
    }
  }

  let searchTimeout: ReturnType<typeof setTimeout>;
  function onSearchInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => libraryStore.search(value), 300);
  }

  function handleEditTrack(track: Track) {
    editingTrack = track;
  }

  function handleEditAlbum(tracks: Track[], albumName: string, artistName: string) {
    editingAlbum = { tracks, albumName, artistName };
  }

  function handlePlayTrack(track: Track, albumTracks: Track[]) {
    playerStore.playTrack(track, albumTracks);
  }

  function handlePlayAlbum(tracks: Track[]) {
    playerStore.playAlbum(tracks);
  }

  async function handleTrackSaved() {
    editingTrack = null;
    if (libraryStore.libraryRoot) {
      await libraryStore.loadTree(libraryStore.libraryRoot);
    }
  }

  async function handleAlbumSaved() {
    editingAlbum = null;
    if (libraryStore.libraryRoot) {
      await libraryStore.loadTree(libraryStore.libraryRoot);
    }
  }
</script>

<div class="library-page">
  <div class="library-header">
    <h2>Music Library</h2>
    <div class="header-actions">
      <input
        type="text"
        placeholder="Search tracks..."
        oninput={onSearchInput}
        class="search-input"
      />
      {#if libraryStore.tree && !libraryStore.scanning}
        <button class="secondary" onclick={() => libraryStore.scan(libraryStore.libraryRoot)}>
          Rescan
        </button>
      {/if}
      <button class="primary" onclick={pickDirectory} disabled={libraryStore.scanning}>
        {libraryStore.scanning ? "Scanning..." : "Open Directory"}
      </button>
    </div>
  </div>

  {#if libraryStore.error}
    <div class="error-banner">{libraryStore.error}</div>
  {/if}

  {#if libraryStore.scanning}
    <div class="scan-progress">
      <ProgressBar
        value={libraryStore.scanProgress.dirsCompleted}
        max={libraryStore.scanProgress.dirsTotal}
        label="Scanning: {libraryStore.scanProgress.currentFile}"
      />
      <p class="scan-count">
        {libraryStore.scanProgress.filesProcessed} files processed
      </p>
    </div>
  {/if}

  {#if libraryStore.searchQuery.length >= 2 && libraryStore.searchResults.length > 0}
    <div class="search-results">
      <h3>Search Results ({libraryStore.searchResults.length})</h3>
      <div class="results-list">
        {#each libraryStore.searchResults as track}
          <div class="result-item">
            <button class="result-play-btn" onclick={() => handlePlayTrack(track, [track])} title="Play">&#9654;</button>
            <span class="result-title">{track.title ?? track.relative_path}</span>
            <span class="result-artist">{track.artist ?? "Unknown"}</span>
            <span class="result-album">{track.album ?? "Unknown"}</span>
          </div>
        {/each}
      </div>
    </div>
  {:else if libraryStore.tree}
    <div class="library-info">
      <span class="root-path">{libraryStore.tree.root}</span>
      <span class="track-count">{libraryStore.tree.total_tracks} tracks</span>
      <span class="artist-count">{libraryStore.tree.artists.length} artists</span>
    </div>
    <TreeView
      artists={libraryStore.tree.artists}
      onEditTrack={handleEditTrack}
      onEditAlbum={handleEditAlbum}
      onPlayTrack={handlePlayTrack}
      onPlayAlbum={handlePlayAlbum}
    />
  {:else if !libraryStore.scanning}
    <div class="empty-state">
      <p>No library loaded. Click "Open Directory" to scan a music folder.</p>
    </div>
  {/if}

  {#if editingTrack}
    <MetadataEditor
      track={editingTrack}
      onSave={handleTrackSaved}
      onClose={() => (editingTrack = null)}
    />
  {/if}

  {#if editingAlbum}
    <AlbumEditor
      tracks={editingAlbum.tracks}
      albumName={editingAlbum.albumName}
      artistName={editingAlbum.artistName}
      onSave={handleAlbumSaved}
      onClose={() => (editingAlbum = null)}
    />
  {/if}
</div>

<style>
  .library-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 12px;
  }

  .library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .library-header h2 {
    font-size: 20px;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .search-input {
    width: 220px;
  }

  .error-banner {
    background: rgba(233, 69, 96, 0.15);
    color: var(--danger);
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .scan-progress {
    padding: 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
  }

  .scan-count {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 8px;
  }

  .library-info {
    display: flex;
    gap: 16px;
    font-size: 13px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .root-path {
    font-family: monospace;
    background: var(--bg-secondary);
    padding: 2px 8px;
    border-radius: var(--radius);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--text-secondary);
  }

  .search-results {
    flex: 1;
    overflow-y: auto;
  }

  .search-results h3 {
    font-size: 14px;
    margin-bottom: 8px;
  }

  .result-item {
    display: flex;
    gap: 16px;
    padding: 6px 8px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .result-item:hover {
    background: var(--bg-secondary);
  }

  .result-play-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    padding: 2px 6px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .result-item:hover .result-play-btn {
    opacity: 1;
  }

  .result-play-btn:hover {
    color: var(--accent);
  }

  .result-title { flex: 2; }
  .result-artist { flex: 1; color: var(--text-secondary); }
  .result-album { flex: 1; color: var(--text-secondary); }
</style>
