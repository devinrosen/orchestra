# Orchestra

A desktop application for synchronizing music libraries between drives — NAS, DAP (digital audio player), laptop, or any mounted volume. Built with Tauri v2, Svelte 5, and Rust.

## Features

- **Library Browser** — Scan any directory and browse by Artist > Album > Track with full metadata (title, duration, format, size)
- **Sync Profiles** — Create named profiles linking a source and target directory, with configurable sync mode and exclude patterns
- **One-Way Sync** — Mirror source to target: adds, updates, and removals
- **Two-Way Sync** — Bidirectional sync with baseline-based three-way comparison to detect which side changed
- **Conflict Resolution** — When both sides changed, choose per-file: keep source, keep target, keep both, or skip
- **Live Progress** — Real-time file-by-file progress with cancellation support
- **Safe Writes** — Copy-then-rename pattern prevents partial files on crash or cancel
- **Search** — Find tracks across your library by title, artist, or album
- **Exclude Patterns** — Glob-based filtering to skip files (e.g., `*.tmp`, `.DS_Store`)

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

## License

MIT
