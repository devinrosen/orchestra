import type { LibraryTree, Track, ProgressEvent, LibraryViewMode, ArtistNode, AlbumEntry, GenreNode, FolderNode } from "../api/types";
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
  libraryRoot = $state<string>("");
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
    this.libraryRoot = path;
    this.scanProgress = { filesFound: 0, filesProcessed: 0, currentFile: "", dirsTotal: 0, dirsCompleted: 0 };

    try {
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
      await commands.setSetting("library_root", path);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.scanning = false;
      this.scanStartedAt = null;
    }
  }

  async loadTree(root: string) {
    try {
      this.libraryRoot = root;
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

  async init() {
    try {
      const root = await commands.getSetting("library_root");
      const savedMode = await commands.getSetting("library_view_mode");
      if (savedMode && ["artist", "album", "genre", "folder"].includes(savedMode)) {
        this.viewMode = savedMode as LibraryViewMode;
      }
      if (root) {
        await this.loadTree(root);
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
