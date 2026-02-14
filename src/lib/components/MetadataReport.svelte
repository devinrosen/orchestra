<script lang="ts">
  import type { Track } from "../api/types";
  import { getIncompleteTracks } from "../api/commands";

  let {
    libraryRoot,
    onEditTrack,
    onClose,
  }: {
    libraryRoot: string;
    onEditTrack: (track: Track) => void;
    onClose: () => void;
  } = $props();

  let tracks = $state<Track[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let filter = $state<"all" | "title" | "artist" | "album" | "artwork">("all");

  const filteredTracks = $derived(
    filter === "all"
      ? tracks
      : tracks.filter((t) => {
          if (filter === "title") return t.title === null;
          if (filter === "artist") return t.artist === null;
          if (filter === "album") return t.album === null;
          if (filter === "artwork") return !t.has_album_art;
          return true;
        })
  );

  const missingCounts = $derived({
    title: tracks.filter((t) => t.title === null).length,
    artist: tracks.filter((t) => t.artist === null).length,
    album: tracks.filter((t) => t.album === null).length,
    artwork: tracks.filter((t) => !t.has_album_art).length,
  });

  async function loadTracks() {
    loading = true;
    error = null;
    try {
      tracks = await getIncompleteTracks(libraryRoot);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function getMissingFields(track: Track): string[] {
    const fields: string[] = [];
    if (track.title === null) fields.push("title");
    if (track.artist === null) fields.push("artist");
    if (track.album === null) fields.push("album");
    if (!track.has_album_art) fields.push("artwork");
    return fields;
  }

  $effect(() => {
    loadTracks();
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="report-overlay" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="report-dialog" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="report-header">
      <h3>Metadata Report</h3>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    {#if loading}
      <div class="loading">Loading...</div>
    {:else if error}
      <div class="error-banner">{error}</div>
    {:else}
      <div class="summary-banner">
        {tracks.length} track{tracks.length !== 1 ? "s" : ""} with missing metadata
      </div>

      <div class="filter-bar">
        <button
          class="filter-btn"
          class:active={filter === "all"}
          onclick={() => (filter = "all")}
        >
          All ({tracks.length})
        </button>
        <button
          class="filter-btn"
          class:active={filter === "title"}
          onclick={() => (filter = "title")}
        >
          Missing Title ({missingCounts.title})
        </button>
        <button
          class="filter-btn"
          class:active={filter === "artist"}
          onclick={() => (filter = "artist")}
        >
          Missing Artist ({missingCounts.artist})
        </button>
        <button
          class="filter-btn"
          class:active={filter === "album"}
          onclick={() => (filter = "album")}
        >
          Missing Album ({missingCounts.album})
        </button>
        <button
          class="filter-btn"
          class:active={filter === "artwork"}
          onclick={() => (filter = "artwork")}
        >
          Missing Artwork ({missingCounts.artwork})
        </button>
      </div>

      <div class="track-list">
        {#each filteredTracks as track}
          <div class="track-row">
            <div class="track-info">
              <span class="track-path">{track.relative_path}</span>
              <div class="badges">
                {#each getMissingFields(track) as field}
                  <span class="badge badge-{field}">{field}</span>
                {/each}
              </div>
            </div>
            <button class="edit-btn" onclick={() => onEditTrack(track)}>Edit</button>
          </div>
        {/each}
        {#if filteredTracks.length === 0}
          <div class="empty">No tracks match this filter.</div>
        {/if}
      </div>

      <div class="report-actions">
        <button class="secondary" onclick={loadTracks}>Refresh</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .report-overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .report-dialog {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    width: 700px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .report-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .report-header h3 {
    font-size: 18px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    font-size: 20px;
    color: var(--text-secondary);
    padding: 4px 8px;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .loading {
    color: var(--text-secondary);
    text-align: center;
    padding: 32px;
  }

  .error-banner {
    background: var(--accent-tint-strong);
    color: var(--danger);
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .summary-banner {
    background: var(--bg-secondary);
    padding: 10px 14px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
  }

  .filter-bar {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .filter-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 4px 10px;
    font-size: 12px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: all 0.15s;
  }

  .filter-btn:hover {
    color: var(--text-primary);
  }

  .filter-btn.active {
    background: var(--accent);
    color: var(--on-accent);
    border-color: var(--accent);
  }

  .track-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .track-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .track-row:hover {
    background: var(--bg-secondary);
  }

  .track-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .track-path {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badges {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 8px;
    font-weight: 500;
    text-transform: uppercase;
  }

  .badge-title {
    background: var(--accent-tint-strong);
    color: var(--danger);
  }

  .badge-artist {
    background: var(--orange-tint);
    color: var(--orange-color);
  }

  .badge-album {
    background: var(--info-tint);
    color: var(--info-color);
  }

  .badge-artwork {
    background: var(--purple-tint);
    color: var(--purple-color);
  }

  .edit-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 3px 10px;
    font-size: 12px;
    border-radius: var(--radius);
    cursor: pointer;
    flex-shrink: 0;
  }

  .edit-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-secondary);
  }

  .empty {
    text-align: center;
    color: var(--text-secondary);
    padding: 24px;
    font-size: 13px;
  }

  .report-actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
