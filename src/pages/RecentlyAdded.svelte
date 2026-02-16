<script lang="ts">
  import { onMount } from "svelte";
  import type { Track } from "../lib/api/types";
  import { getRecentlyAdded, getRecentlyPlayed, getSetting, setSetting } from "../lib/api/commands";
  import { libraryStore } from "../lib/stores/library.svelte";
  import { playerStore } from "../lib/stores/player.svelte";
  import TrackRow from "../lib/components/TrackRow.svelte";
  import MetadataEditor from "../lib/components/MetadataEditor.svelte";

  let recentlyAdded = $state<Track[]>([]);
  let recentlyPlayed = $state<Track[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let days = $state(30);
  let expandedSections = $state<Set<string>>(new Set(["added", "played"]));
  let editingTrack = $state<Track | null>(null);

  const DAY_OPTIONS = [7, 14, 30, 90];

  async function fetchRecentlyAdded() {
    try {
      recentlyAdded = await getRecentlyAdded(days, 200);
    } catch (e) {
      error = String(e);
    }
  }

  async function fetchAll() {
    loading = true;
    error = null;
    try {
      [recentlyAdded, recentlyPlayed] = await Promise.all([
        getRecentlyAdded(days, 200),
        getRecentlyPlayed(50),
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    const savedDays = await getSetting("recently_added_days");
    if (savedDays) {
      const parsed = parseInt(savedDays, 10);
      if (DAY_OPTIONS.includes(parsed)) {
        days = parsed;
      }
    }
    await fetchAll();
  });

  async function handleDaysChange(newDays: number) {
    days = newDays;
    await setSetting("recently_added_days", String(newDays));
    await fetchRecentlyAdded();
  }

  function toggleSection(section: string) {
    const next = new Set(expandedSections);
    if (next.has(section)) next.delete(section);
    else next.add(section);
    expandedSections = next;
  }

  function handleEditTrack(track: Track) {
    editingTrack = track;
  }

  async function handleTrackSaved() {
    editingTrack = null;
    if (libraryStore.libraryRoot) {
      await libraryStore.loadTree(libraryStore.libraryRoot);
    }
  }
</script>

<div class="recent-page">
  <h2>Recent</h2>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else}
    <div class="recent-section">
      <button class="section-header" onclick={() => toggleSection("added")}>
        <span class="chevron" class:expanded={expandedSections.has("added")}>&#9654;</span>
        <span class="section-title">Recently Added</span>
        <span class="section-count">{recentlyAdded.length}</span>
      </button>
      {#if expandedSections.has("added")}
        <div class="section-content">
          <div class="days-selector">
            <span class="days-label">Last</span>
            {#each DAY_OPTIONS as d}
              <button
                class="days-btn"
                class:active={days === d}
                onclick={() => handleDaysChange(d)}
              >{d}d</button>
            {/each}
          </div>
          {#if recentlyAdded.length === 0}
            <div class="empty-hint">No tracks added in the last {days} days</div>
          {:else}
            {#each recentlyAdded as track}
              <TrackRow
                {track}
                siblingTracks={recentlyAdded}
                onPlay={(t, siblings) => playerStore.playTrack(t, siblings)}
                onEdit={handleEditTrack}
              />
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <div class="recent-section">
      <button class="section-header" onclick={() => toggleSection("played")}>
        <span class="chevron" class:expanded={expandedSections.has("played")}>&#9654;</span>
        <span class="section-title">Recently Played</span>
        <span class="section-count">{recentlyPlayed.length}</span>
      </button>
      {#if expandedSections.has("played")}
        <div class="section-content">
          {#if recentlyPlayed.length === 0}
            <div class="empty-hint">No recently played tracks</div>
          {:else}
            {#each recentlyPlayed as track}
              <TrackRow
                {track}
                siblingTracks={recentlyPlayed}
                onPlay={(t, siblings) => playerStore.playTrack(t, siblings)}
                onEdit={handleEditTrack}
              />
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  {#if editingTrack}
    <MetadataEditor
      track={editingTrack}
      onSave={handleTrackSaved}
      onClose={() => (editingTrack = null)}
    />
  {/if}
</div>

<style>
  .recent-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 8px;
  }

  .recent-page h2 {
    font-size: 20px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .loading {
    color: var(--text-secondary);
    padding: 24px;
    text-align: center;
  }

  .error {
    color: var(--text-secondary);
    padding: 24px;
    text-align: center;
  }

  .recent-section {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    background: var(--bg-secondary);
    border: none;
    color: var(--text-primary);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
  }

  .section-header:hover {
    background: var(--bg-tertiary);
  }

  .section-title {
    font-weight: 600;
    flex: 1;
  }

  .section-count {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .section-content {
    padding: 4px 8px;
  }

  .empty-hint {
    padding: 12px 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .chevron {
    display: inline-block;
    font-size: 10px;
    transition: transform 0.15s;
    color: var(--text-secondary);
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .days-selector {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
  }

  .days-label {
    font-size: 12px;
    color: var(--text-secondary);
    margin-right: 4px;
  }

  .days-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-secondary);
    font-size: 12px;
    padding: 2px 8px;
    cursor: pointer;
  }

  .days-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .days-btn.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--bg-primary);
  }
</style>
