<script lang="ts">
  import type { AlbumEntry, Track } from "../api/types";
  import AlbumHeader from "./AlbumHeader.svelte";
  import TrackRow from "./TrackRow.svelte";
  import PlaylistPicker from "./PlaylistPicker.svelte";

  let {
    albums = [],
    onEditTrack,
    onEditAlbum,
    onPlayTrack,
    onPlayAlbum,
  }: {
    albums: AlbumEntry[];
    onEditTrack?: (track: Track) => void;
    onEditAlbum?: (tracks: Track[], albumName: string, artistName: string) => void;
    onPlayTrack?: (track: Track, albumTracks: Track[]) => void;
    onPlayAlbum?: (tracks: Track[]) => void;
  } = $props();

  let expandedAlbums = $state<Set<string>>(new Set());

  let showPicker = $state(false);
  let pickerTrackIds = $state<number[]>([]);

  function handleAddToPlaylist(track: Track) {
    if (track.id != null) {
      pickerTrackIds = [track.id];
      showPicker = true;
    }
  }

  function toggleAlbum(key: string) {
    const next = new Set(expandedAlbums);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expandedAlbums = next;
  }

</script>

<div class="tree-view">
  {#each albums as album}
    {@const albumKey = `${album.artist}\0${album.name}`}
    <div class="album-node">
      <AlbumHeader
        albumName={album.name}
        artistName={album.artist}
        year={album.year}
        trackCount={album.tracks.length}
        expanded={expandedAlbums.has(albumKey)}
        boldName
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

{#if showPicker}
  <PlaylistPicker trackIds={pickerTrackIds} onClose={() => showPicker = false} />
{/if}

<style>
  .tree-view {
    overflow-y: auto;
    padding: 8px;
  }

  .children {
    padding-left: 20px;
  }
</style>
