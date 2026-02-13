# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Full development mode (frontend HMR + Rust backend)
npm run tauri dev

# Frontend only
npm run dev          # Vite dev server on localhost:1420
npm run build        # Production build to dist/
npm run check        # Svelte + TypeScript type checking

# Rust backend (from src-tauri/)
cargo build          # Debug build
cargo test           # Run all unit tests (13 tests in sync/diff.rs and sync/two_way.rs)
cargo test test_name # Run a single test by name
```

## Architecture

Tauri v2 desktop app with a Rust backend and Svelte 5 frontend communicating via IPC.

### IPC Pattern

Frontend calls Rust via typed `invoke()` wrappers in `src/lib/api/commands.ts`. For long-running operations (scan, sync), a Tauri v2 `Channel<ProgressEvent>` is passed as a parameter — the Rust side sends progress events through it, and the frontend listens via `channel.onmessage`. The channel must be set up *before* invoking the command.

All 14 commands are registered in `src-tauri/src/lib.rs` via `generate_handler![]`.

### State Management

- **Rust**: SQLite connection behind `Mutex<Connection>` managed as Tauri state. A `Mutex<CancelToken>` (wrapping `Arc<AtomicBool>`) enables sync cancellation.
- **Frontend**: Svelte 5 rune-based stores — classes using `$state` and `$derived` runes, exported as singletons (e.g., `libraryStore`, `profilesStore`, `syncStore`).

### Sync Flow

Three-phase pattern: **Diff → Preview → Execute**

1. `compute_diff` scans source/target, compares files, returns `(DiffResult, Vec<Conflict>)`
2. Frontend shows diff summary + conflict resolution UI
3. `execute_sync` copies/deletes files with progress reporting and cancellation support

**Two-way sync** uses baseline-based three-way comparison: after each sync, file state (hash, mtime, size) is snapshotted to the `sync_state` table. Future diffs compare current state against baseline to determine which side changed.

**Safe writes**: copy-then-rename pattern (`tmp_sync` → `fsync` → atomic rename), preserving source mtime.

### Key Modules

- **commands/** — IPC surface. Each command is `async` with `tauri::State` injection for DB and CancelToken.
- **sync/diff.rs** — One-way diff: compares by size+mtime first, then lazy BLAKE3 hash.
- **sync/two_way.rs** — Three-way diff using baselines. Handles: both-modified, deleted-and-modified, first-sync-differs.
- **scanner/** — `walkdir` traversal → `lofty` metadata extraction → `blake3` hashing. Hashing is lazy (only during diff, not scan).
- **db/** — rusqlite repos. Schema uses WAL mode, foreign keys. `sync_state` table keyed by `(profile_id, relative_path)`.
- **models/** — Shared types serialized via serde between Rust and TypeScript. `ProgressEvent` is a tagged enum.

### Frontend Structure

Simple conditional-rendering router in `App.svelte` (no SvelteKit). Pages: Library (scan + Artist>Album>Track tree), SyncProfiles (CRUD), SyncPreview (diff + conflicts + progress), Settings. Components are in `src/lib/components/`.

TypeScript types in `src/lib/api/types.ts` mirror Rust structs exactly.

## Conventions

- Audio formats supported: FLAC, MP3, M4A/AAC, WAV, ALAC, OGG, OPUS, WMA (defined in `models/track.rs:AUDIO_EXTENSIONS`)
- Exclude patterns use `glob` crate syntax
- Profile IDs are UUIDv4 strings
- All timestamps are Unix epoch seconds (i64)
- `AppError` serializes to string for frontend consumption
- `tauri::Manager` trait must be imported when using `app.path()` or `app.manage()`
