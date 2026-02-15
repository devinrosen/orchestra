<script lang="ts">
  import type { LibraryStats } from "../lib/api/types";
  import * as commands from "../lib/api/commands";
  import { libraryStore } from "../lib/stores/library.svelte";

  let stats = $state<LibraryStats | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    const root = libraryStore.activeRoot;
    if (root) {
      loadStats(root);
    } else {
      stats = null;
    }
  });

  async function loadStats(root: string) {
    loading = true;
    error = null;
    try {
      stats = await commands.getLibraryStats(root);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1_073_741_824) {
      return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
    }
    return `${(bytes / 1_048_576).toFixed(1)} MB`;
  }

  function formatDuration(secs: number): string {
    const hours = Math.floor(secs / 3600);
    const minutes = Math.floor((secs % 3600) / 60);
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    const s = Math.floor(secs % 60);
    return `${minutes}m ${s}s`;
  }

  function formatBitrate(kbps: number): string {
    return `${Math.round(kbps)} kbps`;
  }
</script>

<div class="statistics-page">
  <h2>Library Statistics</h2>

  {#if !libraryStore.activeRoot}
    <p class="empty-state">No library loaded. Scan a directory first.</p>
  {:else if loading}
    <p>Loading statistics...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if stats}
    <div class="summary-cards">
      <div class="card">
        <div class="card-value">{stats.total_tracks.toLocaleString()}</div>
        <div class="card-label">Tracks</div>
      </div>
      <div class="card">
        <div class="card-value">{stats.total_artists.toLocaleString()}</div>
        <div class="card-label">Artists</div>
      </div>
      <div class="card">
        <div class="card-value">{stats.total_albums.toLocaleString()}</div>
        <div class="card-label">Albums</div>
      </div>
      <div class="card">
        <div class="card-value">{formatBytes(stats.total_size)}</div>
        <div class="card-label">Library Size</div>
      </div>
      <div class="card">
        <div class="card-value">{formatDuration(stats.total_duration_secs)}</div>
        <div class="card-label">Total Duration</div>
      </div>
      <div class="card">
        <div class="card-value">{stats.avg_bitrate != null ? formatBitrate(stats.avg_bitrate) : "N/A"}</div>
        <div class="card-label">Avg Bitrate</div>
      </div>
    </div>

    <div class="breakdowns">
      <div class="breakdown-section">
        <h3>Format Breakdown</h3>
        {#if stats.formats.length === 0}
          <p class="empty-state">No format data</p>
        {:else}
          {@const maxCount = stats.formats[0].count}
          <div class="breakdown-list">
            {#each stats.formats as fmt}
              <div class="breakdown-row">
                <span class="breakdown-name">{fmt.format.toUpperCase()}</span>
                <div class="breakdown-bar-container">
                  <div
                    class="breakdown-bar"
                    style="width: {(fmt.count / maxCount) * 100}%"
                  ></div>
                </div>
                <span class="breakdown-count">{fmt.count.toLocaleString()}</span>
                <span class="breakdown-pct">{((fmt.count / stats.total_tracks) * 100).toFixed(0)}%</span>
                <span class="breakdown-size">{formatBytes(fmt.total_size)}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="breakdown-section">
        <h3>Genre Distribution</h3>
        {#if stats.genres.length === 0}
          <p class="empty-state">No genre data</p>
        {:else}
          {@const maxGenreCount = stats.genres[0].count}
          <div class="breakdown-list">
            {#each stats.genres as genre}
              <div class="breakdown-row">
                <span class="breakdown-name">{genre.genre}</span>
                <div class="breakdown-bar-container">
                  <div
                    class="breakdown-bar genre-bar"
                    style="width: {(genre.count / maxGenreCount) * 100}%"
                  ></div>
                </div>
                <span class="breakdown-count">{genre.count.toLocaleString()}</span>
                <span class="breakdown-pct">{((genre.count / stats.total_tracks) * 100).toFixed(0)}%</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .statistics-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .statistics-page h2 {
    font-size: 20px;
    font-weight: 600;
  }

  .empty-state {
    color: var(--text-secondary);
  }

  .error {
    color: var(--danger);
  }

  .summary-cards {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .card {
    background: var(--bg-secondary);
    border-radius: var(--radius);
    padding: 16px;
    text-align: center;
  }

  .card-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .card-label {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 4px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .breakdowns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }

  .breakdown-section h3 {
    font-size: 14px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 12px;
  }

  .breakdown-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .breakdown-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
  }

  .breakdown-name {
    width: 60px;
    font-weight: 500;
    font-size: 13px;
    flex-shrink: 0;
  }

  .breakdown-bar-container {
    flex: 1;
    height: 8px;
    background: var(--bg-primary);
    border-radius: 4px;
    overflow: hidden;
  }

  .breakdown-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .breakdown-bar.genre-bar {
    background: var(--success);
  }

  .breakdown-count {
    width: 50px;
    text-align: right;
    font-size: 13px;
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .breakdown-pct {
    width: 36px;
    text-align: right;
    font-size: 12px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .breakdown-size {
    width: 70px;
    text-align: right;
    font-size: 12px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }
</style>
