<script lang="ts">
  import type { DeviceWithStatus } from "../api/types";

  let {
    device,
    busy = false,
    onConfigure,
    onSync,
    onDelete,
  }: {
    device: DeviceWithStatus;
    busy?: boolean;
    onConfigure: () => void;
    onSync: () => void;
    onDelete: () => void;
  } = $props();

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  let capacityPercent = $derived(
    device.device.capacity_bytes
      ? 0 // We don't have used bytes info, just show capacity
      : 0,
  );
</script>

<div class="device-card">
  <div class="device-header">
    <div class="device-name-row">
      <span class="status-dot" class:connected={device.connected}></span>
      <h3 class="device-name">{device.device.name}</h3>
    </div>
    <span class="connection-status">
      {device.connected ? "Connected" : "Disconnected"}
    </span>
  </div>

  <div class="device-info">
    {#if device.device.capacity_bytes}
      <div class="info-row">
        <span class="info-label">Capacity</span>
        <span class="info-value">{formatSize(device.device.capacity_bytes)}</span>
      </div>
    {/if}
    <div class="info-row">
      <span class="info-label">Selection</span>
      <span class="info-value">
        {#if device.selected_artists.length === 0 && device.selected_albums.length === 0}
          None selected
        {:else}
          {device.selected_artists.length} artist{device.selected_artists.length !== 1 ? "s" : ""}{#if device.selected_albums.length > 0}, {device.selected_albums.length} album{device.selected_albums.length !== 1 ? "s" : ""}{/if}
        {/if}
      </span>
    </div>
    {#if device.device.music_folder}
      <div class="info-row">
        <span class="info-label">Music folder</span>
        <span class="info-value">{device.device.music_folder}</span>
      </div>
    {/if}
    {#if device.device.last_synced_at}
      <div class="info-row">
        <span class="info-label">Last synced</span>
        <span class="info-value">{formatDate(device.device.last_synced_at)}</span>
      </div>
    {/if}
  </div>

  <div class="device-actions">
    <button class="secondary" onclick={onConfigure}>Configure</button>
    <button
      class="primary"
      onclick={onSync}
      disabled={!device.connected || (device.selected_artists.length === 0 && device.selected_albums.length === 0) || busy}
    >
      {busy ? "In Progress..." : "Sync"}
    </button>
    <button class="danger-btn" onclick={onDelete}>Delete</button>
  </div>
</div>

<style>
  .device-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .device-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .device-name-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--danger);
    flex-shrink: 0;
  }

  .status-dot.connected {
    background: var(--success);
  }

  .device-name {
    font-size: 16px;
    font-weight: 600;
  }

  .connection-status {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .device-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .info-label {
    color: var(--text-secondary);
  }

  .info-value {
    color: var(--text-primary);
  }

  .device-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .danger-btn {
    background: none;
    color: var(--danger);
    border: 1px solid var(--danger);
    padding: 6px 12px;
    font-size: 13px;
  }

  .danger-btn:hover {
    background: rgba(233, 69, 96, 0.1);
  }
</style>
