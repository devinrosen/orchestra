<script lang="ts">
  import { onMount } from "svelte";
  import type { Track, Favorite } from "../lib/api/types";
  import { listFavorites, getFavoriteTracks } from "../lib/api/commands";
  import { favoritesStore } from "../lib/stores/favorites.svelte";
  import { playerStore } from "../lib/stores/player.svelte";
  import TrackRow from "../lib/components/TrackRow.svelte";
  import MetadataEditor from "../lib/components/MetadataEditor.svelte";
  import { formatDuration } from "../lib/utils/format";

  let favoriteArtists = $state<Favorite[]>([]);
  let favoriteAlbums = $state<Favorite[]>([]);
  let favoriteTracks = $state<Track[]>([]);
  let loading = $state(true);

  let expandedSections = $state<Set<string>>(new Set(["artists", "albums", "tracks"]));
  let editingTrack = $state<Track | null>(null);

  function toggleSection(section: string) {
    const next = new Set(expandedSections);
    if (next.has(section)) next.delete(section);
    else next.add(section);
    expandedSections = next;
  }

  async function loadData() {
    loading = true;
    try {
      const [artists, albums, tracks] = await Promise.all([
        listFavorites("artist"),
        listFavorites("album"),
        getFavoriteTracks(),
      ]);
      favoriteArtists = artists;
      favoriteAlbums = albums;
      favoriteTracks = tracks;
    } catch (e) {
      console.error("Failed to load favorites:", e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  function parseAlbumKey(entityId: string): { artist: string; album: string } {
    const idx = entityId.indexOf("\0");
    if (idx === -1) return { artist: "", album: entityId };
    return { artist: entityId.substring(0, idx), album: entityId.substring(idx + 1) };
  }

  async function unfavoriteArtist(name: string) {
    await favoritesStore.toggle("artist", name);
    favoriteArtists = favoriteArtists.filter((f) => f.entity_id !== name);
  }

  async function unfavoriteAlbum(entityId: string) {
    await favoritesStore.toggle("album", entityId);
    favoriteAlbums = favoriteAlbums.filter((f) => f.entity_id !== entityId);
  }

  function handlePlayTrack(track: Track, siblingTracks: Track[]) {
    playerStore.playTrack(track, siblingTracks);
  }

  function handleEditTrack(track: Track) {
    editingTrack = track;
  }

  async function handleTrackSaved() {
    editingTrack = null;
    await loadData();
  }
</script>

<div class="favorites-page">
  <h2>Favorites</h2>

  {#if loading}
    <div class="loading">Loading favorites...</div>
  {:else}
    <div class="fav-section">
      <button class="section-header" onclick={() => toggleSection("artists")}>
        <span class="chevron" class:expanded={expandedSections.has("artists")}>&#9654;</span>
        <span class="section-title">Artists</span>
        <span class="section-count">{favoriteArtists.length}</span>
      </button>
      {#if expandedSections.has("artists")}
        <div class="section-content">
          {#if favoriteArtists.length === 0}
            <div class="empty-hint">No favorite artists yet</div>
          {:else}
            {#each favoriteArtists as fav}
              <div class="fav-item">
                <span class="fav-name">{fav.entity_id}</span>
                <button
                  class="unfav-btn"
                  onclick={() => unfavoriteArtist(fav.entity_id)}
                  title="Remove from favorites"
                >&#x2665;</button>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <div class="fav-section">
      <button class="section-header" onclick={() => toggleSection("albums")}>
        <span class="chevron" class:expanded={expandedSections.has("albums")}>&#9654;</span>
        <span class="section-title">Albums</span>
        <span class="section-count">{favoriteAlbums.length}</span>
      </button>
      {#if expandedSections.has("albums")}
        <div class="section-content">
          {#if favoriteAlbums.length === 0}
            <div class="empty-hint">No favorite albums yet</div>
          {:else}
            {#each favoriteAlbums as fav}
              {@const parsed = parseAlbumKey(fav.entity_id)}
              <div class="fav-item">
                <span class="fav-name">{parsed.album}</span>
                <span class="fav-artist">{parsed.artist}</span>
                <button
                  class="unfav-btn"
                  onclick={() => unfavoriteAlbum(fav.entity_id)}
                  title="Remove from favorites"
                >&#x2665;</button>
              </div>
            {/each}
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

  .fav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .fav-item:hover {
    background: var(--bg-secondary);
  }

  .fav-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fav-artist {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .unfav-btn {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 14px;
    padding: 2px 6px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .fav-item:hover .unfav-btn {
    opacity: 1;
  }

  .unfav-btn:hover {
    color: var(--danger);
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
