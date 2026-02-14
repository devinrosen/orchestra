<script lang="ts">
  import type { Track, TrackMetadataUpdate, AlbumArt } from "../api/types";
  import { getTrackArtwork, updateTrackMetadata } from "../api/commands";

  let {
    track,
    onSave,
    onClose,
  }: {
    track: Track;
    onSave: (tracks: Track[]) => void;
    onClose: () => void;
  } = $props();

  // svelte-ignore state_referenced_locally — intentional one-time copy for editing
  let title = $state(track.title ?? "");
  // svelte-ignore state_referenced_locally
  let artist = $state(track.artist ?? "");
  // svelte-ignore state_referenced_locally
  let albumArtist = $state(track.album_artist ?? "");
  // svelte-ignore state_referenced_locally
  let album = $state(track.album ?? "");
  // svelte-ignore state_referenced_locally
  let trackNumber = $state(track.track_number?.toString() ?? "");
  // svelte-ignore state_referenced_locally
  let discNumber = $state(track.disc_number?.toString() ?? "");
  // svelte-ignore state_referenced_locally
  let year = $state(track.year?.toString() ?? "");
  // svelte-ignore state_referenced_locally
  let genre = $state(track.genre ?? "");

  let artwork = $state<AlbumArt | null>(null);
  let saving = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    getTrackArtwork(track.file_path).then((art) => {
      artwork = art;
    }).catch(() => {});
  });

  async function handleSave() {
    saving = true;
    error = null;
    try {
      const update: TrackMetadataUpdate = {
        file_path: track.file_path,
        title: title || null,
        artist: artist || null,
        album_artist: albumArtist || null,
        album: album || null,
        track_number: trackNumber ? parseInt(trackNumber) : null,
        disc_number: discNumber ? parseInt(discNumber) : null,
        year: year ? parseInt(year) : null,
        genre: genre || null,
      };
      const updated = await updateTrackMetadata([update]);
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
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="editor-dialog" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="editor-header">
      <h3>Edit Track</h3>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    <div class="file-path">{track.file_path}</div>

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
          <span>Title</span>
          <input type="text" bind:value={title} />
        </label>
        <label class="field">
          <span>Artist</span>
          <input type="text" bind:value={artist} />
        </label>
        <label class="field">
          <span>Album Artist</span>
          <input type="text" bind:value={albumArtist} />
        </label>
        <label class="field">
          <span>Album</span>
          <input type="text" bind:value={album} />
        </label>
        <div class="field-row">
          <label class="field small">
            <span>Track #</span>
            <input type="number" min="0" bind:value={trackNumber} />
          </label>
          <label class="field small">
            <span>Disc #</span>
            <input type="number" min="0" bind:value={discNumber} />
          </label>
          <label class="field small">
            <span>Year</span>
            <input type="number" min="0" bind:value={year} />
          </label>
        </div>
        <label class="field">
          <span>Genre</span>
          <input type="text" bind:value={genre} />
        </label>
      </div>
    </div>

    <div class="dialog-actions">
      <button class="secondary" onclick={onClose}>Cancel</button>
      <button class="primary" onclick={handleSave} disabled={saving}>
        {saving ? "Saving..." : "Save"}
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

  .file-path {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    padding: 4px 8px;
    border-radius: var(--radius);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
</style>
