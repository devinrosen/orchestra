<script lang="ts">
  import type { GenreNode, Track } from "../api/types";
  import TrackRow from "./TrackRow.svelte";

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
              <div class="album-header">
                <button class="tree-toggle" onclick={() => toggleAlbum(albumKey)}>
                  <span class="chevron" class:expanded={expandedAlbums.has(albumKey)}>&#9654;</span>
                  <span class="album-name">{album.name}</span>
                  <span class="album-artist">{album.artist}</span>
                  {#if album.year}<span class="year">({album.year})</span>{/if}
                  <span class="count">{album.tracks.length} track{album.tracks.length !== 1 ? "s" : ""}</span>
                </button>
                {#if onPlayAlbum}
                  <button
                    class="action-btn play-album-btn"
                    onclick={(e) => { e.stopPropagation(); onPlayAlbum(album.tracks); }}
                    title="Play album"
                  >&#9654;</button>
                {/if}
                {#if onEditAlbum}
                  <button
                    class="action-btn edit-btn"
                    onclick={(e) => { e.stopPropagation(); onEditAlbum(album.tracks, album.name, album.artist); }}
                    title="Edit album metadata"
                  >&#9998;</button>
                {/if}
              </div>

              {#if expandedAlbums.has(albumKey)}
                <div class="children">
                  {#each album.tracks as track}
                    <TrackRow
                      {track}
                      siblingTracks={album.tracks}
                      onPlay={onPlayTrack}
                      onEdit={onEditTrack}
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

  .album-name {
    font-weight: 500;
  }

  .album-artist {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .year {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .count {
    color: var(--text-secondary);
    font-size: 12px;
    margin-left: auto;
  }

  .children {
    padding-left: 20px;
  }

  .album-header {
    display: flex;
    align-items: center;
  }

  .album-header .tree-toggle {
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

  .album-header:hover .action-btn {
    opacity: 1;
  }

  .action-btn:hover {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .play-album-btn {
    font-size: 12px;
  }

</style>
