<script lang="ts">
  import DiffView from "../lib/components/DiffView.svelte";
  import ConflictCard from "../lib/components/ConflictCard.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";
  import { syncStore } from "../lib/stores/sync.svelte";
  import type { Resolution } from "../lib/api/types";

  let {
    profileId,
    onNavigate,
  }: {
    profileId: string;
    onNavigate: (page: string, data?: Record<string, unknown>) => void;
  } = $props();

  $effect(() => {
    syncStore.computeDiff(profileId);
  });

  function handleResolve(relativePath: string, resolution: Resolution) {
    syncStore.setResolution(relativePath, resolution);
  }

  async function startSync() {
    await syncStore.executeSync(profileId);
  }

  function handleCancel() {
    syncStore.cancel();
  }

  function handleDone() {
    syncStore.reset();
    onNavigate("profiles");
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }
</script>

<div class="sync-preview-page">
  {#if syncStore.phase === "computing_diff"}
    <div class="loading">
      <h2>Computing Diff...</h2>
      <p>Comparing source and target directories</p>
    </div>
  {:else if syncStore.phase === "previewing" && syncStore.diffResult}
    <div class="preview-header">
      <h2>Sync Preview</h2>
      <div class="preview-actions">
        <button class="secondary" onclick={() => onNavigate("profiles")}>Back</button>
        <button
          class="primary"
          onclick={startSync}
          disabled={syncStore.diffResult.entries.every((e) => e.action === "unchanged")}
        >
          Execute Sync
        </button>
      </div>
    </div>

    <DiffView diff={syncStore.diffResult} />

    {#if syncStore.conflicts.length > 0}
      <div class="conflicts-section">
        <h3>Conflicts ({syncStore.conflicts.length})</h3>
        <p class="conflicts-hint">Choose how to resolve each conflict before syncing.</p>
        {#each syncStore.conflicts as conflict, i}
          <ConflictCard
            {conflict}
            resolution={syncStore.resolutions[i]?.resolution ?? "skip"}
            onResolve={(r) => handleResolve(conflict.relative_path, r)}
          />
        {/each}
      </div>
    {/if}

  {:else if syncStore.phase === "syncing"}
    <div class="sync-progress">
      <h2>Syncing...</h2>
      <ProgressBar
        value={syncStore.progress.filesCompleted}
        max={syncStore.progress.totalFiles}
        label="{syncStore.progress.currentFile}"
      />
      <div class="progress-details">
        <span>{syncStore.progress.filesCompleted} / {syncStore.progress.totalFiles} files</span>
        <span>{formatSize(syncStore.progress.bytesCompleted)} / {formatSize(syncStore.progress.totalBytes)}</span>
      </div>
      <button class="secondary" onclick={handleCancel}>Cancel</button>
    </div>

  {:else if syncStore.phase === "complete"}
    <div class="complete">
      <h2>Sync Complete</h2>
      {#if syncStore.syncErrors.length > 0}
        <div class="sync-errors">
          <h3>Errors ({syncStore.syncErrors.length})</h3>
          {#each syncStore.syncErrors as err}
            <div class="error-item">
              <span class="error-file">{err.file}</span>
              <span class="error-msg">{err.error}</span>
            </div>
          {/each}
        </div>
      {/if}
      <button class="primary" onclick={handleDone}>Done</button>
    </div>

  {:else if syncStore.phase === "error"}
    <div class="error-state">
      <h2>Error</h2>
      <p>{syncStore.error}</p>
      <button class="primary" onclick={handleDone}>Back</button>
    </div>
  {/if}
</div>

<style>
  .sync-preview-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 16px;
    overflow-y: auto;
  }

  .loading, .complete, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 12px;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .preview-header h2 {
    font-size: 20px;
    font-weight: 600;
  }

  .preview-actions {
    display: flex;
    gap: 8px;
  }

  .conflicts-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .conflicts-section h3 {
    font-size: 16px;
  }

  .conflicts-hint {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .sync-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 16px;
    max-width: 500px;
    margin: 0 auto;
    width: 100%;
  }

  .progress-details {
    display: flex;
    justify-content: space-between;
    width: 100%;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .sync-errors {
    width: 100%;
    max-width: 600px;
  }

  .error-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    font-size: 13px;
    border-left: 3px solid var(--danger);
    margin-bottom: 4px;
  }

  .error-file { font-weight: 500; }
  .error-msg { color: var(--text-secondary); }

  .error-state p {
    color: var(--danger);
  }
</style>
