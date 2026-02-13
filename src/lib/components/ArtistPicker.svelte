<script lang="ts">
  import type { ArtistSummary } from "../api/types";

  let {
    artists,
    selectedArtists,
    onSave,
    onCancel,
  }: {
    artists: ArtistSummary[];
    selectedArtists: string[];
    onSave: (selected: string[]) => void;
    onCancel: () => void;
  } = $props();

  let searchQuery = $state("");
  // We intentionally capture just the initial value to create a local editing copy
  let initialSelected = selectedArtists;
  let selected = $state<Set<string>>(new Set(initialSelected));

  let filteredArtists = $derived(
    searchQuery.trim().length === 0
      ? artists
      : artists.filter((a) =>
          a.name.toLowerCase().includes(searchQuery.toLowerCase()),
        ),
  );

  let totalSelected = $derived(selected.size);
  let totalSize = $derived(
    artists
      .filter((a) => selected.has(a.name))
      .reduce((sum, a) => sum + a.total_size, 0),
  );
  let totalTracks = $derived(
    artists
      .filter((a) => selected.has(a.name))
      .reduce((sum, a) => sum + a.track_count, 0),
  );

  function toggle(name: string) {
    const next = new Set(selected);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    selected = next;
  }

  function selectAll() {
    selected = new Set(filteredArtists.map((a) => a.name));
  }

  function deselectAll() {
    selected = new Set();
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function handleSave() {
    onSave(Array.from(selected));
  }
</script>

<div class="artist-picker">
  <div class="picker-header">
    <h3>Select Artists</h3>
    <div class="picker-actions">
      <button class="secondary" onclick={selectAll}>Select All</button>
      <button class="secondary" onclick={deselectAll}>Deselect All</button>
    </div>
  </div>

  <input
    type="text"
    placeholder="Search artists..."
    bind:value={searchQuery}
    class="search-input"
  />

  <div class="artist-list">
    {#each filteredArtists as artist}
      <label class="artist-row" class:selected={selected.has(artist.name)}>
        <input
          type="checkbox"
          checked={selected.has(artist.name)}
          onchange={() => toggle(artist.name)}
        />
        <span class="artist-name">{artist.name}</span>
        <span class="artist-meta">
          {artist.album_count} album{artist.album_count !== 1 ? "s" : ""}
          &middot; {artist.track_count} track{artist.track_count !== 1 ? "s" : ""}
          &middot; {formatSize(artist.total_size)}
        </span>
      </label>
    {/each}
    {#if filteredArtists.length === 0}
      <div class="empty">No artists match your search</div>
    {/if}
  </div>

  <div class="picker-footer">
    <div class="selection-summary">
      {totalSelected} artist{totalSelected !== 1 ? "s" : ""} selected
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
    height: 100%;
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

  .artist-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background 0.1s;
  }

  .artist-row:hover {
    background: var(--bg-secondary);
  }

  .artist-row.selected {
    background: rgba(78, 204, 163, 0.08);
  }

  .artist-row input[type="checkbox"] {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .artist-name {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
  }

  .artist-meta {
    font-size: 12px;
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
