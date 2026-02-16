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
cargo test           # Run all unit tests
cargo test test_name # Run a single test by name
```

## Architecture

Tauri v2 desktop app with a Rust backend and Svelte 5 frontend communicating via IPC.

### IPC Pattern

Frontend calls Rust via typed `invoke()` wrappers in `src/lib/api/commands.ts`. For long-running operations (scan, sync), a Tauri v2 `Channel<ProgressEvent>` is passed as a parameter — the Rust side sends progress events through it, and the frontend listens via `channel.onmessage`. The channel must be set up *before* invoking the command.

All 14 commands are registered in `src-tauri/src/lib.rs` via `generate_handler![]`.

**UI test mocks**: When adding a new `invoke()` command, also add a handler in `e2e/tauri-mocks.ts` so the `/ui-test` Playwright tests render the page with data instead of error/empty state.

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
- Artist grouping uses `COALESCE(album_artist, artist, 'Unknown Artist')` — this is the canonical SQL pattern for display-artist across all queries
- `tauri::Manager` trait must be imported when using `app.path()` or `app.manage()`
- **Svelte 5 prop-to-local-state pattern**: When a component needs an editable local copy of a prop, use `// svelte-ignore state_referenced_locally` above `let x = $state(prop.field)`. This is the correct Svelte 5 approach — intermediate-const workarounds do not work. Add the comment once per group of initializations. See `MetadataEditor.svelte` for the canonical example.
- **No hardcoded colors in components**: All colors in `.svelte` `<style>` blocks must use CSS custom properties from `src/app.css` (e.g., `var(--bg-primary)`, `var(--accent)`, `var(--border)`). Never introduce raw hex (`#fff`), `rgb()`, or `rgba()` values — use or extend the theme variables in `app.css` instead. This ensures UI skins/themes work correctly.
- **`track_from_row` positional column convention**: `library_repo::track_from_row` maps Track columns by positional index (0-18). All SELECT statements that feed into `track_from_row` must list columns in the exact same order: `id, file_path, relative_path, library_root, title, artist, album_artist, album, track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate`. Adding a column requires updating `track_from_row` AND every SELECT that uses it. See `library_repo.rs`, `playlist_repo.rs`, and `favorite_repo.rs`.
- **File deletion is permanent** (`std::fs::remove_file`): Sync operations delete files permanently because the source copy still exists. For user-initiated destructive operations (e.g., duplicate deletion), permanent deletion is acceptable when behind an explicit confirm dialog. If a future feature deletes files where no other copy exists, consider adding the `trash` crate for OS trash-bin support.
- **`libraryStore.libraryRoot` is falsy until a scan or load completes**: `libraryRoot` is typed `string` and initialized to `""`. It stays `""` if no library has been scanned yet, if `getSetting("library_root")` returns null/undefined on startup, or before `loadTree` resolves. `tree` can be non-null while `libraryRoot` is still `""` during the brief async gap. Always guard reads of `libraryRoot` before using it in commands or passing it to child components: `if (libraryStore.libraryRoot) { ... }` and `{#if condition && libraryStore.libraryRoot}` in templates. See `Library.svelte` for the canonical guarding pattern.

## Memory

See `.claude/memory/MEMORY.md` for project context, architecture notes, and implementation patterns.
