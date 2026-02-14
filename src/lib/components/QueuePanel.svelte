<script lang="ts">
  import { onMount } from "svelte";
  import { playerStore } from "../stores/player.svelte";
  import { formatDuration } from "../utils/format";

  let { onClose }: { onClose: () => void } = $props();

  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let dragging = $state(false);
  let listEl: HTMLDivElement;

  onMount(() => {
    // Auto-scroll to the currently playing track
    if (listEl) {
      const currentRow = listEl.querySelector(".now-playing");
      if (currentRow) {
        currentRow.scrollIntoView({ block: "center" });
      }
    }
  });

  function handlePointerDown(e: PointerEvent, index: number) {
    const target = e.target as HTMLElement;
    if (!target.closest(".drag-handle")) return;
    e.preventDefault();
    dragIndex = index;
    dragging = true;

    const onPointerMove = (me: PointerEvent) => {
      const el = document.elementFromPoint(me.clientX, me.clientY);
      if (el) {
        const row = el.closest("[data-track-index]") as HTMLElement | null;
        if (row) {
          dragOverIndex = Number(row.dataset.trackIndex);
        }
      }
    };

    const onPointerUp = () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      dragging = false;

      if (
        dragIndex !== null &&
        dragOverIndex !== null &&
        dragIndex !== dragOverIndex
      ) {
        playerStore.moveInQueue(dragIndex, dragOverIndex);
      }
      dragIndex = null;
      dragOverIndex = null;
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  function handleClick(index: number) {
    playerStore.jumpTo(index);
  }

  function handleRemove(e: Event, index: number) {
    e.stopPropagation();
    playerStore.removeFromQueue(index);
    if (playerStore.queue.length === 0) {
      onClose();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="queue-backdrop" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="queue-panel" onclick={(e) => e.stopPropagation()}>
    <div class="queue-header">
      <span class="queue-title">Play Queue</span>
      <button class="close-btn" onclick={onClose} title="Close">x</button>
    </div>
    <div class="queue-info">{playerStore.queue.length} tracks</div>
    <div class="queue-list" bind:this={listEl}>
      {#each playerStore.queue as track, i}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="queue-track-row"
          class:drag-over={dragOverIndex === i && dragIndex !== null && dragIndex !== i}
          class:dragging-source={dragIndex === i && dragging}
          class:now-playing={playerStore.queueIndex === i}
          data-track-index={i}
          onpointerdown={(e) => handlePointerDown(e, i)}
          onclick={() => handleClick(i)}
        >
          <span class="drag-handle">&#x2630;</span>
          <span class="track-pos">{i + 1}</span>
          <span class="track-title">{track.title ?? track.relative_path}</span>
          <span class="track-artist">{track.artist ?? ""}</span>
          <span class="track-duration">{formatDuration(track.duration_secs)}</span>
          <button class="remove-btn" onclick={(e) => handleRemove(e, i)} title="Remove from queue">x</button>
        </div>
      {/each}
      {#if playerStore.queue.length === 0}
        <div class="empty-hint">No tracks in queue</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .queue-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 150;
    background: var(--overlay-bg);
  }

  .queue-panel {
    position: fixed;
    bottom: 72px;
    right: 16px;
    width: 380px;
    max-height: 60vh;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    box-shadow: var(--overlay-shadow);
  }

  .queue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }

  .queue-title {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 2px 6px;
    cursor: pointer;
    border-radius: var(--radius);
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .queue-info {
    font-size: 12px;
    color: var(--text-secondary);
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
  }

  .queue-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .queue-track-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    font-size: 13px;
    cursor: pointer;
  }

  .queue-track-row:hover {
    background: var(--bg-tertiary);
  }

  .queue-track-row.drag-over {
    border-top: 2px solid var(--accent);
  }

  .queue-track-row.dragging-source {
    opacity: 0.4;
  }

  .queue-track-row.now-playing .track-title {
    color: var(--accent);
    font-weight: 600;
  }

  .drag-handle {
    cursor: grab;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 0 4px;
    opacity: 0.5;
    user-select: none;
    touch-action: none;
  }

  .queue-track-row:hover .drag-handle {
    opacity: 1;
  }

  .track-pos {
    color: var(--text-secondary);
    width: 24px;
    text-align: right;
    flex-shrink: 0;
  }

  .track-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track-artist {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track-duration {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .remove-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 2px 6px;
    opacity: 0;
    transition: opacity 0.15s;
    cursor: pointer;
  }

  .queue-track-row:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    color: var(--danger);
  }

  .empty-hint {
    padding: 16px 12px;
    color: var(--text-secondary);
    font-size: 13px;
    text-align: center;
  }
</style>
