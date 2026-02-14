<script lang="ts">
  import { onMount } from "svelte";
  import DeviceCard from "../lib/components/DeviceCard.svelte";
  import ArtistPicker from "../lib/components/ArtistPicker.svelte";
  import DiffView from "../lib/components/DiffView.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";
  import { deviceStore } from "../lib/stores/device.svelte";
  import type { DetectedVolume, AlbumSelection } from "../lib/api/types";

  type SubView = "list" | "configure" | "sync";
  let subView = $state<SubView>("list");
  let registerName = $state("");
  let registerMusicFolder = $state("");
  let registeringVolume = $state<DetectedVolume | null>(null);
  let configuringDeviceId = $state<string | null>(null);
  let ejectingDeviceId = $state<string | null>(null);

  onMount(() => {
    deviceStore.loadDevices();
  });

  function handleDetect() {
    deviceStore.detectVolumes();
  }

  function startRegister(vol: DetectedVolume) {
    registeringVolume = vol;
    registerName = vol.volume_name;
    registerMusicFolder = "";
  }

  async function confirmRegister() {
    if (!registeringVolume) return;
    await deviceStore.registerDevice({
      name: registerName,
      volume_uuid: registeringVolume.volume_uuid,
      volume_name: registeringVolume.volume_name,
      mount_path: registeringVolume.mount_path,
      capacity_bytes: registeringVolume.capacity_bytes || null,
      music_folder: registerMusicFolder,
    });
    registeringVolume = null;
  }

  function handleConfigure(deviceId: string) {
    configuringDeviceId = deviceId;
    deviceStore.loadArtists();
    deviceStore.loadAlbums();
    subView = "configure";
  }

  async function handleSaveSelection(artists: string[], albums: AlbumSelection[]) {
    if (!configuringDeviceId) return;
    await deviceStore.setArtists(configuringDeviceId, artists);
    await deviceStore.setAlbums(configuringDeviceId, albums);
    subView = "list";
  }

  function handleCancelConfigure() {
    subView = "list";
  }

  async function handleSync(deviceId: string) {
    deviceStore.selectDevice(deviceId);
    subView = "sync";
    // If already computing or syncing, just show the sync view (don't restart)
    if (
      deviceStore.syncPhase === "computing_diff" ||
      deviceStore.syncPhase === "syncing" ||
      deviceStore.syncPhase === "previewing"
    ) {
      return;
    }
    await deviceStore.computeDiff(deviceId);
  }

  async function handleExecuteSync() {
    if (!deviceStore.selectedDeviceId) return;
    await deviceStore.executeSync(deviceStore.selectedDeviceId);
  }

  function handleCancelSync() {
    deviceStore.cancelSync();
  }

  function handleSyncDone() {
    deviceStore.resetSync();
    subView = "list";
    deviceStore.loadDevices();
  }

  async function handleDelete(deviceId: string) {
    await deviceStore.deleteDevice(deviceId);
  }

  function handleEjectRequest(deviceId: string) {
    ejectingDeviceId = deviceId;
  }

  async function confirmEject() {
    if (!ejectingDeviceId) return;
    await deviceStore.ejectDevice(ejectingDeviceId);
    ejectingDeviceId = null;
  }

  function cancelEject() {
    ejectingDeviceId = null;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  let isBusy = $derived(
    deviceStore.syncPhase === "computing_diff" ||
    deviceStore.syncPhase === "syncing",
  );

  let unregisteredVolumes = $derived(
    deviceStore.detectedVolumes.filter((v) => !v.already_registered),
  );
</script>

<div class="device-page">
  {#if subView === "list"}
    <div class="page-header">
      <h2>Devices</h2>
      <button
        class="primary"
        onclick={handleDetect}
        disabled={deviceStore.detecting}
      >
        {deviceStore.detecting ? "Detecting..." : "Detect Devices"}
      </button>
    </div>

    {#if deviceStore.error}
      <div class="error-banner">{deviceStore.error}</div>
    {/if}

    {#if deviceStore.devices.length > 0}
      <div class="devices-grid">
        {#each deviceStore.devices as device}
          <DeviceCard
            {device}
            busy={isBusy}
            ejecting={deviceStore.ejecting === device.device.id}
            onConfigure={() => handleConfigure(device.device.id)}
            onSync={() => handleSync(device.device.id)}
            onDelete={() => handleDelete(device.device.id)}
            onEject={() => handleEjectRequest(device.device.id)}
          />
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <p>No devices registered yet.</p>
        <p class="hint">Connect your DAP and click "Detect Devices" to get started.</p>
      </div>
    {/if}

    {#if unregisteredVolumes.length > 0}
      <div class="detected-section">
        <h3>Detected Volumes</h3>
        <div class="detected-list">
          {#each unregisteredVolumes as vol}
            <div class="detected-volume">
              <div class="vol-info">
                <span class="vol-name">{vol.volume_name}</span>
                <span class="vol-meta">
                  {formatSize(vol.capacity_bytes)} &middot; {vol.bus_protocol}
                  &middot; {formatSize(vol.free_bytes)} free
                </span>
              </div>
              <button class="secondary" onclick={() => startRegister(vol)}>Register</button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if registeringVolume}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="register-dialog-overlay" role="presentation" onclick={() => (registeringVolume = null)}>
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
        <div class="register-dialog" role="dialog" onclick={(e) => e.stopPropagation()}>
          <h3>Register Device</h3>
          <label class="field">
            <span>Device Name</span>
            <input type="text" bind:value={registerName} placeholder="My DAP" />
          </label>
          <label class="field">
            <span>Music Folder (relative path, empty for root)</span>
            <input type="text" bind:value={registerMusicFolder} placeholder="e.g. Music" />
          </label>
          <div class="dialog-actions">
            <button class="secondary" onclick={() => (registeringVolume = null)}>Cancel</button>
            <button class="primary" onclick={confirmRegister} disabled={!registerName.trim()}>
              Register
            </button>
          </div>
        </div>
      </div>
    {/if}

    {#if ejectingDeviceId}
      {@const ejectDevice = deviceStore.devices.find((d) => d.device.id === ejectingDeviceId)}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="register-dialog-overlay" role="presentation" onclick={cancelEject}>
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
        <div class="register-dialog" role="dialog" onclick={(e) => e.stopPropagation()}>
          <h3>Eject Device</h3>
          <p>Are you sure you want to eject "{ejectDevice?.device.name}"?</p>
          <p class="hint">Make sure no sync is in progress before ejecting.</p>
          <div class="dialog-actions">
            <button class="secondary" onclick={cancelEject}>Cancel</button>
            <button class="primary" onclick={confirmEject}>Eject</button>
          </div>
        </div>
      </div>
    {/if}

  {:else if subView === "configure"}
    {@const confDevice = deviceStore.devices.find((d) => d.device.id === configuringDeviceId)}
    <div class="page-header">
      <button class="secondary" onclick={handleCancelConfigure}>Back</button>
      <h2>
        Configure: {confDevice?.device.name ?? "Device"}
      </h2>
    </div>

    {#if deviceStore.loadingArtists || deviceStore.loadingAlbums}
      <div class="loading">Loading artists...</div>
    {:else}
      <ArtistPicker
        artists={deviceStore.availableArtists}
        albums={deviceStore.availableAlbums}
        selectedArtists={confDevice?.selected_artists ?? []}
        selectedAlbums={confDevice?.selected_albums ?? []}
        onSave={handleSaveSelection}
        onCancel={handleCancelConfigure}
      />
    {/if}

  {:else if subView === "sync"}
    <div class="sync-view">
      {#if deviceStore.syncPhase === "computing_diff"}
        <div class="center-state">
          <h2>Computing Diff...</h2>
          {#if deviceStore.diffProgress.phase === "scanning"}
            <p>Scanning device — {deviceStore.diffProgress.filesFound} files found</p>
          {:else}
            <div class="diff-progress-wrapper">
              <ProgressBar
                value={deviceStore.diffProgress.filesCompared}
                max={deviceStore.diffProgress.totalFiles}
                label="Comparing files..."
              />
              <p class="progress-sub">
                {deviceStore.diffProgress.filesCompared} / {deviceStore.diffProgress.totalFiles} files compared
              </p>
            </div>
          {/if}
          <p class="progress-file">{deviceStore.diffProgress.currentFile}</p>
        </div>
      {:else if deviceStore.syncPhase === "previewing" && deviceStore.diffResult}
        <div class="page-header">
          <h2>Device Sync Preview</h2>
          <div class="header-actions">
            <button class="secondary" onclick={handleSyncDone}>Back</button>
            <button
              class="primary"
              onclick={handleExecuteSync}
              disabled={deviceStore.diffResult.entries.every((e) => e.action === "unchanged")}
            >
              Execute Sync
            </button>
          </div>
        </div>
        <DiffView diff={deviceStore.diffResult} />
      {:else if deviceStore.syncPhase === "syncing"}
        <div class="center-state">
          <h2>Syncing to Device...</h2>
          <div class="progress-wrapper">
            <ProgressBar
              value={deviceStore.syncProgress.filesCompleted}
              max={deviceStore.syncProgress.totalFiles}
              label={deviceStore.syncProgress.currentFile}
            />
            <div class="progress-details">
              <span>
                {deviceStore.syncProgress.filesCompleted} / {deviceStore.syncProgress.totalFiles} files
              </span>
              <span>
                {formatSize(deviceStore.syncProgress.bytesCompleted)} / {formatSize(deviceStore.syncProgress.totalBytes)}
              </span>
            </div>
          </div>
          <button class="secondary" onclick={handleCancelSync}>Cancel</button>
        </div>
      {:else if deviceStore.syncPhase === "complete"}
        <div class="center-state">
          <h2>Sync Complete</h2>
          {#if deviceStore.syncErrors.length > 0}
            <div class="sync-errors">
              <h3>Errors ({deviceStore.syncErrors.length})</h3>
              {#each deviceStore.syncErrors as err}
                <div class="error-item">
                  <span class="error-file">{err.file}</span>
                  <span class="error-msg">{err.error}</span>
                </div>
              {/each}
            </div>
          {/if}
          <button class="primary" onclick={handleSyncDone}>Done</button>
        </div>
      {:else if deviceStore.syncPhase === "error"}
        <div class="center-state">
          <h2>Error</h2>
          <p class="error-text">{deviceStore.error}</p>
          <button class="primary" onclick={handleSyncDone}>Back</button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .device-page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    height: 100%;
  }

  .page-header {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .page-header h2 {
    font-size: 20px;
    font-weight: 600;
    flex: 1;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .error-banner {
    background: rgba(233, 69, 96, 0.1);
    color: var(--danger);
    padding: 10px 14px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .devices-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
  }

  .empty-state {
    text-align: center;
    padding: 48px 16px;
    color: var(--text-secondary);
  }

  .empty-state .hint {
    font-size: 13px;
    margin-top: 8px;
  }

  .detected-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .detected-section h3 {
    font-size: 16px;
    font-weight: 600;
  }

  .detected-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .detected-volume {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .vol-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .vol-name {
    font-weight: 500;
    font-size: 14px;
  }

  .vol-meta {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .register-dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .register-dialog {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    width: 400px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .register-dialog h3 {
    font-size: 18px;
    font-weight: 600;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .hint {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
  }

  .loading {
    text-align: center;
    padding: 48px;
    color: var(--text-secondary);
  }

  .sync-view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
  }

  .center-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 12px;
  }

  .progress-wrapper {
    width: 100%;
    max-width: 500px;
  }

  .progress-details {
    display: flex;
    justify-content: space-between;
    width: 100%;
    font-size: 13px;
    color: var(--text-secondary);
    margin-top: 8px;
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
  .error-text { color: var(--danger); }

  .diff-progress-wrapper {
    width: 100%;
    max-width: 500px;
  }

  .progress-sub {
    font-size: 13px;
    color: var(--text-secondary);
    margin-top: 6px;
    text-align: center;
  }

  .progress-file {
    font-size: 12px;
    color: var(--text-secondary);
    max-width: 500px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: center;
  }
</style>
