<script lang="ts">
  import type { DuplicateResult, DuplicateGroup, DuplicateMatchType } from "../api/types";
  import { findDuplicates, deleteDuplicateTracks } from "../api/commands";

  let {
    libraryRoot,
    onClose,
  }: {
    libraryRoot: string;
    onClose: () => void;
  } = $props();

  let result = $state<DuplicateResult | null>(null);
  let loading = $state(true);
  let hashing = $state(false);
  let hashProgress = $state({ current: 0, total: 0 });
  let error = $state<string | null>(null);
  let filter = $state<"all" | "content_hash" | "metadata_similarity">("all");
  let selectedForDeletion = $state(new Map<number, string>());
  let deleting = $state(false);

  const filteredGroups = $derived(
    result
      ? filter === "all"
        ? result.groups
        : result.groups.filter((g) => g.match_type === filter)
      : [],
  );

  const contentCount = $derived(
    result ? result.groups.filter((g) => g.match_type === "content_hash").length : 0,
  );

  const metadataCount = $derived(
    result ? result.groups.filter((g) => g.match_type === "metadata_similarity").length : 0,
  );

  const selectedCount = $derived(selectedForDeletion.size);

  const selectedBytes = $derived(
    result
      ? result.groups
          .flatMap((g) => g.tracks)
          .filter((t) => t.id !== null && selectedForDeletion.has(t.id!))
          .reduce((sum, t) => sum + t.file_size, 0)
      : 0,
  );

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function matchLabel(type: DuplicateMatchType): string {
    return type === "content_hash" ? "HASH" : "METADATA";
  }

  async function loadDuplicates() {
    loading = true;
    hashing = false;
    error = null;
    selectedForDeletion = new Map();
    try {
      result = await findDuplicates(libraryRoot, (event) => {
        if (event.type === "scan_progress") {
          hashing = true;
          hashProgress = { current: event.files_processed, total: event.files_found };
        } else if (event.type === "scan_complete") {
          hashing = false;
        }
      });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
      hashing = false;
    }
  }

  function toggleSelection(trackId: number, filePath: string) {
    const next = new Map(selectedForDeletion);
    if (next.has(trackId)) {
      next.delete(trackId);
    } else {
      next.set(trackId, filePath);
    }
    selectedForDeletion = next;
  }

  function selectAllButBest(group: DuplicateGroup) {
    const next = new Map(selectedForDeletion);
    // Find the "best" track: highest bitrate, then largest file size
    const sorted = [...group.tracks].sort((a, b) => {
      const bitrateA = a.bitrate ?? 0;
      const bitrateB = b.bitrate ?? 0;
      if (bitrateB !== bitrateA) return bitrateB - bitrateA;
      return b.file_size - a.file_size;
    });
    const bestId = sorted[0]?.id;
    for (const track of group.tracks) {
      if (track.id !== null && track.id !== bestId) {
        next.set(track.id!, track.file_path);
      } else if (track.id !== null) {
        next.delete(track.id!);
      }
    }
    selectedForDeletion = next;
  }

  async function deleteSelected() {
    if (selectedForDeletion.size === 0) return;
    deleting = true;
    try {
      const trackIds = Array.from(selectedForDeletion.keys());
      const filePaths = Array.from(selectedForDeletion.values());
      await deleteDuplicateTracks(trackIds, filePaths);
      await loadDuplicates();
    } catch (e) {
      error = String(e);
    } finally {
      deleting = false;
    }
  }

  $effect(() => {
    loadDuplicates();
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="report-overlay" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="report-dialog" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="report-header">
      <h3>Duplicate Detection</h3>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    {#if loading}
      <div class="loading">
        {#if hashing}
          Hashing tracks... {hashProgress.current}/{hashProgress.total}
        {:else}
          Scanning for duplicates...
        {/if}
      </div>
    {:else if error}
      <div class="error-banner">{error}</div>
    {:else if result}
      <div class="summary-banner">
        {result.groups.length} duplicate group{result.groups.length !== 1 ? "s" : ""} found
        &mdash; {result.total_duplicate_tracks} extra track{result.total_duplicate_tracks !== 1 ? "s" : ""}
        ({formatBytes(result.total_wasted_bytes)} wasted)
      </div>

      <div class="filter-bar">
        <button
          class="filter-btn"
          class:active={filter === "all"}
          onclick={() => (filter = "all")}
        >
          All ({result.groups.length})
        </button>
        <button
          class="filter-btn"
          class:active={filter === "content_hash"}
          onclick={() => (filter = "content_hash")}
        >
          Content Match ({contentCount})
        </button>
        <button
          class="filter-btn"
          class:active={filter === "metadata_similarity"}
          onclick={() => (filter = "metadata_similarity")}
        >
          Metadata Match ({metadataCount})
        </button>
      </div>

      <div class="group-list">
        {#each filteredGroups as group}
          <div class="dup-group">
            <div class="group-header">
              <span class="match-badge badge-{group.match_type}">{matchLabel(group.match_type)}</span>
              <span class="group-count">{group.tracks.length} copies</span>
              <button class="select-best-btn" onclick={() => selectAllButBest(group)}>
                Select All But Best
              </button>
            </div>
            <div class="group-tracks">
              {#each group.tracks as track}
                <label class="track-row">
                  <input
                    type="checkbox"
                    checked={track.id !== null && selectedForDeletion.has(track.id)}
                    onchange={() => {
                      if (track.id !== null) toggleSelection(track.id, track.file_path);
                    }}
                  />
                  <div class="track-info">
                    <span class="track-path">{track.relative_path}</span>
                    <div class="track-meta">
                      <span class="format-badge">{track.format}</span>
                      {#if track.bitrate}
                        <span class="meta-detail">{track.bitrate} kbps</span>
                      {/if}
                      <span class="meta-detail">{formatBytes(track.file_size)}</span>
                    </div>
                  </div>
                </label>
              {/each}
            </div>
          </div>
        {/each}
        {#if filteredGroups.length === 0}
          <div class="empty">No duplicates match this filter.</div>
        {/if}
      </div>

      <div class="report-actions">
        <button class="secondary" onclick={loadDuplicates}>Refresh</button>
        <button
          class="primary danger-btn"
          disabled={selectedCount === 0 || deleting}
          onclick={deleteSelected}
        >
          {deleting
            ? "Deleting..."
            : `Delete ${selectedCount} selected (${formatBytes(selectedBytes)})`}
        </button>
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
    width: 750px;
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

  .group-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .dup-group {
    background: var(--bg-secondary);
    border-radius: var(--radius);
    padding: 12px;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .match-badge {
    font-size: 10px;
    padding: 2px 8px;
    border-radius: 8px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge-content_hash {
    background: var(--success-tint);
    color: var(--success);
  }

  .badge-metadata_similarity {
    background: var(--info-tint);
    color: var(--info-color);
  }

  .group-count {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .select-best-btn {
    margin-left: auto;
    background: var(--bg-tertiary, var(--bg-primary));
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 2px 8px;
    font-size: 11px;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .select-best-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-secondary);
  }

  .group-tracks {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .track-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    border-radius: var(--radius);
    font-size: 13px;
    cursor: pointer;
  }

  .track-row:hover {
    background: var(--bg-tertiary);
  }

  .track-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
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

  .track-meta {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .format-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 8px;
    font-weight: 500;
    text-transform: uppercase;
    background: var(--orange-tint);
    color: var(--orange-color);
  }

  .meta-detail {
    font-size: 11px;
    color: var(--text-secondary);
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
    gap: 8px;
  }

  .danger-btn {
    background: var(--danger);
    border-color: var(--danger);
  }

  .danger-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
