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
  import Favorites from "./pages/Favorites.svelte";
  import GlobalStatusBar from "./lib/components/GlobalStatusBar.svelte";
  import NowPlayingBar from "./lib/components/NowPlayingBar.svelte";
  import VisualizerPanel from "./lib/components/VisualizerPanel.svelte";
  import EqualizerPanel from "./lib/components/EqualizerPanel.svelte";
  import ShortcutsHelpOverlay from "./lib/components/ShortcutsHelpOverlay.svelte";
  import { libraryStore } from "./lib/stores/library.svelte";
  import { playerStore } from "./lib/stores/player.svelte";
  import { playlistStore } from "./lib/stores/playlist.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";
  import { favoritesStore } from "./lib/stores/favorites.svelte";
  import { shortcutsStore } from "./lib/stores/shortcuts.svelte";

  type Page = "library" | "favorites" | "statistics" | "playlists" | "profiles" | "sync-preview" | "devices" | "settings";

  let currentPage = $state<Page>("library");
  let pageData = $state<Record<string, unknown>>({});
  let showShortcutsOverlay = $state(false);

  onMount(() => {
    libraryStore.init();
    playlistStore.load();
    themeStore.init();
    favoritesStore.load();
    shortcutsStore.load();
  });

  $effect(() => {
    document.documentElement.setAttribute("data-theme", themeStore.resolved);
  });

  function navigate(page: string, data?: Record<string, unknown>) {
    currentPage = page as Page;
    pageData = data ?? {};
  }

  function handleGlobalKey(e: KeyboardEvent) {
    // Do not intercept shortcuts while typing in an input field
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement ||
      e.target instanceof HTMLSelectElement
    ) {
      return;
    }

    const action = shortcutsStore.match(e);
    if (!action) return;

    switch (action) {
      case "play-pause":
        if (playerStore.currentTrack) {
          e.preventDefault();
          playerStore.togglePlayPause();
        }
        break;
      case "next-track":
        if (playerStore.hasNext) {
          e.preventDefault();
          playerStore.next();
        }
        break;
      case "prev-track":
        if (playerStore.hasPrev) {
          e.preventDefault();
          playerStore.previous();
        }
        break;
      case "volume-up":
        e.preventDefault();
        playerStore.setVolume(Math.min(1, playerStore.volume + 0.05));
        break;
      case "volume-down":
        e.preventDefault();
        playerStore.setVolume(Math.max(0, playerStore.volume - 0.05));
        break;
      case "nav-library":
        e.preventDefault();
        navigate("library");
        break;
      case "nav-favorites":
        e.preventDefault();
        navigate("favorites");
        break;
      case "nav-playlists":
        e.preventDefault();
        navigate("playlists");
        break;
      case "nav-profiles":
        e.preventDefault();
        navigate("profiles");
        break;
      case "nav-devices":
        e.preventDefault();
        navigate("devices");
        break;
      case "nav-settings":
        e.preventDefault();
        navigate("settings");
        break;
      case "rescan-library":
        if (libraryStore.libraryRoot && !libraryStore.scanning) {
          e.preventDefault();
          libraryStore.scan(libraryStore.libraryRoot);
        }
        break;
      case "focus-search":
        e.preventDefault();
        window.dispatchEvent(new CustomEvent("focus-search"));
        break;
      case "show-shortcuts":
        e.preventDefault();
        showShortcutsOverlay = !showShortcutsOverlay;
        break;
    }
  }

  const navItems: { page: Page; label: string; title: string }[] = [
    { page: "library",    label: "Library",       title: "Library (1)" },
    { page: "favorites",  label: "Favorites",     title: "Favorites (2)" },
    { page: "statistics", label: "Statistics",    title: "Statistics" },
    { page: "playlists",  label: "Playlists",     title: "Playlists (3)" },
    { page: "profiles",   label: "Sync Profiles", title: "Sync Profiles (4)" },
    { page: "devices",    label: "Devices",       title: "Devices (5)" },
    { page: "settings",   label: "Settings",      title: "Settings (6)" },
  ];
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="app-layout" onkeydown={handleGlobalKey}>
  <nav class="sidebar">
    <div class="app-title">Orchestra</div>
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={currentPage === item.page}
        onclick={() => navigate(item.page)}
        title={item.title}
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

{#if showShortcutsOverlay}
  <ShortcutsHelpOverlay onClose={() => (showShortcutsOverlay = false)} />
{/if}

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
