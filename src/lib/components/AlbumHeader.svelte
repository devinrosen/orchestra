<script lang="ts">
  let {
    albumName,
    artistName,
    year,
    trackCount,
    expanded,
    boldName = false,
    onToggle,
    onPlay,
    onEdit,
  }: {
    albumName: string;
    artistName?: string;
    year: number | null;
    trackCount: number;
    expanded: boolean;
    boldName?: boolean;
    onToggle: () => void;
    onPlay?: () => void;
    onEdit?: () => void;
  } = $props();
</script>

<div class="album-header">
  <button class="tree-toggle" onclick={onToggle}>
    <span class="chevron" class:expanded>&#9654;</span>
    <span class="album-name" class:bold={boldName}>{albumName}</span>
    {#if artistName}
      <span class="album-artist">{artistName}</span>
    {/if}
    {#if year}<span class="year">({year})</span>{/if}
    <span class="count">{trackCount} track{trackCount !== 1 ? "s" : ""}</span>
  </button>
  {#if onPlay}
    <button
      class="action-btn play-album-btn"
      onclick={(e) => { e.stopPropagation(); onPlay(); }}
      title="Play album"
    >&#9654;</button>
  {/if}
  {#if onEdit}
    <button
      class="action-btn edit-btn"
      onclick={(e) => { e.stopPropagation(); onEdit(); }}
      title="Edit album metadata"
    >&#9998;</button>
  {/if}
</div>

<style>
  .album-header {
    display: flex;
    align-items: center;
  }

  .album-header .tree-toggle {
    flex: 1;
  }

  .album-name {
    font-weight: 500;
  }

  .album-name.bold {
    font-weight: 600;
  }

  .album-artist {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .year {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 4px 8px;
    border-radius: var(--radius);
    opacity: 0;
    transition: opacity 0.15s;
  }

  .album-header:hover .action-btn {
    opacity: 1;
  }

  .action-btn:hover {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .play-album-btn {
    font-size: 12px;
  }
</style>
