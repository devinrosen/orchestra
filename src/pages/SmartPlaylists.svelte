<script lang="ts">
  import { onMount } from "svelte";
  import { smartPlaylistStore } from "../lib/stores/smartPlaylist.svelte";
  import { playerStore } from "../lib/stores/player.svelte";
  import { formatDuration } from "../lib/utils/format";
  import type { Rule, Track } from "../lib/api/types";

  // Field metadata for the rule builder
  type FieldType = 'string' | 'numeric' | 'boolean';

  const FIELD_TYPES: Record<string, FieldType> = {
    genre: 'string',
    artist: 'string',
    album_artist: 'string',
    album: 'string',
    title: 'string',
    format: 'string',
    year: 'numeric',
    bitrate: 'numeric',
    duration_secs: 'numeric',
    track_number: 'numeric',
    has_album_art: 'boolean',
  };

  const FIELD_LABELS: Record<string, string> = {
    genre: 'Genre',
    artist: 'Artist',
    album_artist: 'Album Artist',
    album: 'Album',
    title: 'Title',
    format: 'Format',
    year: 'Year',
    bitrate: 'Bitrate',
    duration_secs: 'Duration (s)',
    track_number: 'Track Number',
    has_album_art: 'Has Album Art',
  };

  const STRING_OPS: { value: string; label: string }[] = [
    { value: 'contains', label: 'contains' },
    { value: 'not_contains', label: 'does not contain' },
    { value: 'equals', label: 'equals' },
    { value: 'not_equals', label: 'does not equal' },
    { value: 'starts_with', label: 'starts with' },
    { value: 'ends_with', label: 'ends with' },
  ];

  const NUMERIC_OPS: { value: string; label: string }[] = [
    { value: 'equals', label: '=' },
    { value: 'not_equals', label: '!=' },
    { value: 'greater_than', label: '>' },
    { value: 'less_than', label: '<' },
    { value: 'greater_than_or_equal', label: '>=' },
    { value: 'less_than_or_equal', label: '<=' },
  ];

  const BOOLEAN_OPS: { value: string; label: string }[] = [
    { value: 'equals', label: 'is' },
  ];

  function getOpsForField(field: string): { value: string; label: string }[] {
    const type = FIELD_TYPES[field] ?? 'string';
    if (type === 'boolean') return BOOLEAN_OPS;
    if (type === 'numeric') return NUMERIC_OPS;
    return STRING_OPS;
  }

  function getDefaultOp(field: string): string {
    return getOpsForField(field)[0].value;
  }

  function getDefaultValue(field: string): string {
    const type = FIELD_TYPES[field] ?? 'string';
    if (type === 'boolean') return 'true';
    if (type === 'numeric') return '0';
    return '';
  }

  function makeCondition(field = 'genre'): Rule {
    return { type: 'condition', field, op: getDefaultOp(field), value: getDefaultValue(field) };
  }

  function makeGroup(): Rule {
    return { type: 'group', operator: 'and', rules: [makeCondition()] };
  }

  // ── Local state ──────────────────────────────────────────────────────
  let showCreateForm = $state(false);
  let newName = $state('');
  let editingName = $state(false);
  let renameValue = $state('');

  // The rule being edited is a copy of the selected playlist's rule
  let editingRule = $state<Rule | null>(null);

  onMount(() => {
    smartPlaylistStore.load();
  });

  // Sync editingRule when selection changes
  $effect(() => {
    if (smartPlaylistStore.selectedPlaylist) {
      editingRule = JSON.parse(JSON.stringify(smartPlaylistStore.selectedPlaylist.playlist.rule));
    } else {
      editingRule = null;
    }
  });

  async function handleCreate() {
    if (!newName.trim()) return;
    const defaultRule = makeCondition();
    await smartPlaylistStore.create(newName.trim(), defaultRule);
    newName = '';
    showCreateForm = false;
  }

  function startRename() {
    if (!smartPlaylistStore.selectedPlaylist) return;
    renameValue = smartPlaylistStore.selectedPlaylist.playlist.name;
    editingName = true;
  }

  async function finishRename() {
    if (!smartPlaylistStore.selectedPlaylist || !renameValue.trim()) {
      editingName = false;
      return;
    }
    await smartPlaylistStore.rename(smartPlaylistStore.selectedPlaylist.playlist.id, renameValue.trim());
    editingName = false;
  }

  async function saveRule() {
    if (!smartPlaylistStore.selectedPlaylist || !editingRule) return;
    await smartPlaylistStore.updateRule(smartPlaylistStore.selectedPlaylist.playlist.id, editingRule);
  }

  function playAll() {
    if (!smartPlaylistStore.selectedPlaylist) return;
    playerStore.playPlaylist(smartPlaylistStore.selectedPlaylist.tracks);
  }

  function playTrack(track: Track) {
    if (!smartPlaylistStore.selectedPlaylist) return;
    playerStore.playTrack(track, smartPlaylistStore.selectedPlaylist.tracks);
  }

  // ── Rule tree mutation helpers ───────────────────────────────────────
  // All mutations produce a new rule tree (immutable-style) and reassign editingRule

  function updateConditionField(path: number[], field: string) {
    if (!editingRule) return;
    editingRule = applyAtPath(editingRule, path, (_node) => {
      return { type: 'condition', field, op: getDefaultOp(field), value: getDefaultValue(field) } as Rule;
    });
  }

  function updateConditionOp(path: number[], op: string) {
    if (!editingRule) return;
    editingRule = applyAtPath(editingRule, path, (node) => {
      if (node.type === 'condition') return { ...node, op };
      return node;
    });
  }

  function updateConditionValue(path: number[], value: string) {
    if (!editingRule) return;
    editingRule = applyAtPath(editingRule, path, (node) => {
      if (node.type === 'condition') return { ...node, value };
      return node;
    });
  }

  function toggleGroupOperator(path: number[]) {
    if (!editingRule) return;
    editingRule = applyAtPath(editingRule, path, (node) => {
      if (node.type === 'group') {
        return { ...node, operator: node.operator === 'and' ? 'or' : 'and' } as Rule;
      }
      return node;
    });
  }

  function addConditionToGroup(path: number[]) {
    if (!editingRule) return;
    editingRule = applyAtPath(editingRule, path, (node) => {
      if (node.type === 'group') {
        return { ...node, rules: [...node.rules, makeCondition()] } as Rule;
      }
      return node;
    });
  }

  function addGroupToGroup(path: number[]) {
    if (!editingRule) return;
    editingRule = applyAtPath(editingRule, path, (node) => {
      if (node.type === 'group') {
        return { ...node, rules: [...node.rules, makeGroup()] } as Rule;
      }
      return node;
    });
  }

  function deleteAtPath(path: number[]) {
    if (!editingRule) return;
    if (path.length === 0) {
      // Deleting the root — replace with a fresh condition
      editingRule = makeCondition();
      return;
    }
    // path[-1] is the child index; path[0..-2] is the parent path
    const parentPath = path.slice(0, -1);
    const childIndex = path[path.length - 1];
    editingRule = applyAtPath(editingRule, parentPath, (node) => {
      if (node.type === 'group') {
        const rules = node.rules.filter((_, i) => i !== childIndex);
        return { ...node, rules: rules.length > 0 ? rules : [makeCondition()] } as Rule;
      }
      return node;
    });
  }

  // Walk the rule tree at the given path and apply a transform to the node at that path.
  function applyAtPath(root: Rule, path: number[], transform: (node: Rule) => Rule): Rule {
    if (path.length === 0) {
      return transform(root);
    }
    if (root.type !== 'group') return root;
    const [head, ...rest] = path;
    return {
      ...root,
      rules: root.rules.map((child, i) => i === head ? applyAtPath(child, rest, transform) : child),
    } as Rule;
  }
</script>

<div class="smart-playlists-page">
  <!-- Left panel: playlist list -->
  <div class="playlist-sidebar">
    <div class="sidebar-header">
      <h2>Smart Playlists</h2>
      <button class="primary small-btn" onclick={() => { showCreateForm = !showCreateForm; }}>
        + New
      </button>
    </div>

    {#if showCreateForm}
      <div class="create-form">
        <input
          type="text"
          placeholder="Playlist name..."
          bind:value={newName}
          onkeydown={(e) => { if (e.key === 'Enter') handleCreate(); if (e.key === 'Escape') { showCreateForm = false; newName = ''; } }}
        />
        <div class="create-form-actions">
          <button class="primary" onclick={handleCreate} disabled={!newName.trim()}>Create</button>
          <button class="secondary" onclick={() => { showCreateForm = false; newName = ''; }}>Cancel</button>
        </div>
      </div>
    {/if}

    <div class="playlist-list">
      {#each smartPlaylistStore.playlists as pl}
        <button
          class="playlist-item"
          class:active={smartPlaylistStore.selectedPlaylist?.playlist.id === pl.id}
          onclick={() => smartPlaylistStore.select(pl.id)}
        >
          <span class="pl-name">{pl.name}</span>
        </button>
      {/each}
      {#if smartPlaylistStore.playlists.length === 0 && !smartPlaylistStore.loading}
        <div class="empty-hint">No smart playlists yet</div>
      {/if}
      {#if smartPlaylistStore.loading}
        <div class="empty-hint">Loading...</div>
      {/if}
    </div>
  </div>

  <!-- Right panel: detail view -->
  <div class="playlist-detail">
    {#if smartPlaylistStore.selectedPlaylist && editingRule}
      {@const sp = smartPlaylistStore.selectedPlaylist}

      <div class="detail-header">
        {#if editingName}
          <input
            class="rename-input"
            type="text"
            bind:value={renameValue}
            onkeydown={(e) => { if (e.key === 'Enter') finishRename(); if (e.key === 'Escape') editingName = false; }}
            onblur={finishRename}
          />
        {:else}
          <h2 class="detail-title" ondblclick={startRename}>{sp.playlist.name}</h2>
        {/if}
        <div class="detail-actions">
          <button class="secondary" onclick={() => smartPlaylistStore.refresh()} disabled={smartPlaylistStore.evaluating}>
            {smartPlaylistStore.evaluating ? 'Evaluating...' : 'Refresh'}
          </button>
          <button class="secondary" onclick={playAll} disabled={sp.tracks.length === 0}>
            Play All
          </button>
          <button class="danger-btn" onclick={() => smartPlaylistStore.remove(sp.playlist.id)}>
            Delete
          </button>
        </div>
      </div>

      <!-- Rule builder -->
      <div class="rule-builder-section">
        <div class="rule-builder-header">
          <span class="section-label">Rules</span>
          <button class="secondary small-btn" onclick={saveRule}>Save Rules</button>
        </div>
        <div class="rule-builder">
          {@render ruleNode(editingRule, [], true)}
        </div>
      </div>

      <!-- Track list -->
      <div class="detail-info">
        {#if smartPlaylistStore.evaluating}
          Evaluating rules...
        {:else}
          {sp.tracks.length} {sp.tracks.length === 1 ? 'track' : 'tracks'} matched
        {/if}
      </div>
      <div class="track-list">
        {#each sp.tracks as track, i}
          <div
            class="playlist-track-row"
            class:now-playing={playerStore.currentTrack?.file_path === track.file_path}
            role="listitem"
          >
            <button class="track-play-btn" onclick={() => playTrack(track)} title="Play">&#9654;</button>
            <span class="track-pos">{i + 1}</span>
            <span class="track-title">{track.title ?? track.relative_path}</span>
            <span class="track-artist">{track.artist ?? ''}</span>
            <span class="track-duration">{formatDuration(track.duration_secs)}</span>
          </div>
        {/each}
        {#if sp.tracks.length === 0 && !smartPlaylistStore.evaluating}
          <div class="empty-hint">No tracks match these rules.</div>
        {/if}
      </div>
    {:else}
      <div class="no-selection">
        <p>Select a smart playlist or create a new one</p>
      </div>
    {/if}
  </div>
</div>

<!-- Recursive rule node snippet -->
{#snippet ruleNode(rule: Rule, path: number[], isRoot: boolean)}
  {#if rule.type === 'condition'}
    <div class="rule-condition">
      <select
        value={rule.field}
        onchange={(e) => updateConditionField(path, (e.target as HTMLSelectElement).value)}
      >
        {#each Object.entries(FIELD_LABELS) as [val, label]}
          <option value={val}>{label}</option>
        {/each}
      </select>
      <select
        value={rule.op}
        onchange={(e) => updateConditionOp(path, (e.target as HTMLSelectElement).value)}
      >
        {#each getOpsForField(rule.field) as op}
          <option value={op.value}>{op.label}</option>
        {/each}
      </select>
      {#if FIELD_TYPES[rule.field] === 'boolean'}
        <select
          value={rule.value}
          onchange={(e) => updateConditionValue(path, (e.target as HTMLSelectElement).value)}
        >
          <option value="true">Yes</option>
          <option value="false">No</option>
        </select>
      {:else if FIELD_TYPES[rule.field] === 'numeric'}
        <input
          type="number"
          class="value-input"
          value={rule.value}
          oninput={(e) => updateConditionValue(path, (e.target as HTMLInputElement).value)}
        />
      {:else}
        <input
          type="text"
          class="value-input"
          value={rule.value}
          oninput={(e) => updateConditionValue(path, (e.target as HTMLInputElement).value)}
        />
      {/if}
      {#if !isRoot}
        <button class="delete-node-btn" onclick={() => deleteAtPath(path)} title="Remove rule">x</button>
      {/if}
    </div>
  {:else}
    <div class="rule-group">
      <div class="group-header">
        <button
          class="operator-toggle"
          class:op-and={rule.operator === 'and'}
          class:op-or={rule.operator === 'or'}
          onclick={() => toggleGroupOperator(path)}
          title="Toggle AND/OR"
        >
          {rule.operator === 'and' ? 'AND' : 'OR'}
        </button>
        <button class="small-link-btn" onclick={() => addConditionToGroup(path)}>+ Condition</button>
        <button class="small-link-btn" onclick={() => addGroupToGroup(path)}>+ Group</button>
        {#if !isRoot}
          <button class="delete-node-btn" onclick={() => deleteAtPath(path)} title="Remove group">x</button>
        {/if}
      </div>
      <div class="group-children">
        {#each rule.rules as child, i}
          <div class="group-child">
            {@render ruleNode(child, [...path, i], false)}
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/snippet}

{#if smartPlaylistStore.error}
  <div class="error-bar">{smartPlaylistStore.error}</div>
{/if}

<style>
  .smart-playlists-page {
    display: flex;
    height: 100%;
    gap: 0;
  }

  .playlist-sidebar {
    width: 260px;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px 12px;
  }

  .sidebar-header h2 {
    font-size: 16px;
    font-weight: 600;
  }

  .small-btn {
    padding: 4px 10px;
    font-size: 12px;
  }

  .create-form {
    padding: 0 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .create-form input {
    padding: 6px 8px;
    font-size: 13px;
    width: 100%;
  }

  .create-form-actions {
    display: flex;
    gap: 6px;
  }

  .create-form-actions button {
    padding: 5px 10px;
    font-size: 12px;
  }

  .playlist-list {
    flex: 1;
    overflow-y: auto;
  }

  .playlist-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 10px 12px;
    text-align: left;
    background: none;
    border: none;
    border-radius: 0;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
  }

  .playlist-item:hover {
    background: var(--bg-tertiary);
  }

  .playlist-item.active {
    background: var(--bg-tertiary);
    font-weight: 600;
  }

  .pl-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .playlist-detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    padding: 0 16px;
    overflow-y: auto;
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 8px;
    flex-wrap: wrap;
    padding-top: 4px;
  }

  .detail-title {
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
  }

  .rename-input {
    font-size: 18px;
    font-weight: 600;
    padding: 4px 8px;
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .detail-actions {
    display: flex;
    gap: 6px;
    margin-left: auto;
  }

  .detail-actions button {
    padding: 6px 12px;
    font-size: 12px;
  }

  .danger-btn {
    background: var(--danger);
    color: var(--on-accent);
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .danger-btn:hover {
    opacity: 0.85;
  }

  /* Rule builder */
  .rule-builder-section {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
    margin-bottom: 12px;
    background: var(--bg-secondary);
  }

  .rule-builder-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  .section-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .rule-builder {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .rule-condition {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .rule-condition select,
  .rule-condition input.value-input {
    padding: 4px 6px;
    font-size: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .rule-condition input.value-input {
    width: 120px;
  }

  .rule-group {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px;
    background: var(--bg-primary);
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .operator-toggle {
    font-size: 11px;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: var(--radius);
    border: none;
    cursor: pointer;
    letter-spacing: 0.05em;
  }

  .operator-toggle.op-and {
    background: var(--accent-tint-strong);
    color: var(--accent);
    border: 1px solid var(--accent);
  }

  .operator-toggle.op-or {
    background: var(--info-tint);
    color: var(--info-color);
    border: 1px solid var(--info-color);
  }

  .small-link-btn {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 4px;
  }

  .small-link-btn:hover {
    color: var(--accent-hover);
  }

  .delete-node-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 6px;
    margin-left: auto;
    opacity: 0.6;
  }

  .delete-node-btn:hover {
    color: var(--danger);
    opacity: 1;
  }

  .group-children {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-left: 12px;
  }

  .group-child {
    display: flex;
    flex-direction: column;
  }

  /* Detail info */
  .detail-info {
    font-size: 13px;
    color: var(--text-secondary);
    padding-bottom: 8px;
  }

  /* Track list */
  .track-list {
    flex: 1;
    overflow-y: auto;
  }

  .playlist-track-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .playlist-track-row:hover {
    background: var(--bg-secondary);
  }

  .playlist-track-row.now-playing .track-title {
    color: var(--accent);
    font-weight: 600;
  }

  .track-play-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    padding: 2px 4px;
    opacity: 0;
    transition: opacity 0.15s;
    cursor: pointer;
  }

  .playlist-track-row:hover .track-play-btn {
    opacity: 1;
  }

  .track-play-btn:hover {
    color: var(--accent);
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
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track-duration {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }

  .empty-hint {
    padding: 16px 12px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .error-bar {
    position: fixed;
    bottom: 60px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--danger);
    color: var(--on-accent);
    padding: 8px 16px;
    border-radius: var(--radius);
    font-size: 13px;
    z-index: 200;
  }
</style>
