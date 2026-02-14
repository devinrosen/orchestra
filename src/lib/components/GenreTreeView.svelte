<script lang="ts">
  import type { GenreNode, Track } from "../api/types";
  import AlbumHeader from "./AlbumHeader.svelte";
  import TrackRow from "./TrackRow.svelte";
  import PlaylistPicker from "./PlaylistPicker.svelte";

  let {
    genres = [],
    onEditTrack,
    onEditAlbum,
    onPlayTrack,
    onPlayAlbum,
  }: {
    genres: GenreNode[];
    onEditTrack?: (track: Track) => void;
    onEditAlbum?: (tracks: Track[], albumName: string, artistName: string) => void;
    onPlayTrack?: (track: Track, albumTracks: Track[]) => void;
    onPlayAlbum?: (tracks: Track[]) => void;
  } = $props();

  let expandedGenres = $state<Set<string>>(new Set());
  let expandedAlbums = $state<Set<string>>(new Set());

  let showPicker = $state(false);
  let pickerTrackIds = $state<number[]>([]);

  function handleAddToPlaylist(track: Track) {
    if (track.id != null) {
      pickerTrackIds = [track.id];
      showPicker = true;
    }
  }

  function toggleGenre(name: string) {
    const next = new Set(expandedGenres);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    expandedGenres = next;
  }

  function toggleAlbum(key: string) {
    const next = new Set(expandedAlbums);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expandedAlbums = next;
  }

</script>

<div class="tree-view">
  {#each genres as genre}
    <div class="genre-node">
      <button class="tree-toggle" onclick={() => toggleGenre(genre.name)}>
        <span class="chevron" class:expanded={expandedGenres.has(genre.name)}>&#9654;</span>
        <span class="genre-name">{genre.name}</span>
        <span class="count">{genre.albums.length} album{genre.albums.length !== 1 ? "s" : ""}</span>
      </button>

      {#if expandedGenres.has(genre.name)}
        <div class="children">
          {#each genre.albums as album}
            {@const albumKey = `${genre.name}\0${album.artist}\0${album.name}`}
            <div class="album-node">
              <AlbumHeader
                albumName={album.name}
                artistName={album.artist}
                year={album.year}
                trackCount={album.tracks.length}
                expanded={expandedAlbums.has(albumKey)}
                onToggle={() => toggleAlbum(albumKey)}
                onPlay={onPlayAlbum ? () => onPlayAlbum(album.tracks) : undefined}
                onEdit={onEditAlbum ? () => onEditAlbum(album.tracks, album.name, album.artist) : undefined}
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

  .genre-name {
    font-weight: 600;
  }

  .count {
    color: var(--text-secondary);
    font-size: 12px;
    margin-left: auto;
  }

  .children {
    padding-left: 20px;
  }
</style>
