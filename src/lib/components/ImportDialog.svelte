<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import type { ProgressEvent } from "../api/types";
  import { importTracks } from "../api/commands";
  import ProgressBar from "./ProgressBar.svelte";

  let {
    libraryRoot,
    onClose,
    onImported,
  }: {
    libraryRoot: string;
    onClose: () => void;
    onImported: (count: number) => void;
  } = $props();

  let importing = $state(false);
  let progress = $state({ filesProcessed: 0, filesFound: 0, currentFile: "" });
  let error = $state<string | null>(null);
  let result = $state<number | null>(null);

  async function chooseFiles() {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Audio",
          extensions: ["flac", "mp3", "m4a", "aac", "wav", "alac", "ogg", "opus", "wma"],
        },
      ],
    });

    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths.length === 0) return;

    importing = true;
    error = null;
    result = null;
    progress = { filesProcessed: 0, filesFound: paths.length, currentFile: "" };

    try {
      const count = await importTracks(paths, libraryRoot, (event: ProgressEvent) => {
        if (event.type === "scan_progress") {
          progress = {
            filesProcessed: event.files_processed,
            filesFound: event.files_found,
            currentFile: event.current_file,
          };
        }
      });
      result = count;
      onImported(count);
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="import-overlay" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="import-dialog" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="dialog-header">
      <h3>Import Files</h3>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    {#if importing}
      <div class="progress-section">
        <ProgressBar
          value={progress.filesProcessed}
          max={progress.filesFound}
          label="Importing: {progress.currentFile}"
        />
        <p class="progress-count">
          {progress.filesProcessed} of {progress.filesFound} files
        </p>
      </div>
    {:else if error}
      <div class="error-banner">{error}</div>
      <div class="dialog-actions">
        <button class="secondary" onclick={() => { error = null; }}>Try Again</button>
        <button class="secondary" onclick={onClose}>Close</button>
      </div>
    {:else if result !== null}
      <div class="result-banner">
        {result} {result === 1 ? "track" : "tracks"} imported successfully.
      </div>
      <div class="dialog-actions">
        <button class="primary" onclick={onClose}>Done</button>
      </div>
    {:else}
      <p class="dialog-description">
        Select audio files to copy into your library at:
      </p>
      <p class="library-path">{libraryRoot}</p>
      <div class="dialog-actions">
        <button class="secondary" onclick={onClose}>Cancel</button>
        <button class="primary" onclick={chooseFiles}>Choose Files</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .import-overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .import-dialog {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    width: 480px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: var(--overlay-shadow);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dialog-header h3 {
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

  .dialog-description {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .library-path {
    font-family: monospace;
    font-size: 12px;
    background: var(--bg-secondary);
    padding: 6px 10px;
    border-radius: var(--radius);
    color: var(--text-secondary);
    word-break: break-all;
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-count {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .error-banner {
    background: var(--accent-tint-strong);
    color: var(--danger);
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .result-banner {
    background: var(--success-tint);
    color: var(--success);
    padding: 10px 14px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
