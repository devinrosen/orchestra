<script lang="ts">
  import { onDestroy } from "svelte";
  import { deviceStore } from "../stores/device.svelte";
  import { syncStore } from "../stores/sync.svelte";
  import { libraryStore } from "../stores/library.svelte";

  let { onNavigate }: { onNavigate: (page: string) => void } = $props();

  type ExpandedSection = "scan" | "device" | "profile" | null;
  let expandedSection = $state<ExpandedSection>(null);

  // Tick for elapsed time display
  let now = $state(Date.now());
  let intervalId: ReturnType<typeof setInterval> | undefined;

  $effect(() => {
    if (expandedSection) {
      intervalId = setInterval(() => { now = Date.now(); }, 1000);
    } else {
      clearInterval(intervalId);
    }
    return () => clearInterval(intervalId);
  });

  onDestroy(() => clearInterval(intervalId));

  function formatElapsed(startedAt: number | null): string {
    if (!startedAt) return "0:00";
    const seconds = Math.floor((now - startedAt) / 1000);
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function toggleExpand(section: ExpandedSection) {
    expandedSection = expandedSection === section ? null : section;
  }

  let scanActive = $derived(libraryStore.scanning);

  let scanLabel = $derived.by(() => {
    if (!scanActive) return "";
    const p = libraryStore.scanProgress;
    if (p.filesProcessed === 0 && p.filesFound === 0) return "Scanning library...";
    return `Scanning: ${p.filesProcessed} processed`;
  });

  let scanProgress = $derived.by(() => {
    const p = libraryStore.scanProgress;
    if (p.dirsTotal > 0 && p.dirsCompleted > 0) {
      return (p.dirsCompleted / p.dirsTotal) * 100;
    }
    if (p.filesFound > 0 && p.filesProcessed > 0) {
      return (p.filesProcessed / p.filesFound) * 100;
    }
    return -1; // indeterminate
  });

  let deviceActive = $derived(
    deviceStore.syncPhase === "computing_diff" ||
    deviceStore.syncPhase === "syncing",
  );

  let profileActive = $derived(
    syncStore.phase === "computing_diff" ||
    syncStore.phase === "syncing",
  );

  let visible = $derived(scanActive || deviceActive || profileActive);

  let deviceLabel = $derived.by(() => {
    if (!deviceActive) return "";
    const name = deviceStore.selectedDevice?.device.name ?? "Device";
    if (deviceStore.syncPhase === "computing_diff") {
      if (deviceStore.diffProgress.phase === "scanning") {
        return `${name}: Scanning`;
      }
      const pct = deviceStore.diffProgress.totalFiles > 0
        ? Math.round((deviceStore.diffProgress.filesCompared / deviceStore.diffProgress.totalFiles) * 100)
        : 0;
      return `${name}: Comparing ${pct}%`;
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

  // Auto-collapse when visibility changes or specific section becomes inactive
  $effect(() => {
    if (!visible) expandedSection = null;
  });

  $effect(() => {
    if (expandedSection === "scan" && !scanActive) expandedSection = null;
    if (expandedSection === "device" && !deviceActive) expandedSection = null;
    if (expandedSection === "profile" && !profileActive) expandedSection = null;
  });
</script>

{#if visible}
  <div class="status-bar">
    {#if scanActive}
      <button
        class="status-item"
        class:expanded={expandedSection === "scan"}
        onclick={() => toggleExpand("scan")}
      >
        <span class="status-label">{scanLabel}</span>
        <div class="status-track">
          {#if scanProgress >= 0}
            <div class="status-fill" style="width: {scanProgress}%"></div>
          {:else}
            <div class="status-fill indeterminate"></div>
          {/if}
        </div>
        <span class="chevron" class:open={expandedSection === "scan"}>&#9662;</span>
      </button>
    {/if}
    {#if deviceActive}
      <button
        class="status-item"
        class:expanded={expandedSection === "device"}
        onclick={() => toggleExpand("device")}
      >
        <span class="status-label">{deviceLabel}</span>
        <div class="status-track">
          {#if deviceProgress >= 0}
            <div class="status-fill" style="width: {deviceProgress}%"></div>
          {:else}
            <div class="status-fill indeterminate"></div>
          {/if}
        </div>
        <span class="chevron" class:open={expandedSection === "device"}>&#9662;</span>
      </button>
    {/if}
    {#if profileActive}
      <button
        class="status-item"
        class:expanded={expandedSection === "profile"}
        onclick={() => toggleExpand("profile")}
      >
        <span class="status-label">{profileLabel}</span>
        <div class="status-track">
          {#if profileProgress >= 0}
            <div class="status-fill" style="width: {profileProgress}%"></div>
          {:else}
            <div class="status-fill indeterminate"></div>
          {/if}
        </div>
        <span class="chevron" class:open={expandedSection === "profile"}>&#9662;</span>
      </button>
    {/if}
  </div>

  {#if expandedSection === "scan"}
    <div class="detail-panel">
      <div class="current-file" title={libraryStore.scanProgress.currentFile}>
        {libraryStore.scanProgress.currentFile || "Waiting..."}
      </div>
      <div class="stat">
        <span class="stat-label">Files</span>
        <span class="stat-value">{libraryStore.scanProgress.filesProcessed} / {libraryStore.scanProgress.filesFound}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Directories</span>
        <span class="stat-value">{libraryStore.scanProgress.dirsCompleted} / {libraryStore.scanProgress.dirsTotal}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Elapsed</span>
        <span class="stat-value">{formatElapsed(libraryStore.scanStartedAt)}</span>
      </div>
      <div class="detail-actions">
        <button class="link-btn" onclick={() => onNavigate("library")}>Go to Library</button>
        <button class="link-btn" onclick={() => expandedSection = null}>Collapse</button>
      </div>
    </div>
  {/if}

  {#if expandedSection === "device"}
    <div class="detail-panel">
      <div class="current-file" title={deviceStore.syncPhase === "syncing" ? deviceStore.syncProgress.currentFile : deviceStore.diffProgress.currentFile}>
        {#if deviceStore.syncPhase === "syncing"}
          {deviceStore.syncProgress.currentFile || "Waiting..."}
        {:else}
          {deviceStore.diffProgress.currentFile || "Waiting..."}
        {/if}
      </div>
      <div class="stat">
        <span class="stat-label">Phase</span>
        <span class="stat-value">
          {#if deviceStore.syncPhase === "computing_diff"}
            {deviceStore.diffProgress.phase === "scanning" ? "Scanning device" : "Comparing files"}
          {:else}
            Syncing files
          {/if}
        </span>
      </div>
      {#if deviceStore.syncPhase === "computing_diff"}
        <div class="stat">
          <span class="stat-label">Files</span>
          <span class="stat-value">
            {#if deviceStore.diffProgress.phase === "scanning"}
              {deviceStore.diffProgress.filesFound} found
            {:else}
              {deviceStore.diffProgress.filesCompared} / {deviceStore.diffProgress.totalFiles}
            {/if}
          </span>
        </div>
      {:else}
        <div class="stat">
          <span class="stat-label">Files</span>
          <span class="stat-value">{deviceStore.syncProgress.filesCompleted} / {deviceStore.syncProgress.totalFiles}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Transferred</span>
          <span class="stat-value">{formatSize(deviceStore.syncProgress.bytesCompleted)} / {formatSize(deviceStore.syncProgress.totalBytes)}</span>
        </div>
      {/if}
      <div class="stat">
        <span class="stat-label">Elapsed</span>
        <span class="stat-value">{formatElapsed(deviceStore.startedAt)}</span>
      </div>
      <div class="detail-actions">
        <button class="link-btn" onclick={() => onNavigate("devices")}>Go to Devices</button>
        <button class="link-btn" onclick={() => expandedSection = null}>Collapse</button>
      </div>
    </div>
  {/if}

  {#if expandedSection === "profile"}
    <div class="detail-panel">
      <div class="current-file" title={syncStore.progress.currentFile}>
        {syncStore.progress.currentFile || "Waiting..."}
      </div>
      {#if syncStore.phase === "syncing"}
        <div class="stat">
          <span class="stat-label">Files</span>
          <span class="stat-value">{syncStore.progress.filesCompleted} / {syncStore.progress.totalFiles}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Transferred</span>
          <span class="stat-value">{formatSize(syncStore.progress.bytesCompleted)} / {formatSize(syncStore.progress.totalBytes)}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Elapsed</span>
          <span class="stat-value">{formatElapsed(syncStore.syncStartedAt)}</span>
        </div>
      {:else}
        <div class="stat">
          <span class="stat-label">Phase</span>
          <span class="stat-value">Computing diff...</span>
        </div>
      {/if}
      <div class="detail-actions">
        <button class="link-btn" onclick={() => onNavigate("sync-preview")}>Go to Sync Preview</button>
        <button class="link-btn" onclick={() => expandedSection = null}>Collapse</button>
      </div>
    </div>
  {/if}
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

  .status-item.expanded {
    background: var(--bg-tertiary);
  }

  .status-label {
    white-space: nowrap;
    flex-shrink: 0;
  }

  .chevron {
    font-size: 10px;
    color: var(--text-secondary);
    transition: transform 0.15s ease;
    flex-shrink: 0;
  }

  .chevron.open {
    transform: rotate(180deg);
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

  .detail-panel {
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    padding: 12px 16px;
    font-size: 13px;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px 24px;
    animation: expandDown 0.15s ease-out;
    flex-shrink: 0;
  }

  @keyframes expandDown {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .current-file {
    grid-column: 1 / -1;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 12px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-label {
    color: var(--text-secondary);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: 14px;
    font-weight: 500;
  }

  .detail-actions {
    grid-column: 1 / -1;
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 4px;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 12px;
    padding: 4px 0;
    cursor: pointer;
    border-radius: 0;
  }

  .link-btn:hover {
    color: var(--accent-hover);
    text-decoration: underline;
  }
</style>
