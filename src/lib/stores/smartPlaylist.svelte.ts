import type { SmartPlaylist, SmartPlaylistWithTracks, Rule } from "../api/types";
import * as commands from "../api/commands";

class SmartPlaylistStore {
  playlists = $state<SmartPlaylist[]>([]);
  selectedPlaylist = $state<SmartPlaylistWithTracks | null>(null);
  loading = $state(false);
  evaluating = $state(false);
  error = $state<string | null>(null);

  async load() {
    this.loading = true;
    this.error = null;
    try {
      this.playlists = await commands.listSmartPlaylists();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async create(name: string, rule: Rule) {
    this.error = null;
    try {
      const result = await commands.createSmartPlaylist({ name, rule });
      this.playlists = [result.playlist, ...this.playlists];
      this.selectedPlaylist = result;
      return result;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async select(id: string) {
    this.evaluating = true;
    this.error = null;
    try {
      this.selectedPlaylist = await commands.evaluateSmartPlaylist(id);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.evaluating = false;
    }
  }

  async rename(id: string, name: string) {
    this.error = null;
    try {
      const updated = await commands.updateSmartPlaylist({ id, name });
      this.playlists = this.playlists.map((p) => (p.id === updated.id ? updated : p));
      if (this.selectedPlaylist?.playlist.id === updated.id) {
        this.selectedPlaylist = { ...this.selectedPlaylist, playlist: updated };
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  async updateRule(id: string, rule: Rule) {
    this.error = null;
    try {
      const updated = await commands.updateSmartPlaylist({ id, rule });
      this.playlists = this.playlists.map((p) => (p.id === updated.id ? updated : p));
      // Re-evaluate to update tracks
      this.evaluating = true;
      try {
        this.selectedPlaylist = await commands.evaluateSmartPlaylist(id);
      } finally {
        this.evaluating = false;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  async remove(id: string) {
    this.error = null;
    try {
      await commands.deleteSmartPlaylist(id);
      this.playlists = this.playlists.filter((p) => p.id !== id);
      if (this.selectedPlaylist?.playlist.id === id) {
        this.selectedPlaylist = null;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  async refresh() {
    if (!this.selectedPlaylist) return;
    const id = this.selectedPlaylist.playlist.id;
    this.evaluating = true;
    this.error = null;
    try {
      this.selectedPlaylist = await commands.evaluateSmartPlaylist(id);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.evaluating = false;
    }
  }
}

export const smartPlaylistStore = new SmartPlaylistStore();
