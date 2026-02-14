<script lang="ts">
  import { playlistStore } from "../stores/playlist.svelte";

  let {
    trackIds,
    onClose,
  }: {
    trackIds: number[];
    onClose: () => void;
  } = $props();

  let newName = $state("");
  let creating = $state(false);

  async function addToPlaylist(playlistId: string) {
    await playlistStore.addTracks(playlistId, trackIds);
    onClose();
  }

  async function createAndAdd() {
    if (!newName.trim()) return;
    creating = true;
    const result = await playlistStore.create(newName.trim());
    if (result) {
      await playlistStore.addTracks(result.playlist.id, trackIds);
    }
    creating = false;
    onClose();
  }
</script>

<div class="picker-backdrop" onclick={onClose} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="picker-dropdown" onclick={(e) => e.stopPropagation()}>
    <div class="picker-header">Add to Playlist</div>
    <div class="picker-list">
      {#each playlistStore.playlists as pl}
        <button class="picker-item" onclick={() => addToPlaylist(pl.id)}>
          {pl.name}
        </button>
      {/each}
    </div>
    <div class="picker-create">
      <input
        type="text"
        placeholder="New playlist..."
        bind:value={newName}
        onkeydown={(e) => { if (e.key === "Enter") createAndAdd(); }}
      />
      <button
        class="picker-create-btn"
        onclick={createAndAdd}
        disabled={creating || !newName.trim()}
      >+</button>
    </div>
  </div>
</div>

<style>
  .picker-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 100;
  }

  .picker-dropdown {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    width: 280px;
    max-height: 360px;
    display: flex;
    flex-direction: column;
    box-shadow: var(--overlay-shadow);
  }

  .picker-header {
    padding: 10px 12px;
    font-weight: 600;
    font-size: 13px;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
  }

  .picker-list {
    overflow-y: auto;
    flex: 1;
    max-height: 240px;
  }

  .picker-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 12px;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
  }

  .picker-item:hover {
    background: var(--bg-tertiary);
  }

  .picker-create {
    display: flex;
    gap: 6px;
    padding: 8px;
    border-top: 1px solid var(--border);
  }

  .picker-create input {
    flex: 1;
    padding: 6px 8px;
    font-size: 13px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .picker-create-btn {
    padding: 6px 10px;
    background: var(--accent);
    color: var(--on-accent);
    border: none;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .picker-create-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
