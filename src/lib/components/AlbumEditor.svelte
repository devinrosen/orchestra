<script lang="ts">
  import type { Track, TrackMetadataUpdate, AlbumArt } from "../api/types";
  import { getTrackArtwork, updateTrackMetadata } from "../api/commands";
  import { formatDuration } from "../utils/format";

  let {
    tracks,
    albumName,
    artistName,
    onSave,
    onClose,
  }: {
    tracks: Track[];
    albumName: string;
    artistName: string;
    onSave: (tracks: Track[]) => void;
    onClose: () => void;
  } = $props();

  let album = $state(albumName);
  let artist = $state(tracks[0]?.artist ?? "");
  let albumArtist = $state(artistName);
  let year = $state(tracks[0]?.year?.toString() ?? "");
  let genre = $state(tracks[0]?.genre ?? "");

  let artwork = $state<AlbumArt | null>(null);
  let saving = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (tracks.length > 0) {
      getTrackArtwork(tracks[0].file_path).then((art) => {
        artwork = art;
      }).catch(() => {});
    }
  });

  async function handleSave() {
    saving = true;
    error = null;
    try {
      const updates: TrackMetadataUpdate[] = tracks.map((t) => ({
        file_path: t.file_path,
        album: album || null,
        artist: artist || null,
        album_artist: albumArtist || null,
        year: year ? parseInt(year) : null,
        genre: genre || null,
      }));
      const updated = await updateTrackMetadata(updates);
      onSave(updated);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="editor-overlay" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="editor-dialog" role="dialog" onclick={(e) => e.stopPropagation()}>
    <div class="editor-header">
      <h3>Edit Album ({tracks.length} tracks)</h3>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    {#if error}
      <div class="error-banner">{error}</div>
    {/if}

    <div class="editor-body">
      <div class="artwork-column">
        {#if artwork}
          <img
            src="data:{artwork.mime_type};base64,{artwork.data}"
            alt="Album art"
            class="artwork-img"
          />
        {:else}
          <div class="artwork-placeholder">No Artwork</div>
        {/if}
      </div>

      <div class="fields-column">
        <label class="field">
          <span>Album</span>
          <input type="text" bind:value={album} />
        </label>
        <label class="field">
          <span>Artist</span>
          <input type="text" bind:value={artist} />
        </label>
        <label class="field">
          <span>Album Artist</span>
          <input type="text" bind:value={albumArtist} />
        </label>
        <div class="field-row">
          <label class="field small">
            <span>Year</span>
            <input type="number" min="0" bind:value={year} />
          </label>
          <label class="field small">
            <span>Genre</span>
            <input type="text" bind:value={genre} />
          </label>
        </div>
      </div>
    </div>

    <div class="track-list">
      <h4>Tracks</h4>
      {#each tracks as t}
        <div class="track-row">
          <span class="track-num">{t.track_number ?? "-"}</span>
          <span class="track-title">{t.title ?? t.relative_path}</span>
          <span class="track-duration">{formatDuration(t.duration_secs)}</span>
        </div>
      {/each}
    </div>

    <div class="dialog-actions">
      <button class="secondary" onclick={onClose}>Cancel</button>
      <button class="primary" onclick={handleSave} disabled={saving}>
        {saving ? "Saving..." : "Save All"}
      </button>
    </div>
  </div>
</div>

<style>
  .editor-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .editor-dialog {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    width: 600px;
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .editor-header h3 {
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

  .error-banner {
    background: rgba(233, 69, 96, 0.15);
    color: var(--danger);
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .editor-body {
    display: flex;
    gap: 20px;
  }

  .artwork-column {
    flex-shrink: 0;
    width: 160px;
  }

  .artwork-img {
    width: 160px;
    height: 160px;
    object-fit: cover;
    border-radius: var(--radius);
    border: 1px solid var(--border);
  }

  .artwork-placeholder {
    width: 160px;
    height: 160px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .fields-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .field input {
    width: 100%;
  }

  .field-row {
    display: flex;
    gap: 10px;
  }

  .field.small {
    flex: 1;
  }

  .field.small input {
    width: 100%;
  }

  .track-list {
    border-top: 1px solid var(--border);
    padding-top: 12px;
  }

  .track-list h4 {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .track-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 3px 8px;
    font-size: 13px;
    border-radius: var(--radius);
  }

  .track-row:hover {
    background: var(--bg-secondary);
  }

  .track-num {
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

  .track-duration {
    color: var(--text-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
</style>
