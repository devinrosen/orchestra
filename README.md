# Orchestra

A desktop application for synchronizing music libraries between drives — NAS, DAP (digital audio player), laptop, or any mounted volume. Built with Tauri v2, Svelte 5, and Rust.

## Features

### Library
- **Library Browser** — Scan any directory and browse by Artist, Album, Genre, or Folder with full metadata (title, duration, format, size)
- **Contextual Search** — Search filters contextually based on the active view mode, matching at the top-level grouping while preserving the tree structure
- **Browse / Manage Tabs** — Browse tab for searching and playback, Manage tab for library maintenance (rescan, duplicates, metadata report, statistics)
- **Duplicate Detection** — Scan for duplicate tracks by content hash and review/remove them
- **Metadata Report** — Flag tracks missing key metadata fields (title, artist, album, album art)
- **Metadata Editor** — View and edit track metadata individually or in bulk for an album, writing changes back to audio files
- **Library Statistics** — Dashboard showing format breakdown, genre distribution, total size, artist/album/track counts, and average bitrate
- **Rescan** — Re-scan a loaded library to pick up new, changed, or deleted files incrementally

### Playback
- **Music Playback** — Play individual tracks or full albums directly from the library
- **Playlists** — Create, manage, and reorder playlists with export to M3U/PLS formats
- **Play Queue** — View the current play queue, skip ahead, and reorder or remove upcoming tracks
- **Visualizations** — Real-time audio visualizations (waveform, frequency spectrum, radial) via Web Audio API
- **Equalizer** — 10-band graphic EQ with presets (flat, bass boost, vocal, etc.) and manual per-band adjustment

### Sync
- **Sync Profiles** — Create named profiles linking a source and target directory, with configurable sync mode and exclude patterns
- **One-Way Sync** — Mirror source to target: adds, updates, and removals
- **Two-Way Sync** — Bidirectional sync with baseline-based three-way comparison to detect which side changed
- **Sync by Album** — Select individual albums to sync, not just entire artists
- **Conflict Resolution** — When both sides changed, choose per-file: keep source, keep target, keep both, or skip
- **Live Progress** — Real-time file-by-file progress with expandable detail panel and cancellation support
- **Safe Writes** — Copy-then-rename pattern prevents partial files on crash or cancel
- **Eject Device** — Safely unmount a connected device directly from the app
- **Exclude Patterns** — Glob-based filtering to skip files (e.g., `*.tmp`, `.DS_Store`)

### Terminal UI (orchestra-tui)
- **Library Browsing** — 3-pane artist/album/track browser reading from the existing Orchestra database (read-only)
- **Audio Playback** — Play tracks via rodio with play/pause, next/previous, and volume controls
- **Vim-Style Navigation** — `j`/`k` or arrow keys to navigate, `Tab`/`Shift+Tab` to cycle panes, `Enter` to select/play, `Space` to toggle pause, `n`/`p` for next/prev, `+`/`-` for volume, `q` to quit

### UI
- **Light / Dark Mode** — Light and dark themes with system appearance detection and manual override
- **Global Status Bar** — Expandable progress bar showing current file, files completed/total, bytes transferred, and elapsed time

## Supported Formats

FLAC, MP3, AAC/M4A, WAV, ALAC, OGG, OPUS, WMA

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.70
- Platform-specific Tauri v2 dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Getting Started

```bash
# Install dependencies
npm install

# Run in development mode (frontend HMR + Rust backend)
npm run tauri dev

# Build for production
npm run tauri build
```

## Terminal UI

Orchestra includes a standalone terminal UI for browsing and playing your library without the desktop app running. It reads the same SQLite database (read-only) — you must scan a library in the desktop app first.

```bash
# Build and run from src-tauri/
cargo run -p orchestra-tui

# Or specify a custom database path
cargo run -p orchestra-tui -- --db /path/to/orchestra.db
```

The default database location is `~/Library/Application Support/com.orchestra.app/orchestra.db` on macOS.

## Running Tests

```bash
# Rust unit tests (from project root)
cd src-tauri && cargo test
```

## How Two-Way Sync Works

After each successful sync, the app snapshots the state (content hash, modification time, size) of every file on both sides. On the next sync, it performs a three-way comparison:

| Source | Target | Baseline | Result |
|--------|--------|----------|--------|
| unchanged | unchanged | exists | No action |
| changed | unchanged | exists | Copy source to target |
| unchanged | changed | exists | Copy target to source |
| changed | changed (same hash) | exists | No action (convergent edit) |
| changed | changed (different) | exists | **Conflict** |
| deleted | unchanged | exists | Propagate delete |
| deleted | changed | exists | **Conflict** |
| exists | missing | missing | Copy to target (first sync) |
| missing | exists | missing | Copy to source (first sync) |
| exists | exists (different) | missing | **Conflict** (first sync) |

## Tech Stack

- **Backend**: Rust with Tauri v2, rusqlite (bundled SQLite), lofty (metadata), blake3 (hashing), walkdir
- **Frontend**: Svelte 5, TypeScript, Vite
- **IPC**: Tauri commands with Channel-based progress streaming
- **TUI**: Ratatui, crossterm, rodio (audio playback via symphonia)

## License

MIT
