# Music Sync Desktop App - Project Memory

## Tech Stack
- **Backend**: Tauri v2 (Rust) with rusqlite (bundled SQLite), lofty, blake3, walkdir
- **Frontend**: Svelte 5 (runes) + TypeScript + Vite (no SvelteKit)
- **Tauri v2 Channels** for progress reporting (not events)

## Key Paths
- Rust source: `src-tauri/src/` (lib.rs, commands/, scanner/, sync/, db/, models/)
- Frontend: `src/` (App.svelte, lib/api/, lib/stores/, lib/components/, pages/)
- DB: SQLite at app data dir, tables: tracks, sync_profiles, sync_state, settings

## Architecture Notes
- Simple conditional-rendering router in App.svelte (no SvelteKit router)
- Svelte 5 rune-based stores (class with $state/$derived)
- Baseline-based two-way sync with three-way comparison
- Copy-then-rename for safe file writes
- CancelToken (AtomicBool) for sync cancellation
- `tauri::Manager` trait must be imported for `app.path()` and `app.manage()`
- Tauri v2 config: no `title` field under `app` (only in windows array)
- Icons must be RGBA PNGs for Tauri build

## See Also
- [patterns.md](patterns.md) — Implementation patterns and lessons learned

## Test Commands
- `cargo test` from `src-tauri/` - 13 unit tests for diff/sync logic
- `npm run build` - Vite frontend build
- `npm run tauri dev` - Full dev mode
