<script lang="ts">
  import type { Track, ArtistNode, AlbumEntry } from "../lib/api/types";
  import { favoritesStore } from "../lib/stores/favorites.svelte";
  import { libraryStore } from "../lib/stores/library.svelte";
  import { playerStore } from "../lib/stores/player.svelte";
  import TreeView from "../lib/components/TreeView.svelte";
  import AlbumListView from "../lib/components/AlbumListView.svelte";
  import TrackRow from "../lib/components/TrackRow.svelte";
  import MetadataEditor from "../lib/components/MetadataEditor.svelte";
  import AlbumEditor from "../lib/components/AlbumEditor.svelte";

  let expandedSections = $state<Set<string>>(new Set(["artists", "albums", "tracks"]));
  let editingTrack = $state<Track | null>(null);
  let editingAlbum = $state<{ tracks: Track[]; albumName: string; artistName: string } | null>(null);

  let favoriteArtistNodes: ArtistNode[] = $derived(
    libraryStore.tree
      ? libraryStore.tree.artists.filter(a => favoritesStore.isFavorite('artist', a.name))
      : []
  );

  let favoriteAlbumEntries: AlbumEntry[] = $derived(
    libraryStore.tree
      ? libraryStore.tree.artists.flatMap(artist =>
          artist.albums
            .filter(album => favoritesStore.isFavorite('album', artist.name + '\0' + album.name))
            .map(album => ({ name: album.name, artist: artist.name, year: album.year, tracks: album.tracks }))
        )
      : []
  );

  let favoriteTracks: Track[] = $derived(
    libraryStore.allTracks.filter(t => t.id != null && favoritesStore.isFavorite('track', String(t.id)))
  );

  function toggleSection(section: string) {
    const next = new Set(expandedSections);
    if (next.has(section)) next.delete(section);
    else next.add(section);
    expandedSections = next;
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

<div class="favorites-page">
  <h2>Favorites</h2>

  {#if !libraryStore.tree}
    <div class="loading">Library not loaded</div>
  {:else}
    <div class="fav-section">
      <button class="section-header" onclick={() => toggleSection("artists")}>
        <span class="chevron" class:expanded={expandedSections.has("artists")}>&#9654;</span>
        <span class="section-title">Artists</span>
        <span class="section-count">{favoriteArtistNodes.length}</span>
      </button>
      {#if expandedSections.has("artists")}
        <div class="section-content">
          {#if favoriteArtistNodes.length === 0}
            <div class="empty-hint">No favorite artists yet</div>
          {:else}
            <TreeView
              artists={favoriteArtistNodes}
              onEditTrack={handleEditTrack}
              onEditAlbum={handleEditAlbum}
              onPlayTrack={handlePlayTrack}
              onPlayAlbum={handlePlayAlbum}
            />
          {/if}
        </div>
      {/if}
    </div>

    <div class="fav-section">
      <button class="section-header" onclick={() => toggleSection("albums")}>
        <span class="chevron" class:expanded={expandedSections.has("albums")}>&#9654;</span>
        <span class="section-title">Albums</span>
        <span class="section-count">{favoriteAlbumEntries.length}</span>
      </button>
      {#if expandedSections.has("albums")}
        <div class="section-content">
          {#if favoriteAlbumEntries.length === 0}
            <div class="empty-hint">No favorite albums yet</div>
          {:else}
            <AlbumListView
              albums={favoriteAlbumEntries}
              onEditTrack={handleEditTrack}
              onEditAlbum={handleEditAlbum}
              onPlayTrack={handlePlayTrack}
              onPlayAlbum={handlePlayAlbum}
            />
          {/if}
        </div>
      {/if}
    </div>

    <div class="fav-section">
      <button class="section-header" onclick={() => toggleSection("tracks")}>
        <span class="chevron" class:expanded={expandedSections.has("tracks")}>&#9654;</span>
        <span class="section-title">Tracks</span>
        <span class="section-count">{favoriteTracks.length}</span>
      </button>
      {#if expandedSections.has("tracks")}
        <div class="section-content">
          {#if favoriteTracks.length === 0}
            <div class="empty-hint">No favorite tracks yet</div>
          {:else}
            {#each favoriteTracks as track}
              <TrackRow
                {track}
                siblingTracks={favoriteTracks}
                onPlay={handlePlayTrack}
                onEdit={handleEditTrack}
              />
            {/each}
          {/if}
        </div>
      {/if}
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
  .favorites-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 8px;
  }

  .favorites-page h2 {
    font-size: 20px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .loading {
    color: var(--text-secondary);
    padding: 24px;
    text-align: center;
  }

  .fav-section {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    background: var(--bg-secondary);
    border: none;
    color: var(--text-primary);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
  }

  .section-header:hover {
    background: var(--bg-tertiary);
  }

  .section-title {
    font-weight: 600;
    flex: 1;
  }

  .section-count {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .section-content {
    padding: 4px 8px;
  }

  .empty-hint {
    padding: 12px 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .chevron {
    display: inline-block;
    font-size: 10px;
    transition: transform 0.15s;
    color: var(--text-secondary);
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }
</style>
