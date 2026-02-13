import type { LibraryTree, Track, ProgressEvent, LibraryViewMode, AlbumEntry, GenreNode, FolderNode } from "../api/types";
import * as commands from "../api/commands";
import { flattenTree, groupByAlbum, groupByGenre, groupByFolder } from "../utils/library-grouping";

class LibraryStore {
  tree = $state<LibraryTree | null>(null);
  libraryRoot = $state<string>("");
  scanning = $state(false);
  scanProgress = $state({ filesFound: 0, filesProcessed: 0, currentFile: "", dirsTotal: 0, dirsCompleted: 0 });
  searchResults = $state<Track[]>([]);
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

  async search(query: string) {
    this.searchQuery = query;
    if (query.trim().length < 2) {
      this.searchResults = [];
      return;
    }
    try {
      this.searchResults = await commands.searchLibrary(query);
    } catch (e) {
      this.error = String(e);
    }
  }
}

export const libraryStore = new LibraryStore();
