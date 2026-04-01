import { convertFileSrc } from "@tauri-apps/api/core";
import type { Track, AlbumArt } from "../api/types";
import { getTrackArtwork, recordPlay } from "../api/commands";
import { equalizerStore } from "./equalizer.svelte";

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
  visualizerActive = $state(false);
  equalizerActive = $state(false);

  currentTrack = $derived(this.queue.length > 0 ? this.queue[this.queueIndex] : null);
  hasNext = $derived(this.queueIndex < this.queue.length - 1);
  hasPrev = $derived(this.queueIndex > 0);
  progress = $derived(this.duration > 0 ? this.currentTime / this.duration : 0);

  private audio: HTMLAudioElement | null = null;
  private pendingPlay = false;
  private audioContext: AudioContext | null = null;
  private analyser: AnalyserNode | null = null;
  private sourceNode: MediaElementAudioSourceNode | null = null;
  private lastPositionUpdateTime = 0;

  getAnalyser(): AnalyserNode | null {
    return this.analyser;
  }

  toggleVisualizer() {
    this.visualizerActive = !this.visualizerActive;
    if (this.visualizerActive && !this.audioContext && this.audio) {
      this.initAudioContext();
    }
  }

  toggleEqualizer() {
    this.equalizerActive = !this.equalizerActive;
    if (this.equalizerActive && !this.audioContext && this.audio) {
      this.initAudioContext();
    }
  }

  private initAudioContext() {
    if (!this.audio || this.audioContext) return;
    this.audioContext = new AudioContext();
    this.analyser = this.audioContext.createAnalyser();
    this.analyser.fftSize = 2048;
    this.analyser.smoothingTimeConstant = 0.8;
    this.sourceNode = this.audioContext.createMediaElementSource(this.audio);
    equalizerStore.connectFilters(
      this.audioContext,
      this.sourceNode,
      this.analyser,
    );
    this.analyser.connect(this.audioContext.destination);
    equalizerStore.loadState();
  }

  bindAudio(el: HTMLAudioElement) {
    this.audio = el;
    el.crossOrigin = "anonymous";
    el.volume = this.volume;

    el.addEventListener("timeupdate", () => {
      this.currentTime = el.currentTime;
      this.duration = el.duration || 0;
      // Throttle position state updates to ~1 Hz
      const now = Date.now();
      if (now - this.lastPositionUpdateTime > 1000) {
        this.lastPositionUpdateTime = now;
        if ("mediaSession" in navigator && el.duration && isFinite(el.duration)) {
          navigator.mediaSession.setPositionState({
            duration: el.duration,
            playbackRate: el.playbackRate,
            position: el.currentTime,
          });
        }
      }
    });

    el.addEventListener("ended", () => {
      this.handleTrackEnded();
    });

    el.addEventListener("play", () => {
      this.playing = true;
      if ("mediaSession" in navigator) navigator.mediaSession.playbackState = "playing";
    });

    el.addEventListener("pause", () => {
      this.playing = false;
      if ("mediaSession" in navigator) navigator.mediaSession.playbackState = "paused";
    });

    // Wire OS media keys / Now Playing controls via the Web Media Session API.
    // This runs in both Tauri and the Playwright UI-test environment.
    if ("mediaSession" in navigator) {
      navigator.mediaSession.setActionHandler("play", () => this.safePlay());
      navigator.mediaSession.setActionHandler("pause", () => this.audio?.pause());
      navigator.mediaSession.setActionHandler("nexttrack", () => this.next());
      navigator.mediaSession.setActionHandler("previoustrack", () => this.previous());
      navigator.mediaSession.setActionHandler("seekto", (details) => {
        if (details.seekTime != null) this.seek(details.seekTime);
      });
    }

    el.addEventListener("error", () => {
      const code = el.error?.code;
      const msg = el.error?.message || "Unknown playback error";
      if (code === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED) {
        this.error = "File not accessible — check that your library drive is connected";
      } else if (code === MediaError.MEDIA_ERR_NETWORK) {
        this.error = "File not accessible — check that your library drive is connected";
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
      this.safePlay();
    }
  }

  jumpTo(index: number) {
    if (index < 0 || index >= this.queue.length) return;
    this.queueIndex = index;
    this.loadAndPlay();
  }

  removeFromQueue(index: number) {
    if (index < 0 || index >= this.queue.length) return;
    if (this.queue.length === 1) {
      this.queue = [];
      this.queueIndex = 0;
      if (this.audio) {
        this.audio.pause();
        this.audio.src = "";
      }
      this.playing = false;
      this.currentTime = 0;
      this.duration = 0;
      this.artwork = null;
      return;
    }
    if (index < this.queueIndex) {
      this.queueIndex--;
      this.queue.splice(index, 1);
    } else if (index === this.queueIndex) {
      this.queue.splice(index, 1);
      if (this.queueIndex >= this.queue.length) {
        this.queueIndex = this.queue.length - 1;
      }
      this.loadAndPlay();
    } else {
      this.queue.splice(index, 1);
    }
  }

  moveInQueue(fromIndex: number, toIndex: number) {
    if (
      fromIndex === toIndex ||
      fromIndex < 0 ||
      fromIndex >= this.queue.length ||
      toIndex < 0 ||
      toIndex >= this.queue.length
    )
      return;
    const currentFilePath = this.currentTrack?.file_path;
    const [moved] = this.queue.splice(fromIndex, 1);
    this.queue.splice(toIndex, 0, moved);
    if (currentFilePath) {
      const newIndex = this.queue.findIndex(
        (t) => t.file_path === currentFilePath,
      );
      if (newIndex >= 0) this.queueIndex = newIndex;
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

  private safePlay() {
    this.audio?.play().catch((err) => {
      if ((err as DOMException).name !== "AbortError") this.error = String(err);
    });
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
    this.safePlay();
    this.loadArtwork();
    // Record play — fire-and-forget; non-critical if it fails
    if (this.currentTrack.id != null) {
      recordPlay(this.currentTrack.id).catch(() => {});
    }
    // Set OS Now Playing metadata (artwork updated async in loadArtwork)
    if ("mediaSession" in navigator) {
      navigator.mediaSession.metadata = new MediaMetadata({
        title: this.currentTrack.title ?? undefined,
        artist: this.currentTrack.artist ?? this.currentTrack.album_artist ?? undefined,
        album: this.currentTrack.album ?? undefined,
      });
    }
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
    if ("mediaSession" in navigator && this.artwork && this.currentTrack) {
      const track = this.currentTrack;
      navigator.mediaSession.metadata = new MediaMetadata({
        title: track.title ?? undefined,
        artist: track.artist ?? track.album_artist ?? undefined,
        album: track.album ?? undefined,
        artwork: [{ src: `data:${this.artwork.mime_type};base64,${this.artwork.data}`, type: this.artwork.mime_type }],
      });
    }
  }
}

export const playerStore = new PlayerStore();
