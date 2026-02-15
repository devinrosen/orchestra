import type { LibraryTree, LibraryRoot, Track, ProgressEvent, LibraryViewMode, ArtistNode, AlbumEntry, GenreNode, FolderNode } from "../api/types";
import * as commands from "../api/commands";
import { flattenTree, groupByAlbum, groupByGenre, groupByFolder, filterArtists, filterAlbums, filterGenres, filterFolders } from "../utils/library-grouping";
import { favoritesStore } from "./favorites.svelte";

function filterFolderByFavorites(node: FolderNode): FolderNode | null {
  const filteredTracks = node.tracks.filter(
    (t) => t.id != null && favoritesStore.isFavorite('track', String(t.id))
  );
  const filteredChildren = node.children
    .map((c) => filterFolderByFavorites(c))
    .filter((c): c is FolderNode => c !== null);

  if (filteredTracks.length === 0 && filteredChildren.length === 0) return null;
  return { ...node, tracks: filteredTracks, children: filteredChildren };
}

class LibraryStore {
  tree = $state<LibraryTree | null>(null);
  libraryRoots = $state<LibraryRoot[]>([]);
  activeRoot = $state<string | null>(null);
  scanning = $state(false);
  scanStartedAt = $state<number | null>(null);
  scanProgress = $state({ filesFound: 0, filesProcessed: 0, currentFile: "", dirsTotal: 0, dirsCompleted: 0 });
  searchQuery = $state("");
  error = $state<string | null>(null);
  viewMode = $state<LibraryViewMode>("artist");
  incompleteCount = $state<number>(0);

  allTracks = $derived<Track[]>(this.tree ? flattenTree(this.tree.artists) : []);

  albumEntries = $derived<AlbumEntry[]>(
    this.viewMode === "album" ? groupByAlbum(this.allTracks) : []
  );

  genreNodes = $derived<GenreNode[]>(
    this.viewMode === "genre" ? groupByGenre(this.allTracks) : []
  );

  folderTree = $derived<FolderNode | null>(
    this.viewMode === "folder" ? groupByFolder(this.allTracks) : null
  );

  filteredArtists = $derived<ArtistNode[]>(
    this.viewMode === "artist"
      ? (this.searchQuery.length >= 2 && this.tree
          ? filterArtists(this.tree.artists, this.searchQuery)
          : this.tree?.artists ?? [])
      : []
  );

  filteredAlbumEntries = $derived<AlbumEntry[]>(
    this.viewMode === "album"
      ? (this.searchQuery.length >= 2
          ? filterAlbums(this.albumEntries, this.searchQuery)
          : this.albumEntries)
      : []
  );

  filteredGenreNodes = $derived<GenreNode[]>(
    this.viewMode === "genre"
      ? (this.searchQuery.length >= 2
          ? filterGenres(this.genreNodes, this.searchQuery)
          : this.genreNodes)
      : []
  );

  filteredFolderTree = $derived<FolderNode | null>(
    this.viewMode === "folder"
      ? (this.searchQuery.length >= 2 && this.folderTree
          ? filterFolders(this.folderTree, this.searchQuery)
          : this.folderTree)
      : null
  );

  favoritesOnly = $state(false);

  displayArtists = $derived<ArtistNode[]>(
    this.favoritesOnly
      ? this.filteredArtists.filter((a) => favoritesStore.isFavorite('artist', a.name))
      : this.filteredArtists
  );

  displayAlbumEntries = $derived<AlbumEntry[]>(
    this.favoritesOnly
      ? this.filteredAlbumEntries.filter((a) => favoritesStore.isFavorite('album', a.artist + "\0" + a.name))
      : this.filteredAlbumEntries
  );

  displayGenreNodes = $derived<GenreNode[]>(
    this.favoritesOnly
      ? this.filteredGenreNodes
          .map((g) => ({
            ...g,
            albums: g.albums.filter((a) => favoritesStore.isFavorite('album', a.artist + "\0" + a.name)),
          }))
          .filter((g) => g.albums.length > 0)
      : this.filteredGenreNodes
  );

  displayFolderTree = $derived<FolderNode | null>(
    this.favoritesOnly && this.filteredFolderTree
      ? filterFolderByFavorites(this.filteredFolderTree)
      : this.filteredFolderTree
  );

  async setViewMode(mode: LibraryViewMode) {
    this.viewMode = mode;
    try {
      await commands.setSetting("library_view_mode", mode);
    } catch (_) {
      // non-critical — ignore persistence failures
    }
  }

  async scan(path: string) {
    this.scanning = true;
    this.scanStartedAt = Date.now();
    this.error = null;
    this.scanProgress = { filesFound: 0, filesProcessed: 0, currentFile: "", dirsTotal: 0, dirsCompleted: 0 };

    try {
      // Ensure path is registered in library_roots table
      if (!this.libraryRoots.some(r => r.path === path)) {
        await commands.addLibraryRoot(path);
        this.libraryRoots = await commands.listLibraryRoots();
      }

      await commands.scanDirectory(path, (event: ProgressEvent) => {
        if (event.type === "scan_progress") {
          this.scanProgress = {
            filesFound: event.files_found,
            filesProcessed: event.files_processed,
            currentFile: event.current_file,
            dirsTotal: event.dirs_total,
            dirsCompleted: event.dirs_completed,
          };
        } else if (event.type === "scan_tree_updated") {
          // Phase 1 done — reload tree immediately so new/removed albums appear
          this.loadTree(path);
        }
      });
      await this.loadTree(path);
      // Write both library_root (backward compat for device_cmd fallback) and active_library_root
      await commands.setSetting("library_root", path);
      await commands.setSetting("active_library_root", path);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.scanning = false;
      this.scanStartedAt = null;
    }
  }

  async loadTree(root: string) {
    try {
      this.activeRoot = root;
      this.tree = await commands.getLibraryTree(root);
      await this.loadIncompleteCount(root);
    } catch (e) {
      this.error = String(e);
    }
  }

  async loadIncompleteCount(root: string) {
    try {
      const tracks = await commands.getIncompleteTracks(root);
      this.incompleteCount = tracks.length;
    } catch (_) {
      // non-critical
    }
  }

  async addRoot(path: string, label?: string) {
    await commands.addLibraryRoot(path, label);
    this.libraryRoots = await commands.listLibraryRoots();
    if (!this.activeRoot) await this.loadTree(path);
  }

  async removeRoot(path: string) {
    await commands.removeLibraryRoot(path);
    this.libraryRoots = await commands.listLibraryRoots();
    if (this.activeRoot === path) {
      const next = this.libraryRoots[0]?.path ?? null;
      if (next) await this.loadTree(next);
      else { this.tree = null; this.activeRoot = null; }
    }
  }

  async switchRoot(path: string) {
    await this.loadTree(path);
    await commands.setSetting("active_library_root", path);
  }

  async init() {
    try {
      // Load all configured roots
      this.libraryRoots = await commands.listLibraryRoots();

      const savedMode = await commands.getSetting("library_view_mode");
      if (savedMode && ["artist", "album", "genre", "folder"].includes(savedMode)) {
        this.viewMode = savedMode as LibraryViewMode;
      }

      // Determine which root to show: prefer last active, fall back to first root,
      // then fall back to legacy library_root setting for backward compat
      const savedActive = await commands.getSetting("active_library_root");
      let target: string | null = null;
      if (savedActive && this.libraryRoots.some(r => r.path === savedActive)) {
        target = savedActive;
      } else if (this.libraryRoots.length > 0) {
        target = this.libraryRoots[0].path;
      } else {
        // Backward compat: no library_roots rows yet, fall back to legacy setting
        const legacyRoot = await commands.getSetting("library_root");
        if (legacyRoot) target = legacyRoot;
      }

      if (target) {
        await this.loadTree(target);
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  search(query: string) {
    this.searchQuery = query;
  }
}

export const libraryStore = new LibraryStore();
