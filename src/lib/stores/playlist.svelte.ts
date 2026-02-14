import type { Playlist, PlaylistWithTracks } from "../api/types";
import * as commands from "../api/commands";

class PlaylistStore {
  playlists = $state<Playlist[]>([]);
  selectedPlaylist = $state<PlaylistWithTracks | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async load() {
    this.loading = true;
    this.error = null;
    try {
      this.playlists = await commands.listPlaylists();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async create(name: string) {
    this.error = null;
    try {
      const result = await commands.createPlaylist({ name });
      this.playlists = [result.playlist, ...this.playlists];
      return result;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async select(id: string) {
    this.error = null;
    try {
      this.selectedPlaylist = await commands.getPlaylist(id);
    } catch (e) {
      this.error = String(e);
    }
  }

  async rename(id: string, name: string) {
    this.error = null;
    try {
      const updated = await commands.updatePlaylist({ id, name });
      this.playlists = this.playlists.map((p) => (p.id === updated.id ? updated : p));
      if (this.selectedPlaylist?.playlist.id === updated.id) {
        this.selectedPlaylist = { ...this.selectedPlaylist, playlist: updated };
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  async remove(id: string) {
    this.error = null;
    try {
      await commands.deletePlaylist(id);
      this.playlists = this.playlists.filter((p) => p.id !== id);
      if (this.selectedPlaylist?.playlist.id === id) {
        this.selectedPlaylist = null;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  async addTracks(playlistId: string, trackIds: number[]) {
    this.error = null;
    try {
      const result = await commands.addTracksToPlaylist({
        playlist_id: playlistId,
        track_ids: trackIds,
      });
      if (this.selectedPlaylist?.playlist.id === playlistId) {
        this.selectedPlaylist = result;
      }
      this.playlists = this.playlists.map((p) =>
        p.id === result.playlist.id ? result.playlist : p,
      );
    } catch (e) {
      this.error = String(e);
    }
  }

  async removeTracks(playlistId: string, trackIds: number[]) {
    this.error = null;
    try {
      const result = await commands.removeTracksFromPlaylist({
        playlist_id: playlistId,
        track_ids: trackIds,
      });
      if (this.selectedPlaylist?.playlist.id === playlistId) {
        this.selectedPlaylist = result;
      }
      this.playlists = this.playlists.map((p) =>
        p.id === result.playlist.id ? result.playlist : p,
      );
    } catch (e) {
      this.error = String(e);
    }
  }

  async reorder(playlistId: string, trackIds: number[]) {
    this.error = null;
    try {
      const result = await commands.reorderPlaylist({
        playlist_id: playlistId,
        track_ids: trackIds,
      });
      if (this.selectedPlaylist?.playlist.id === playlistId) {
        this.selectedPlaylist = result;
      }
    } catch (e) {
      this.error = String(e);
    }
  }
}

export const playlistStore = new PlaylistStore();
