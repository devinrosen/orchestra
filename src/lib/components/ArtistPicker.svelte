<script lang="ts">
  import type { ArtistSummary, AlbumSummary, AlbumSelection } from "../api/types";

  let {
    artists,
    albums = [],
    selectedArtists,
    selectedAlbums = [],
    onSave,
    onCancel,
  }: {
    artists: ArtistSummary[];
    albums?: AlbumSummary[];
    selectedArtists: string[];
    selectedAlbums?: AlbumSelection[];
    onSave: (artists: string[], albums: AlbumSelection[]) => void;
    onCancel: () => void;
  } = $props();

  let searchQuery = $state("");
  let selectedArtistSet = $state<Set<string>>(new Set(selectedArtists));
  let selectedAlbumSet = $state<Set<string>>(
    new Set(selectedAlbums.map((a) => `${a.artist_name}|||${a.album_name}`)),
  );
  let expandedArtists = $state<Set<string>>(new Set());

  // Reset local editing copy when the prop changes
  $effect(() => {
    selectedArtistSet = new Set(selectedArtists);
  });
  $effect(() => {
    selectedAlbumSet = new Set(
      selectedAlbums.map((a) => `${a.artist_name}|||${a.album_name}`),
    );
  });

  let albumsByArtist = $derived.by(() => {
    const map = new Map<string, AlbumSummary[]>();
    for (const album of albums) {
      const existing = map.get(album.artist_name) ?? [];
      existing.push(album);
      map.set(album.artist_name, existing);
    }
    return map;
  });

  let filteredArtists = $derived(
    searchQuery.trim().length === 0
      ? artists
      : artists.filter(
          (a) =>
            a.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            (albumsByArtist.get(a.name) ?? []).some((alb) =>
              alb.album_name.toLowerCase().includes(searchQuery.toLowerCase()),
            ),
        ),
  );

  let totalSelectedArtists = $derived(selectedArtistSet.size);
  let totalSelectedAlbums = $derived(selectedAlbumSet.size);
  let totalSize = $derived.by(() => {
    let size = 0;
    for (const a of artists) {
      if (selectedArtistSet.has(a.name)) {
        size += a.total_size;
      }
    }
    for (const key of selectedAlbumSet) {
      const [artistName, albumName] = key.split("|||");
      // Skip if the whole artist is already selected
      if (selectedArtistSet.has(artistName)) continue;
      const album = albums.find(
        (a) => a.artist_name === artistName && a.album_name === albumName,
      );
      if (album) size += album.total_size;
    }
    return size;
  });
  let totalTracks = $derived.by(() => {
    let count = 0;
    for (const a of artists) {
      if (selectedArtistSet.has(a.name)) {
        count += a.track_count;
      }
    }
    for (const key of selectedAlbumSet) {
      const [artistName, albumName] = key.split("|||");
      if (selectedArtistSet.has(artistName)) continue;
      const album = albums.find(
        (a) => a.artist_name === artistName && a.album_name === albumName,
      );
      if (album) count += album.track_count;
    }
    return count;
  });

  function toggleArtist(name: string) {
    const next = new Set(selectedArtistSet);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
      // Remove individual album selections for this artist since whole artist is selected
      const nextAlbums = new Set(selectedAlbumSet);
      for (const key of nextAlbums) {
        if (key.startsWith(`${name}|||`)) {
          nextAlbums.delete(key);
        }
      }
      selectedAlbumSet = nextAlbums;
    }
    selectedArtistSet = next;
  }

  function toggleAlbum(artistName: string, albumName: string) {
    const key = `${artistName}|||${albumName}`;
    const next = new Set(selectedAlbumSet);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    selectedAlbumSet = next;
  }

  function toggleExpanded(name: string) {
    const next = new Set(expandedArtists);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    expandedArtists = next;
  }

  function isArtistIndeterminate(artistName: string): boolean {
    if (selectedArtistSet.has(artistName)) return false;
    const artistAlbums = albumsByArtist.get(artistName) ?? [];
    if (artistAlbums.length === 0) return false;
    return artistAlbums.some((a) =>
      selectedAlbumSet.has(`${artistName}|||${a.album_name}`),
    );
  }

  function selectAll() {
    selectedArtistSet = new Set(filteredArtists.map((a) => a.name));
    // Clear album selections since all artists are selected
    const nextAlbums = new Set(selectedAlbumSet);
    for (const key of nextAlbums) {
      const artistName = key.split("|||")[0];
      if (selectedArtistSet.has(artistName)) {
        nextAlbums.delete(key);
      }
    }
    selectedAlbumSet = nextAlbums;
  }

  function deselectAll() {
    selectedArtistSet = new Set();
    selectedAlbumSet = new Set();
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024)
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function handleSave() {
    const artistList = Array.from(selectedArtistSet);
    const albumList: AlbumSelection[] = [];
    for (const key of selectedAlbumSet) {
      const [artist_name, album_name] = key.split("|||");
      // Don't include albums for fully selected artists
      if (!selectedArtistSet.has(artist_name)) {
        albumList.push({ artist_name, album_name });
      }
    }
    onSave(artistList, albumList);
  }
</script>

<div class="artist-picker">
  <div class="picker-header">
    <h3>Select Artists & Albums</h3>
    <div class="picker-actions">
      <button class="secondary" onclick={selectAll}>Select All</button>
      <button class="secondary" onclick={deselectAll}>Deselect All</button>
    </div>
  </div>

  <input
    type="text"
    placeholder="Search artists or albums..."
    bind:value={searchQuery}
    class="search-input"
  />

  <div class="artist-list">
    {#each filteredArtists as artist}
      <div class="artist-group" class:has-selection={selectedArtistSet.has(artist.name) || isArtistIndeterminate(artist.name)}>
        <div class="artist-row">
          {#if (albumsByArtist.get(artist.name) ?? []).length > 0}
            <button
              class="expand-btn"
              onclick={() => toggleExpanded(artist.name)}
              aria-label={expandedArtists.has(artist.name) ? "Collapse" : "Expand"}
            >
              <span class="chevron" class:expanded={expandedArtists.has(artist.name)}>&#9654;</span>
            </button>
          {:else}
            <span class="expand-placeholder"></span>
          {/if}
          <label class="artist-label" class:selected={selectedArtistSet.has(artist.name)}>
            <input
              type="checkbox"
              checked={selectedArtistSet.has(artist.name)}
              indeterminate={isArtistIndeterminate(artist.name)}
              onchange={() => toggleArtist(artist.name)}
            />
            <span class="artist-name">{artist.name}</span>
            <span class="artist-meta">
              {artist.album_count} album{artist.album_count !== 1 ? "s" : ""}
              &middot; {artist.track_count} track{artist.track_count !== 1 ? "s" : ""}
              &middot; {formatSize(artist.total_size)}
            </span>
          </label>
        </div>
        {#if expandedArtists.has(artist.name)}
          <div class="album-list">
            {#each albumsByArtist.get(artist.name) ?? [] as album}
              <label
                class="album-row"
                class:selected={selectedArtistSet.has(artist.name) || selectedAlbumSet.has(`${artist.name}|||${album.album_name}`)}
              >
                <input
                  type="checkbox"
                  checked={selectedArtistSet.has(artist.name) || selectedAlbumSet.has(`${artist.name}|||${album.album_name}`)}
                  disabled={selectedArtistSet.has(artist.name)}
                  onchange={() => toggleAlbum(artist.name, album.album_name)}
                />
                <span class="album-name">{album.album_name}</span>
                <span class="album-meta">
                  {#if album.year}{album.year} &middot; {/if}
                  {album.track_count} track{album.track_count !== 1 ? "s" : ""}
                  &middot; {formatSize(album.total_size)}
                </span>
              </label>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
    {#if filteredArtists.length === 0}
      <div class="empty">No artists match your search</div>
    {/if}
  </div>

  <div class="picker-footer">
    <div class="selection-summary">
      {totalSelectedArtists} artist{totalSelectedArtists !== 1 ? "s" : ""}
      {#if totalSelectedAlbums > 0}
        , {totalSelectedAlbums} album{totalSelectedAlbums !== 1 ? "s" : ""}
      {/if}
      selected
      &middot; {totalTracks} tracks
      &middot; {formatSize(totalSize)}
    </div>
    <div class="footer-actions">
      <button class="secondary" onclick={onCancel}>Cancel</button>
      <button class="primary" onclick={handleSave}>Save Selection</button>
    </div>
  </div>
</div>

<style>
  .artist-picker {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    min-height: 0;
  }

  .picker-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .picker-header h3 {
    font-size: 18px;
    font-weight: 600;
  }

  .picker-actions {
    display: flex;
    gap: 8px;
  }

  .search-input {
    width: 100%;
  }

  .artist-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-height: 0;
  }

  .artist-group.has-selection {
    background: var(--success-tint-subtle);
    border-radius: var(--radius);
  }

  .artist-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 4px 4px 2px;
  }

  .expand-btn {
    background: none;
    border: none;
    padding: 4px 6px;
    cursor: pointer;
    color: var(--text-secondary);
    font-size: 10px;
    line-height: 1;
    flex-shrink: 0;
  }

  .expand-btn:hover {
    color: var(--text-primary);
  }

  .expand-placeholder {
    width: 22px;
    flex-shrink: 0;
  }

  .chevron {
    display: inline-block;
    transition: transform 0.15s;
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .artist-label {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 8px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background 0.1s;
    flex: 1;
    min-width: 0;
  }

  .artist-label:hover {
    background: var(--bg-secondary);
  }

  .artist-label.selected {
    background: var(--success-tint-strong);
  }

  .artist-label input[type="checkbox"],
  .album-row input[type="checkbox"] {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .artist-name {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .artist-meta {
    font-size: 12px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .album-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding-left: 32px;
    padding-bottom: 4px;
  }

  .album-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 10px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background 0.1s;
    font-size: 13px;
  }

  .album-row:hover {
    background: var(--bg-secondary);
  }

  .album-row.selected {
    background: var(--success-tint-medium);
  }

  .album-row input[type="checkbox"]:disabled {
    opacity: 0.5;
  }

  .album-name {
    flex: 1;
    font-weight: 400;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .album-meta {
    font-size: 11px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .empty {
    text-align: center;
    padding: 32px;
    color: var(--text-secondary);
  }

  .picker-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .selection-summary {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .footer-actions {
    display: flex;
    gap: 8px;
  }
</style>
