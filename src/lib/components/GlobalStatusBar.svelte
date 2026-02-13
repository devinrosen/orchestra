<script lang="ts">
  import { deviceStore } from "../stores/device.svelte";
  import { syncStore } from "../stores/sync.svelte";

  let { onNavigate }: { onNavigate: (page: string) => void } = $props();

  let deviceActive = $derived(
    deviceStore.syncPhase === "computing_diff" ||
    deviceStore.syncPhase === "syncing",
  );

  let profileActive = $derived(
    syncStore.phase === "computing_diff" ||
    syncStore.phase === "syncing",
  );

  let visible = $derived(deviceActive || profileActive);

  let deviceLabel = $derived.by(() => {
    if (!deviceActive) return "";
    const name = deviceStore.selectedDevice?.device.name ?? "Device";
    if (deviceStore.syncPhase === "computing_diff") {
      if (deviceStore.diffProgress.phase === "scanning") {
        return `${name}: Scanning device — ${deviceStore.diffProgress.filesFound} files`;
      }
      const pct = deviceStore.diffProgress.totalFiles > 0
        ? Math.round((deviceStore.diffProgress.filesCompared / deviceStore.diffProgress.totalFiles) * 100)
        : 0;
      return `${name}: Comparing files ${pct}%`;
    }
    if (deviceStore.syncPhase === "syncing") {
      const pct = deviceStore.syncProgress.totalFiles > 0
        ? Math.round((deviceStore.syncProgress.filesCompleted / deviceStore.syncProgress.totalFiles) * 100)
        : 0;
      return `${name}: Syncing ${pct}%`;
    }
    return "";
  });

  let deviceProgress = $derived.by(() => {
    if (deviceStore.syncPhase === "computing_diff" && deviceStore.diffProgress.phase === "comparing") {
      return deviceStore.diffProgress.totalFiles > 0
        ? (deviceStore.diffProgress.filesCompared / deviceStore.diffProgress.totalFiles) * 100
        : 0;
    }
    if (deviceStore.syncPhase === "syncing") {
      return deviceStore.syncProgress.totalFiles > 0
        ? (deviceStore.syncProgress.filesCompleted / deviceStore.syncProgress.totalFiles) * 100
        : 0;
    }
    return -1; // indeterminate
  });

  let profileLabel = $derived.by(() => {
    if (!profileActive) return "";
    if (syncStore.phase === "computing_diff") return "Profile: Computing diff...";
    if (syncStore.phase === "syncing") {
      const pct = syncStore.progress.totalFiles > 0
        ? Math.round((syncStore.progress.filesCompleted / syncStore.progress.totalFiles) * 100)
        : 0;
      return `Profile: Syncing ${pct}%`;
    }
    return "";
  });

  let profileProgress = $derived.by(() => {
    if (syncStore.phase === "syncing") {
      return syncStore.progress.totalFiles > 0
        ? (syncStore.progress.filesCompleted / syncStore.progress.totalFiles) * 100
        : 0;
    }
    return -1;
  });
</script>

{#if visible}
  <div class="status-bar">
    {#if deviceActive}
      <button class="status-item" onclick={() => onNavigate("devices")}>
        <span class="status-label">{deviceLabel}</span>
        <div class="status-track">
          {#if deviceProgress >= 0}
            <div class="status-fill" style="width: {deviceProgress}%"></div>
          {:else}
            <div class="status-fill indeterminate"></div>
          {/if}
        </div>
      </button>
    {/if}
    {#if profileActive}
      <button class="status-item" onclick={() => onNavigate("sync-preview")}>
        <span class="status-label">{profileLabel}</span>
        <div class="status-track">
          {#if profileProgress >= 0}
            <div class="status-fill" style="width: {profileProgress}%"></div>
          {:else}
            <div class="status-fill indeterminate"></div>
          {/if}
        </div>
      </button>
    {/if}
  </div>
{/if}

<style>
  .status-bar {
    display: flex;
    gap: 1px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .status-item {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px;
    background: var(--bg-secondary);
    border: none;
    border-radius: 0;
    color: var(--text-primary);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .status-item:hover {
    background: var(--bg-tertiary);
  }

  .status-label {
    white-space: nowrap;
    flex-shrink: 0;
  }

  .status-track {
    flex: 1;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    overflow: hidden;
    min-width: 60px;
  }

  .status-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .status-fill.indeterminate {
    width: 30%;
    animation: slide 1.2s ease-in-out infinite;
  }

  @keyframes slide {
    0% { transform: translateX(-100%); }
    50% { transform: translateX(230%); }
    100% { transform: translateX(-100%); }
  }
</style>
