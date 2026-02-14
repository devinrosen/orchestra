<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Track, LibraryViewMode } from "../lib/api/types";
  import TreeView from "../lib/components/TreeView.svelte";
  import AlbumListView from "../lib/components/AlbumListView.svelte";
  import GenreTreeView from "../lib/components/GenreTreeView.svelte";
  import FolderTreeView from "../lib/components/FolderTreeView.svelte";
  import MetadataEditor from "../lib/components/MetadataEditor.svelte";
  import AlbumEditor from "../lib/components/AlbumEditor.svelte";
  import MetadataReport from "../lib/components/MetadataReport.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";
  import { libraryStore } from "../lib/stores/library.svelte";
  import { playerStore } from "../lib/stores/player.svelte";

  const searchPlaceholders: Record<LibraryViewMode, string> = {
    artist: "Search artists...",
    album: "Search albums...",
    genre: "Search genres...",
    folder: "Search folders...",
  };

  let editingTrack = $state<Track | null>(null);
  let editingAlbum = $state<{ tracks: Track[]; albumName: string; artistName: string } | null>(null);
  let showMetadataReport = $state(false);

  const viewModes: { mode: LibraryViewMode; label: string }[] = [
    { mode: "artist", label: "Artists" },
    { mode: "album", label: "Albums" },
    { mode: "genre", label: "Genres" },
    { mode: "folder", label: "Folders" },
  ];

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

  function handleReportEditTrack(track: Track) {
    showMetadataReport = false;
    editingTrack = track;
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

  function infoSummary(): string {
    const tree = libraryStore.tree;
    if (!tree) return "";
    const mode = libraryStore.viewMode;
    if (mode === "artist") return `${libraryStore.filteredArtists.length} artists`;
    if (mode === "album") return `${libraryStore.filteredAlbumEntries.length} albums`;
    if (mode === "genre") return `${libraryStore.filteredGenreNodes.length} genres`;
    return "";
  }

  function isSearchActive(): boolean {
    return libraryStore.searchQuery.length >= 2;
  }

  function hasNoResults(): boolean {
    if (!isSearchActive()) return false;
    const mode = libraryStore.viewMode;
    if (mode === "artist") return libraryStore.filteredArtists.length === 0;
    if (mode === "album") return libraryStore.filteredAlbumEntries.length === 0;
    if (mode === "genre") return libraryStore.filteredGenreNodes.length === 0;
    if (mode === "folder") return libraryStore.filteredFolderTree === null;
    return false;
  }

  const noResultsMessages: Record<LibraryViewMode, string> = {
    artist: "No matching artists",
    album: "No matching albums",
    genre: "No matching genres",
    folder: "No matching folders",
  };
</script>

<div class="library-page">
  <div class="library-header">
    <h2>Music Library</h2>
    <div class="header-actions">
      <input
        type="text"
        placeholder={searchPlaceholders[libraryStore.viewMode]}
        oninput={onSearchInput}
        class="search-input"
      />
      {#if libraryStore.tree && !libraryStore.scanning}
        <button class="secondary report-btn" onclick={() => (showMetadataReport = true)}>
          Metadata Report
          {#if libraryStore.incompleteCount > 0}
            <span class="report-badge">{libraryStore.incompleteCount}</span>
          {/if}
        </button>
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

  {#if libraryStore.tree}
    <div class="library-info">
      <span class="root-path">{libraryStore.tree.root}</span>
      <span class="track-count">{libraryStore.tree.total_tracks} tracks</span>
      <span class="info-summary">{infoSummary()}</span>
    </div>

    <div class="view-mode-toggle">
      {#each viewModes as { mode, label }}
        <button
          class="mode-btn"
          class:active={libraryStore.viewMode === mode}
          onclick={() => libraryStore.setViewMode(mode)}
        >
          {label}
        </button>
      {/each}
    </div>

    {#if hasNoResults()}
      <div class="no-results">{noResultsMessages[libraryStore.viewMode]}</div>
    {:else if libraryStore.viewMode === "artist"}
      <TreeView
        artists={libraryStore.filteredArtists}
        onEditTrack={handleEditTrack}
        onEditAlbum={handleEditAlbum}
        onPlayTrack={handlePlayTrack}
        onPlayAlbum={handlePlayAlbum}
      />
    {:else if libraryStore.viewMode === "album"}
      <AlbumListView
        albums={libraryStore.filteredAlbumEntries}
        onEditTrack={handleEditTrack}
        onEditAlbum={handleEditAlbum}
        onPlayTrack={handlePlayTrack}
        onPlayAlbum={handlePlayAlbum}
      />
    {:else if libraryStore.viewMode === "genre"}
      <GenreTreeView
        genres={libraryStore.filteredGenreNodes}
        onEditTrack={handleEditTrack}
        onEditAlbum={handleEditAlbum}
        onPlayTrack={handlePlayTrack}
        onPlayAlbum={handlePlayAlbum}
      />
    {:else if libraryStore.viewMode === "folder" && libraryStore.filteredFolderTree}
      <FolderTreeView
        root={libraryStore.filteredFolderTree}
        onEditTrack={handleEditTrack}
        onPlayTrack={handlePlayTrack}
        onPlayFolder={handlePlayAlbum}
      />
    {/if}
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

  {#if showMetadataReport && libraryStore.libraryRoot}
    <MetadataReport
      libraryRoot={libraryStore.libraryRoot}
      onEditTrack={handleReportEditTrack}
      onClose={() => (showMetadataReport = false)}
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
    background: var(--accent-tint-strong);
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

  .view-mode-toggle {
    display: flex;
    gap: 0;
    flex-shrink: 0;
    background: var(--bg-secondary);
    border-radius: var(--radius);
    padding: 2px;
    width: fit-content;
  }

  .mode-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    padding: 5px 14px;
    font-size: 13px;
    border-radius: calc(var(--radius) - 2px);
    cursor: pointer;
    transition: all 0.15s;
  }

  .mode-btn:hover {
    color: var(--text-primary);
  }

  .mode-btn.active {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-weight: 500;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--text-secondary);
  }

  .no-results {
    padding: 24px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .report-btn {
    position: relative;
  }

  .report-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--danger);
    color: var(--on-accent);
    font-size: 10px;
    font-weight: 600;
    min-width: 18px;
    height: 18px;
    border-radius: 9px;
    padding: 0 4px;
    margin-left: 4px;
  }
</style>
