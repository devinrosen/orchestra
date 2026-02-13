<script lang="ts">
  import type { DiffResult, DiffEntry } from "../api/types";

  let { diff }: { diff: DiffResult } = $props();

  let filter = $state<"all" | "add" | "remove" | "update" | "conflict">("all");
  let filteredEntries = $derived(
    filter === "all" ? diff.entries.filter((e) => e.action !== "unchanged") : diff.entries.filter((e) => e.action === filter),
  );

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function actionIcon(action: string): string {
    switch (action) {
      case "add": return "+";
      case "remove": return "-";
      case "update": return "~";
      case "conflict": return "!";
      default: return "";
    }
  }

  function actionClass(action: string): string {
    switch (action) {
      case "add": return "action-add";
      case "remove": return "action-remove";
      case "update": return "action-update";
      case "conflict": return "action-conflict";
      default: return "";
    }
  }
</script>

<div class="diff-view">
  <div class="diff-summary">
    <span class="stat add">+{diff.total_add} add</span>
    <span class="stat remove">-{diff.total_remove} remove</span>
    <span class="stat update">~{diff.total_update} update</span>
    {#if diff.total_conflict > 0}
      <span class="stat conflict">!{diff.total_conflict} conflict</span>
    {/if}
    <span class="stat transfer">{formatSize(diff.bytes_to_transfer)} to transfer</span>
  </div>

  <div class="diff-filters">
    <button class:active={filter === "all"} onclick={() => (filter = "all")}>
      All ({diff.entries.length - diff.total_unchanged})
    </button>
    <button class:active={filter === "add"} onclick={() => (filter = "add")}>Add ({diff.total_add})</button>
    <button class:active={filter === "remove"} onclick={() => (filter = "remove")}>Remove ({diff.total_remove})</button>
    <button class:active={filter === "update"} onclick={() => (filter = "update")}>Update ({diff.total_update})</button>
    {#if diff.total_conflict > 0}
      <button class:active={filter === "conflict"} onclick={() => (filter = "conflict")}>
        Conflicts ({diff.total_conflict})
      </button>
    {/if}
  </div>

  <div class="diff-list">
    {#each filteredEntries as entry}
      <div class="diff-entry {actionClass(entry.action)}">
        <span class="diff-icon">{actionIcon(entry.action)}</span>
        <span class="diff-path">{entry.relative_path}</span>
        <span class="diff-direction">
          {entry.direction === "source_to_target" ? "→" : entry.direction === "target_to_source" ? "←" : "↔"}
        </span>
        <span class="diff-size">{formatSize(entry.source_size ?? entry.target_size ?? 0)}</span>
      </div>
    {/each}
    {#if filteredEntries.length === 0}
      <div class="empty">No changes in this category</div>
    {/if}
  </div>
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .diff-summary {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .stat {
    font-size: 13px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: var(--radius);
  }

  .stat.add { color: var(--success); background: rgba(78, 204, 163, 0.1); }
  .stat.remove { color: var(--danger); background: rgba(233, 69, 96, 0.1); }
  .stat.update { color: var(--warning); background: rgba(240, 165, 0, 0.1); }
  .stat.conflict { color: #ff6b6b; background: rgba(255, 107, 107, 0.1); }
  .stat.transfer { color: var(--text-secondary); background: var(--bg-secondary); }

  .diff-filters {
    display: flex;
    gap: 4px;
  }

  .diff-filters button {
    padding: 4px 12px;
    font-size: 12px;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .diff-filters button.active {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: var(--accent);
  }

  .diff-list {
    overflow-y: auto;
    max-height: 400px;
  }

  .diff-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: var(--radius);
    font-size: 13px;
    border-left: 3px solid transparent;
  }

  .diff-entry:hover {
    background: var(--bg-secondary);
  }

  .diff-entry.action-add { border-left-color: var(--success); }
  .diff-entry.action-remove { border-left-color: var(--danger); }
  .diff-entry.action-update { border-left-color: var(--warning); }
  .diff-entry.action-conflict { border-left-color: #ff6b6b; }

  .diff-icon {
    font-weight: 700;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
  }

  .action-add .diff-icon { color: var(--success); }
  .action-remove .diff-icon { color: var(--danger); }
  .action-update .diff-icon { color: var(--warning); }
  .action-conflict .diff-icon { color: #ff6b6b; }

  .diff-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .diff-direction {
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .diff-size {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .empty {
    text-align: center;
    color: var(--text-secondary);
    padding: 32px;
  }
</style>
