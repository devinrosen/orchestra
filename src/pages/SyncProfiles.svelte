<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import ProfileCard from "../lib/components/ProfileCard.svelte";
  import { profilesStore } from "../lib/stores/profiles.svelte";
  import type { SyncMode, CreateProfileRequest } from "../lib/api/types";

  let { onNavigate }: { onNavigate: (page: string, data?: Record<string, unknown>) => void } = $props();

  let showForm = $state(false);
  let formName = $state("");
  let formSource = $state("");
  let formTarget = $state("");
  let formMode = $state<SyncMode>("one_way");
  let formExclude = $state("");
  let editingId = $state<string | null>(null);

  $effect(() => {
    profilesStore.load();
  });

  async function pickPath(which: "source" | "target") {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      if (which === "source") formSource = selected;
      else formTarget = selected;
    }
  }

  async function submitForm() {
    const excludePatterns = formExclude
      .split("\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    if (editingId) {
      await profilesStore.update({
        id: editingId,
        name: formName,
        source_path: formSource,
        target_path: formTarget,
        sync_mode: formMode,
        exclude_patterns: excludePatterns,
      });
    } else {
      await profilesStore.create({
        name: formName,
        source_path: formSource,
        target_path: formTarget,
        sync_mode: formMode,
        exclude_patterns: excludePatterns,
      });
    }
    resetForm();
  }

  function resetForm() {
    showForm = false;
    editingId = null;
    formName = "";
    formSource = "";
    formTarget = "";
    formMode = "one_way";
    formExclude = "";
  }

  function editProfile(profile: typeof profilesStore.profiles[0]) {
    editingId = profile.id;
    formName = profile.name;
    formSource = profile.source_path;
    formTarget = profile.target_path;
    formMode = profile.sync_mode;
    formExclude = profile.exclude_patterns.join("\n");
    showForm = true;
  }

  async function handleDelete(id: string) {
    await profilesStore.remove(id);
  }

  function handleSync(profileId: string) {
    onNavigate("sync-preview", { profileId });
  }
</script>

<div class="profiles-page">
  <div class="page-header">
    <h2>Sync Profiles</h2>
    <button class="primary" onclick={() => (showForm = !showForm)}>
      {showForm ? "Cancel" : "New Profile"}
    </button>
  </div>

  {#if profilesStore.error}
    <div class="error-banner">{profilesStore.error}</div>
  {/if}

  {#if showForm}
    <div class="profile-form">
      <h3>{editingId ? "Edit Profile" : "Create Profile"}</h3>
      <div class="form-field">
        <label for="profile-name">Name</label>
        <input id="profile-name" type="text" bind:value={formName} placeholder="My Sync Profile" />
      </div>
      <div class="form-field">
        <label for="profile-source">Source</label>
        <div class="path-picker">
          <input id="profile-source" type="text" bind:value={formSource} placeholder="/path/to/source" />
          <button class="secondary" onclick={() => pickPath("source")}>Browse</button>
        </div>
      </div>
      <div class="form-field">
        <label for="profile-target">Target</label>
        <div class="path-picker">
          <input id="profile-target" type="text" bind:value={formTarget} placeholder="/path/to/target" />
          <button class="secondary" onclick={() => pickPath("target")}>Browse</button>
        </div>
      </div>
      <div class="form-field">
        <label for="sync-mode">Sync Mode</label>
        <select id="sync-mode" bind:value={formMode}>
          <option value="one_way">One-Way (Source → Target)</option>
          <option value="two_way">Two-Way (Bidirectional)</option>
        </select>
      </div>
      <div class="form-field">
        <label for="exclude">Exclude Patterns (one per line)</label>
        <textarea id="exclude" bind:value={formExclude} rows="3" placeholder="*.tmp&#10;.DS_Store"></textarea>
      </div>
      <button
        class="primary"
        onclick={submitForm}
        disabled={!formName || !formSource || !formTarget}
      >
        {editingId ? "Update" : "Create"}
      </button>
    </div>
  {/if}

  <div class="profiles-list">
    {#each profilesStore.profiles as profile}
      <ProfileCard
        {profile}
        onSelect={() => editProfile(profile)}
        onDelete={() => handleDelete(profile.id)}
        onSync={() => handleSync(profile.id)}
      />
    {/each}
    {#if profilesStore.profiles.length === 0 && !profilesStore.loading}
      <div class="empty-state">
        <p>No sync profiles yet. Create one to get started.</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .profiles-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 16px;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .page-header h2 {
    font-size: 20px;
    font-weight: 600;
  }

  .error-banner {
    background: var(--accent-tint-strong);
    color: var(--danger);
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13px;
  }

  .profile-form {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .profile-form h3 {
    font-size: 16px;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-field label {
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .path-picker {
    display: flex;
    gap: 8px;
  }

  .path-picker input {
    flex: 1;
  }

  textarea {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px;
    font-family: monospace;
    font-size: 13px;
    resize: vertical;
  }

  .profiles-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
    flex: 1;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--text-secondary);
  }
</style>
