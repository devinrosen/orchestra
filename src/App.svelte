<script lang="ts">
  import { onMount } from "svelte";
  import "./app.css";
  import Library from "./pages/Library.svelte";
  import SyncProfiles from "./pages/SyncProfiles.svelte";
  import SyncPreview from "./pages/SyncPreview.svelte";
  import DeviceSync from "./pages/DeviceSync.svelte";
  import Settings from "./pages/Settings.svelte";
  import Statistics from "./pages/Statistics.svelte";
  import Playlists from "./pages/Playlists.svelte";
  import SmartPlaylists from "./pages/SmartPlaylists.svelte";
  import Favorites from "./pages/Favorites.svelte";
  import GlobalStatusBar from "./lib/components/GlobalStatusBar.svelte";
  import NowPlayingBar from "./lib/components/NowPlayingBar.svelte";
  import VisualizerPanel from "./lib/components/VisualizerPanel.svelte";
  import EqualizerPanel from "./lib/components/EqualizerPanel.svelte";
  import { libraryStore } from "./lib/stores/library.svelte";
  import { playerStore } from "./lib/stores/player.svelte";
  import { playlistStore } from "./lib/stores/playlist.svelte";
  import { smartPlaylistStore } from "./lib/stores/smartPlaylist.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";
  import { favoritesStore } from "./lib/stores/favorites.svelte";

  type Page = "library" | "favorites" | "statistics" | "playlists" | "smart-playlists" | "profiles" | "sync-preview" | "devices" | "settings";

  let currentPage = $state<Page>("library");
  let pageData = $state<Record<string, unknown>>({});

  onMount(() => {
    libraryStore.init();
    playlistStore.load();
    smartPlaylistStore.load();
    themeStore.init();
    favoritesStore.load();
  });

  $effect(() => {
    document.documentElement.setAttribute("data-theme", themeStore.resolved);
  });

  function navigate(page: string, data?: Record<string, unknown>) {
    currentPage = page as Page;
    pageData = data ?? {};
  }

  const navItems: { page: Page; label: string }[] = [
    { page: "library", label: "Library" },
    { page: "favorites", label: "Favorites" },
    { page: "statistics", label: "Statistics" },
    { page: "playlists", label: "Playlists" },
    { page: "smart-playlists", label: "Smart Playlists" },
    { page: "profiles", label: "Sync Profiles" },
    { page: "devices", label: "Devices" },
    { page: "settings", label: "Settings" },
  ];
</script>

<div class="app-layout">
  <nav class="sidebar">
    <div class="app-title">Orchestra</div>
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={currentPage === item.page}
        onclick={() => navigate(item.page)}
      >
        {item.label}
      </button>
    {/each}
  </nav>

  <div class="main-column">
    <GlobalStatusBar onNavigate={navigate} />
    <main class="content">
    {#if currentPage === "library"}
      <Library />
    {:else if currentPage === "favorites"}
      <Favorites />
    {:else if currentPage === "statistics"}
      <Statistics />
    {:else if currentPage === "playlists"}
      <Playlists />
    {:else if currentPage === "smart-playlists"}
      <SmartPlaylists />
    {:else if currentPage === "profiles"}
      <SyncProfiles onNavigate={navigate} />
    {:else if currentPage === "sync-preview"}
      <SyncPreview
        profileId={pageData.profileId as string}
        onNavigate={navigate}
      />
    {:else if currentPage === "devices"}
      <DeviceSync />
    {:else if currentPage === "settings"}
      <Settings />
    {/if}
    </main>
    {#if playerStore.currentTrack && playerStore.visualizerActive}
      <VisualizerPanel onClose={() => playerStore.toggleVisualizer()} />
    {/if}
    {#if playerStore.currentTrack && playerStore.equalizerActive}
      <EqualizerPanel onClose={() => playerStore.toggleEqualizer()} />
    {/if}
    {#if playerStore.currentTrack}
      <NowPlayingBar />
    {/if}
  </div>
</div>

<style>
  .app-layout {
    display: flex;
    height: 100vh;
    width: 100vw;
  }

  .sidebar {
    width: 200px;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 16px 8px;
    gap: 4px;
    flex-shrink: 0;
  }

  .app-title {
    font-size: 18px;
    font-weight: 700;
    padding: 8px 12px 16px;
    color: var(--accent);
  }

  .nav-item {
    background: none;
    border: none;
    color: var(--text-secondary);
    padding: 10px 12px;
    border-radius: var(--radius);
    text-align: left;
    font-size: 14px;
    transition: all 0.15s;
  }

  .nav-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .nav-item.active {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    font-weight: 500;
  }

  .main-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .content {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
  }
</style>
