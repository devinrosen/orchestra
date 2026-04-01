<script lang="ts">
  import type { OrganizePreviewResult, OrganizeApplyResult } from "../api/types";

  interface Props {
    preview: OrganizePreviewResult;
    applying: boolean;
    progress: { completed: number; total: number; currentFile: string };
    result: OrganizeApplyResult | null;
    onApply: (excludedIds: Set<number>) => void;
  }

  let { preview, applying, progress, result, onApply }: Props = $props();

  let excludedIds = $state<Set<number>>(new Set());

  function toggleExclude(trackId: number) {
    const next = new Set(excludedIds);
    if (next.has(trackId)) {
      next.delete(trackId);
    } else {
      next.add(trackId);
    }
    excludedIds = next;
  }

  const okItems = $derived(
    preview.items.filter((item) => item.status.type === "Ok"),
  );
  const applyableCount = $derived(
    okItems.filter((item) => !excludedIds.has(item.track_id)).length,
  );
  const progressPct = $derived(
    progress.total > 0 ? Math.round((progress.completed / progress.total) * 100) : 0,
  );

  function statusLabel(type: string): string {
    switch (type) {
      case "Ok": return "Will move";
      case "AlreadyCorrect": return "Already correct";
      case "Collision": return "Collision";
      case "Error": return "Error";
      default: return type;
    }
  }

  function statusClass(type: string): string {
    switch (type) {
      case "Ok": return "status-ok";
      case "AlreadyCorrect": return "status-correct";
      case "Collision": return "status-collision";
      case "Error": return "status-error";
      default: return "";
    }
  }
</script>

<div class="organize-preview">
  <div class="summary-bar">
    <span class="summary-item">Total: <strong>{preview.total}</strong></span>
    <span class="summary-item status-ok-text">Will move: <strong>{okItems.length}</strong></span>
    <span class="summary-item status-correct-text">Already correct: <strong>{preview.already_correct}</strong></span>
    {#if preview.collisions > 0}
      <span class="summary-item status-collision-text">Collisions: <strong>{preview.collisions}</strong></span>
    {/if}
    {#if preview.errors > 0}
      <span class="summary-item status-error-text">Errors: <strong>{preview.errors}</strong></span>
    {/if}
  </div>

  <div class="table-wrap">
    <table class="preview-table">
      <thead>
        <tr>
          <th class="col-check"></th>
          <th class="col-status">Status</th>
          <th class="col-path">Current Path</th>
          <th class="col-arrow"></th>
          <th class="col-path">Proposed Path</th>
        </tr>
      </thead>
      <tbody>
        {#each preview.items as item (item.track_id)}
          {@const stype = item.status.type}
          {@const isOk = stype === "Ok"}
          <tr class="row {statusClass(stype)}" class:excluded={excludedIds.has(item.track_id)}>
            <td class="col-check">
              {#if isOk}
                <input
                  type="checkbox"
                  checked={!excludedIds.has(item.track_id)}
                  onchange={() => toggleExclude(item.track_id)}
                  disabled={applying}
                  aria-label="Include in apply"
                />
              {/if}
            </td>
            <td class="col-status">
              <span class="status-badge {statusClass(stype)}">{statusLabel(stype)}</span>
              {#if stype === "Collision" && "conflicting_track_id" in item.status}
                <span class="collision-hint"> (id: {item.status.conflicting_track_id ?? "?"})</span>
              {/if}
              {#if stype === "Error" && "reason" in item.status}
                <span class="error-hint" title={item.status.reason}> — {item.status.reason}</span>
              {/if}
            </td>
            <td class="col-path path-cell">{item.current_relative_path}</td>
            <td class="col-arrow">→</td>
            <td class="col-path path-cell">{item.proposed_relative_path}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if applying}
    <div class="progress-section">
      <div class="progress-bar-wrap">
        <div class="progress-bar" style="width: {progressPct}%"></div>
      </div>
      <p class="progress-text">
        {progress.completed} / {progress.total} — {progress.currentFile}
      </p>
    </div>
  {/if}

  {#if result}
    <div class="result-banner">
      Done — moved {result.moved}, skipped {result.skipped}{result.errors.length > 0 ? `, ${result.errors.length} error(s)` : ""}.
    </div>
  {/if}

  <div class="action-row">
    <button
      class="apply-btn"
      disabled={applying || applyableCount === 0 || !!result}
      onclick={() => onApply(excludedIds)}
    >
      {applying ? "Applying…" : `Apply (${applyableCount} file${applyableCount === 1 ? "" : "s"})`}
    </button>
  </div>
</div>

<style>
  .organize-preview {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .summary-bar {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    font-size: 13px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
    border: 1px solid var(--border);
  }

  .summary-item {
    color: var(--text-secondary);
  }

  .status-ok-text { color: var(--accent); }
  .status-correct-text { color: var(--text-secondary); }
  .status-collision-text { color: #d97706; }
  .status-error-text { color: #ef4444; }

  .table-wrap {
    overflow-x: auto;
    max-height: 420px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .preview-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  .preview-table thead th {
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-weight: 500;
    padding: 6px 8px;
    text-align: left;
    position: sticky;
    top: 0;
    z-index: 1;
    border-bottom: 1px solid var(--border);
  }

  .preview-table tbody tr {
    border-bottom: 1px solid var(--border);
  }

  .preview-table tbody tr:last-child {
    border-bottom: none;
  }

  .preview-table td {
    padding: 5px 8px;
    vertical-align: middle;
  }

  .row.excluded {
    opacity: 0.45;
  }

  .col-check { width: 28px; }
  .col-status { width: 130px; white-space: nowrap; }
  .col-arrow { width: 24px; color: var(--text-secondary); text-align: center; }

  .path-cell {
    font-family: monospace;
    word-break: break-all;
    color: var(--text-primary);
  }

  .status-badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
  }

  .status-ok .status-badge { background: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent); }
  .status-correct .status-badge { background: var(--bg-tertiary); color: var(--text-secondary); }
  .status-collision .status-badge { background: #d9770620; color: #d97706; }
  .status-error .status-badge { background: #ef444420; color: #ef4444; }

  .collision-hint,
  .error-hint {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .progress-bar-wrap {
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s;
  }

  .progress-text {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0;
    font-family: monospace;
    word-break: break-all;
  }

  .result-banner {
    padding: 8px 12px;
    background: color-mix(in srgb, var(--accent) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
    border-radius: var(--radius);
    font-size: 13px;
    color: var(--text-primary);
  }

  .action-row {
    display: flex;
    justify-content: flex-end;
  }

  .apply-btn {
    padding: 8px 20px;
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: var(--radius);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .apply-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .apply-btn:not(:disabled):hover {
    opacity: 0.85;
  }
</style>
