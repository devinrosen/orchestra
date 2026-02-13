import type { LibraryTree, Track, ProgressEvent } from "../api/types";
import * as commands from "../api/commands";

class LibraryStore {
  tree = $state<LibraryTree | null>(null);
  libraryRoot = $state<string>("");
  scanning = $state(false);
  scanProgress = $state({ filesFound: 0, filesProcessed: 0, currentFile: "" });
  searchResults = $state<Track[]>([]);
  searchQuery = $state("");
  error = $state<string | null>(null);

  async scan(path: string) {
    this.scanning = true;
    this.error = null;
    this.libraryRoot = path;
    this.scanProgress = { filesFound: 0, filesProcessed: 0, currentFile: "" };

    try {
      await commands.scanDirectory(path, (event: ProgressEvent) => {
        if (event.type === "scan_progress") {
          this.scanProgress = {
            filesFound: event.files_found,
            filesProcessed: event.files_processed,
            currentFile: event.current_file,
          };
        }
      });
      await this.loadTree(path);
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
