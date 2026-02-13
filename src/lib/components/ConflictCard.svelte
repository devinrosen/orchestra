<script lang="ts">
  import type { Conflict, Resolution } from "../api/types";

  let {
    conflict,
    resolution,
    onResolve,
  }: {
    conflict: Conflict;
    resolution: Resolution;
    onResolve: (resolution: Resolution) => void;
  } = $props();

  function formatSize(bytes: number | null): string {
    if (bytes == null) return "deleted";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(ts: number | null): string {
    if (ts == null) return "N/A";
    return new Date(ts * 1000).toLocaleString();
  }

  const conflictLabel: Record<string, string> = {
    both_modified: "Both Modified",
    deleted_and_modified: "Deleted & Modified",
    first_sync_differs: "First Sync Conflict",
  };

  const resolutions: { value: Resolution; label: string }[] = [
    { value: "keep_source", label: "Keep Source" },
    { value: "keep_target", label: "Keep Target" },
    { value: "keep_both", label: "Keep Both" },
    { value: "skip", label: "Skip" },
  ];
</script>

<div class="conflict-card">
  <div class="conflict-header">
    <span class="conflict-path">{conflict.relative_path}</span>
    <span class="conflict-type">{conflictLabel[conflict.conflict_type] ?? conflict.conflict_type}</span>
  </div>

  <div class="conflict-details">
    <div class="side">
      <span class="side-label">Source</span>
      <span>{formatSize(conflict.source_size)}</span>
      <span class="date">{formatDate(conflict.source_modified)}</span>
    </div>
    <div class="side">
      <span class="side-label">Target</span>
      <span>{formatSize(conflict.target_size)}</span>
      <span class="date">{formatDate(conflict.target_modified)}</span>
    </div>
  </div>

  <div class="resolution-options">
    {#each resolutions as opt}
      <button
        class:active={resolution === opt.value}
        onclick={() => onResolve(opt.value)}
      >
        {opt.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .conflict-card {
    background: var(--bg-secondary);
    border: 1px solid #ff6b6b33;
    border-radius: var(--radius);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .conflict-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .conflict-path {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .conflict-type {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 12px;
    background: rgba(255, 107, 107, 0.15);
    color: #ff6b6b;
    flex-shrink: 0;
  }

  .conflict-details {
    display: flex;
    gap: 16px;
  }

  .side {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
  }

  .side-label {
    font-weight: 600;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .date {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .resolution-options {
    display: flex;
    gap: 4px;
  }

  .resolution-options button {
    flex: 1;
    padding: 6px 8px;
    font-size: 12px;
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .resolution-options button.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
</style>
