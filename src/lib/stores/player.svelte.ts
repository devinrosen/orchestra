import { convertFileSrc } from "@tauri-apps/api/core";
import type { Track, AlbumArt } from "../api/types";
import { getTrackArtwork } from "../api/commands";

class PlayerStore {
  queue = $state<Track[]>([]);
  queueIndex = $state(0);
  playing = $state(false);
  currentTime = $state(0);
  duration = $state(0);
  volume = $state(1);
  muted = $state(false);
  artwork = $state<AlbumArt | null>(null);
  error = $state<string | null>(null);

  currentTrack = $derived(this.queue.length > 0 ? this.queue[this.queueIndex] : null);
  hasNext = $derived(this.queueIndex < this.queue.length - 1);
  hasPrev = $derived(this.queueIndex > 0);
  progress = $derived(this.duration > 0 ? this.currentTime / this.duration : 0);

  private audio: HTMLAudioElement | null = null;
  private pendingPlay = false;

  bindAudio(el: HTMLAudioElement) {
    this.audio = el;
    el.volume = this.volume;

    el.addEventListener("timeupdate", () => {
      this.currentTime = el.currentTime;
      this.duration = el.duration || 0;
    });

    el.addEventListener("ended", () => {
      this.handleTrackEnded();
    });

    el.addEventListener("play", () => {
      this.playing = true;
    });

    el.addEventListener("pause", () => {
      this.playing = false;
    });

    el.addEventListener("error", () => {
      const code = el.error?.code;
      const msg = el.error?.message || "Unknown playback error";
      if (code === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED) {
        this.error = "Format not supported by browser";
      } else if (code === MediaError.MEDIA_ERR_NETWORK) {
        this.error = "File not found or network error";
      } else {
        this.error = msg;
      }
      this.playing = false;
    });

    if (this.pendingPlay) {
      this.pendingPlay = false;
      this.loadAndPlay();
    }
  }

  playTrack(track: Track, albumTracks: Track[]) {
    const index = albumTracks.findIndex((t) => t.file_path === track.file_path);
    this.queue = albumTracks;
    this.queueIndex = index >= 0 ? index : 0;
    this.loadAndPlay();
  }

  playPlaylist(tracks: Track[]) {
    this.queue = tracks;
    this.queueIndex = 0;
    this.loadAndPlay();
  }

  playAlbum(tracks: Track[]) {
    const sorted = [...tracks].sort((a, b) => {
      const discA = a.disc_number ?? 1;
      const discB = b.disc_number ?? 1;
      if (discA !== discB) return discA - discB;
      return (a.track_number ?? 0) - (b.track_number ?? 0);
    });
    this.queue = sorted;
    this.queueIndex = 0;
    this.loadAndPlay();
  }

  togglePlayPause() {
    if (!this.audio) return;
    if (this.playing) {
      this.audio.pause();
    } else {
      this.audio.play();
    }
  }

  next() {
    if (this.hasNext) {
      this.queueIndex++;
      this.loadAndPlay();
    }
  }

  previous() {
    if (this.audio && this.audio.currentTime > 3) {
      this.audio.currentTime = 0;
    } else if (this.hasPrev) {
      this.queueIndex--;
      this.loadAndPlay();
    } else if (this.audio) {
      this.audio.currentTime = 0;
    }
  }

  seek(time: number) {
    if (this.audio) {
      this.audio.currentTime = time;
    }
  }

  setVolume(vol: number) {
    this.volume = vol;
    if (this.audio) {
      this.audio.volume = vol;
    }
  }

  setMuted(m: boolean) {
    this.muted = m;
    if (this.audio) {
      this.audio.muted = m;
    }
  }

  private handleTrackEnded() {
    if (this.hasNext) {
      this.queueIndex++;
      this.loadAndPlay();
    } else {
      this.playing = false;
    }
  }

  private loadAndPlay() {
    if (!this.currentTrack) return;
    if (!this.audio) {
      this.pendingPlay = true;
      return;
    }
    this.error = null;
    const src = convertFileSrc(this.currentTrack.file_path);
    this.audio.src = src;
    this.audio.play();
    this.loadArtwork();
  }

  private async loadArtwork() {
    if (!this.currentTrack) {
      this.artwork = null;
      return;
    }
    try {
      this.artwork = await getTrackArtwork(this.currentTrack.file_path);
    } catch {
      this.artwork = null;
    }
  }
}

export const playerStore = new PlayerStore();
