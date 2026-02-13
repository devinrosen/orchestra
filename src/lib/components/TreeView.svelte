<script lang="ts">
  import type { ArtistNode, AlbumNode, Track } from "../api/types";
  import { playerStore } from "../stores/player.svelte";

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

  function formatDuration(secs: number | null): string {
    if (secs == null) return "--:--";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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

      {#if expandedArtists.has(artist.name)}
        <div class="children">
          {#each artist.albums as album}
            {@const albumKey = `${artist.name}::${album.name}`}
            <div class="album-node">
              <div class="album-header">
                <button class="tree-toggle" onclick={() => toggleAlbum(albumKey)}>
                  <span class="chevron" class:expanded={expandedAlbums.has(albumKey)}>&#9654;</span>
                  <span class="album-name">{album.name}</span>
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
                    onclick={(e) => { e.stopPropagation(); onEditAlbum(album.tracks, album.name, artist.name); }}
                    title="Edit album metadata"
                  >&#9998;</button>
                {/if}
              </div>

              {#if expandedAlbums.has(albumKey)}
                <div class="children">
                  {#each album.tracks as track}
                    {@const isPlaying = playerStore.currentTrack?.file_path === track.file_path}
                    <div class="track-row" class:now-playing={isPlaying}>
                      {#if onPlayTrack}
                        <button
                          class="track-play-btn"
                          onclick={(e) => { e.stopPropagation(); onPlayTrack(track, album.tracks); }}
                          title="Play track"
                        >&#9654;</button>
                      {/if}
                      <button
                        class="track-node"
                        onclick={() => onEditTrack?.(track)}
                        title="Edit track metadata"
                      >
                        <span class="track-num">{track.track_number ?? "-"}</span>
                        <span class="track-title">{track.title ?? track.relative_path}</span>
                        <span class="track-duration">{formatDuration(track.duration_secs)}</span>
                        <span class="track-format">{track.format.toUpperCase()}</span>
                        <span class="track-size">{formatSize(track.file_size)}</span>
                      </button>
                    </div>
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

  .artist-name {
    font-weight: 600;
  }

  .album-name {
    font-weight: 500;
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

  .track-row {
    display: flex;
    align-items: center;
    border-radius: var(--radius);
  }

  .track-row.now-playing {
    background-color: rgba(233, 69, 96, 0.1);
  }

  .track-row.now-playing .track-title {
    color: var(--accent);
    font-weight: 600;
  }

  .track-play-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    padding: 4px 4px 4px 8px;
    opacity: 0;
    transition: opacity 0.15s;
    flex-shrink: 0;
  }

  .track-row:hover .track-play-btn {
    opacity: 1;
  }

  .track-play-btn:hover {
    color: var(--accent);
  }

  .track-node {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 8px;
    border-radius: var(--radius);
    font-size: 13px;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
  }

  .track-node:hover {
    background-color: var(--bg-secondary);
  }

  .track-num {
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

  .track-duration, .track-format, .track-size {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .track-format {
    background: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
  }
</style>
