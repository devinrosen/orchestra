<script lang="ts">
  import { organizeStore } from "../lib/stores/organize.svelte";
  import { libraryStore } from "../lib/stores/library.svelte";
  import OrganizePatternEditor from "../lib/components/OrganizePatternEditor.svelte";
  import OrganizePreview from "../lib/components/OrganizePreview.svelte";
  import { onMount } from "svelte";

  onMount(() => {
    organizeStore.init();
  });

  function handleApply(excludedIds: Set<number>) {
    if (!libraryStore.libraryRoot) return;
    // Filter out excluded items from the preview before applying
    if (organizeStore.preview) {
      const filteredItems = organizeStore.preview.items.filter(
        (item) => item.status.type === "Ok" && !excludedIds.has(item.track_id),
      );
      if (filteredItems.length === 0) return;
      // Temporarily replace preview items to only apply non-excluded Ok items
      const originalItems = organizeStore.preview.items;
      organizeStore.preview = {
        ...organizeStore.preview,
        items: filteredItems,
      };
      organizeStore.applyOrganize(libraryStore.libraryRoot).then(() => {
        // Restore original preview so user can see full list with results
        if (organizeStore.preview) {
          organizeStore.preview = {
            ...organizeStore.preview,
            items: originalItems,
          };
        }
      });
    }
  }
</script>

<div class="organize-page">
  <h2>Organize Library</h2>
  <p class="description">
    Rename and move files in your library to match a naming pattern based on
    track metadata. Preview changes before applying.
  </p>

  {#if !libraryStore.libraryRoot}
    <div class="no-library">
      No library loaded. Scan a directory in the Library tab first.
    </div>
  {:else}
    <div class="editor-section">
      <OrganizePatternEditor
        pattern={organizeStore.pattern}
        onPatternChange={(p) => {
          organizeStore.pattern = p;
        }}
      />
    </div>

    <div class="actions-row">
      <button
        class="preview-btn"
        disabled={organizeStore.previewing || organizeStore.applying}
        onclick={() => organizeStore.previewOrganize(libraryStore.libraryRoot)}
      >
        {organizeStore.previewing ? "Previewing…" : "Preview Changes"}
      </button>

      {#if organizeStore.preview}
        <button
          class="reset-btn"
          disabled={organizeStore.applying}
          onclick={() => organizeStore.resetPreview()}
        >
          Reset
        </button>
      {/if}
    </div>

    {#if organizeStore.error}
      <div class="error-msg">{organizeStore.error}</div>
    {/if}

    {#if organizeStore.preview}
      <OrganizePreview
        preview={organizeStore.preview}
        applying={organizeStore.applying}
        progress={organizeStore.progress}
        result={organizeStore.result}
        onApply={handleApply}
      />
    {/if}
  {/if}
</div>

<style>
  .organize-page {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1000px;
  }

  h2 {
    margin: 0;
    font-size: 20px;
  }

  .description {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .no-library {
    padding: 20px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 14px;
  }

  .editor-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
  }

  .actions-row {
    display: flex;
    gap: 8px;
  }

  .preview-btn {
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

  .preview-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .preview-btn:not(:disabled):hover {
    opacity: 0.85;
  }

  .reset-btn {
    padding: 8px 16px;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .reset-btn:hover:not(:disabled) {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .reset-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .error-msg {
    padding: 8px 12px;
    background: #ef444420;
    border: 1px solid #ef4444;
    border-radius: var(--radius);
    color: #ef4444;
    font-size: 13px;
  }
</style>
