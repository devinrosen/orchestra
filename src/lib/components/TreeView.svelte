<script lang="ts">
  import type { ArtistNode, Track } from "../api/types";
  import AlbumHeader from "./AlbumHeader.svelte";
  import TrackRow from "./TrackRow.svelte";
  import PlaylistPicker from "./PlaylistPicker.svelte";
  import { favoritesStore } from "../stores/favorites.svelte";

  let {
    artists = [],
    onEditTrack,
    onEditAlbum,
    onPlayTrack,
    onPlayAlbum,
  }: {
    artists: ArtistNode[];
    onEditTrack?: (track: Track) => void;
    onEditAlbum?: (tracks: Track[], albumName: string, artistName: string) => void;
    onPlayTrack?: (track: Track, albumTracks: Track[]) => void;
    onPlayAlbum?: (tracks: Track[]) => void;
  } = $props();

  let expandedArtists = $state<Set<string>>(new Set());
  let expandedAlbums = $state<Set<string>>(new Set());

  let showPicker = $state(false);
  let pickerTrackIds = $state<number[]>([]);

  function handleAddToPlaylist(track: Track) {
    if (track.id != null) {
      pickerTrackIds = [track.id];
      showPicker = true;
    }
  }

  function toggleArtist(name: string) {
    const next = new Set(expandedArtists);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    expandedArtists = next;
  }

  function toggleAlbum(key: string) {
    const next = new Set(expandedAlbums);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expandedAlbums = next;
  }

</script>

<div class="tree-view">
  {#each artists as artist}
    <div class="artist-node">
      <button class="tree-toggle" onclick={() => toggleArtist(artist.name)}>
        <span class="chevron" class:expanded={expandedArtists.has(artist.name)}>&#9654;</span>
        <span class="artist-name">{artist.name}</span>
        <span class="count">{artist.albums.length} album{artist.albums.length !== 1 ? "s" : ""}</span>
      </button>
      <button
        class="artist-fav-btn"
        class:favorited={favoritesStore.isFavorite('artist', artist.name)}
        onclick={(e) => { e.stopPropagation(); favoritesStore.toggle('artist', artist.name); }}
        title={favoritesStore.isFavorite('artist', artist.name) ? "Remove from favorites" : "Add to favorites"}
      >{favoritesStore.isFavorite('artist', artist.name) ? "\u2665" : "\u2661"}</button>

      {#if expandedArtists.has(artist.name)}
        <div class="children">
          {#each artist.albums as album}
            {@const albumKey = `${artist.name}::${album.name}`}
            <div class="album-node">
              <AlbumHeader
                albumName={album.name}
                year={album.year}
                trackCount={album.tracks.length}
                expanded={expandedAlbums.has(albumKey)}
                onToggle={() => toggleAlbum(albumKey)}
                onPlay={onPlayAlbum ? () => onPlayAlbum(album.tracks) : undefined}
                onEdit={onEditAlbum ? () => onEditAlbum(album.tracks, album.name, artist.name) : undefined}
                isFavorited={favoritesStore.isFavorite('album', artist.name + "\0" + album.name)}
                onToggleFavorite={() => favoritesStore.toggle('album', artist.name + "\0" + album.name)}
              />

              {#if expandedAlbums.has(albumKey)}
                <div class="children">
                  {#each album.tracks as track}
                    <TrackRow
                      {track}
                      siblingTracks={album.tracks}
                      onPlay={onPlayTrack}
                      onEdit={onEditTrack}
                      onAddToPlaylist={handleAddToPlaylist}
                    />
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>

{#if showPicker}
  <PlaylistPicker trackIds={pickerTrackIds} onClose={() => showPicker = false} />
{/if}

<style>
  .tree-view {
    overflow-y: auto;
    padding: 8px;
  }

  .artist-name {
    font-weight: 600;
  }

  .artist-fav-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 4px 8px;
    border-radius: var(--radius);
    opacity: 0;
    transition: opacity 0.15s;
    cursor: pointer;
  }

  .artist-node:hover .artist-fav-btn {
    opacity: 1;
  }

  .artist-fav-btn.favorited {
    color: var(--accent);
    opacity: 1;
  }

  .artist-fav-btn:hover {
    color: var(--accent);
    background: var(--bg-tertiary);
  }
</style>
