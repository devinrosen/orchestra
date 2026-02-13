<script lang="ts">
  import type { SyncProfile } from "../api/types";

  let {
    profile,
    onSelect,
    onDelete,
    onSync,
  }: {
    profile: SyncProfile;
    onSelect: () => void;
    onDelete: () => void;
    onSync: () => void;
  } = $props();

  function formatDate(ts: number | null): string {
    if (ts == null) return "Never";
    return new Date(ts * 1000).toLocaleString();
  }
</script>

<div class="profile-card">
  <div class="profile-header">
    <h3>{profile.name}</h3>
    <span class="sync-mode">{profile.sync_mode === "one_way" ? "One-Way" : "Two-Way"}</span>
  </div>
  <div class="profile-paths">
    <div class="path-row">
      <span class="path-label">Source</span>
      <span class="path-value">{profile.source_path}</span>
    </div>
    <div class="path-row">
      <span class="path-label">Target</span>
      <span class="path-value">{profile.target_path}</span>
    </div>
  </div>
  <div class="profile-meta">
    <span>Last synced: {formatDate(profile.last_synced_at)}</span>
  </div>
  <div class="profile-actions">
    <button class="primary" onclick={onSync}>Sync</button>
    <button class="secondary" onclick={onSelect}>Edit</button>
    <button class="danger-btn" onclick={onDelete}>Delete</button>
  </div>
</div>

<style>
  .profile-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .profile-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .profile-header h3 {
    font-size: 16px;
    font-weight: 600;
  }

  .sync-mode {
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 12px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .profile-paths {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .path-row {
    display: flex;
    gap: 8px;
    font-size: 13px;
  }

  .path-label {
    color: var(--text-secondary);
    width: 50px;
    flex-shrink: 0;
  }

  .path-value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .profile-meta {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .profile-actions {
    display: flex;
    gap: 8px;
  }

  .danger-btn {
    background: transparent;
    color: var(--danger);
    border: 1px solid var(--danger);
    padding: 6px 12px;
    font-size: 13px;
  }

  .danger-btn:hover {
    background: rgba(233, 69, 96, 0.1);
  }
</style>
