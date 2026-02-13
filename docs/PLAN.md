# Music Sync Desktop App - Implementation Plan

## Context

Build a greenfield Tauri v2 desktop application to synchronize music between drives (NAS, DAP, laptop). The app provides a browsable music library (Artist > Album > Track), configurable sync profiles, and both one-way and two-way sync with conflict resolution.

## Tech Stack

- **Backend**: Tauri v2 (Rust)
- **Frontend**: Svelte 5 + TypeScript + Vite
- **Database**: SQLite via rusqlite (bundled)
- **Key Rust crates**: `lofty` (metadata), `blake3` (hashing), `walkdir` (traversal), `thiserror`, `serde`
- **Audio formats**: FLAC, MP3, AAC/M4A, WAV, ALAC, OGG, OPUS, WMA

## Project Structure

```
music-management/
├── src/                          # Svelte frontend
│   ├── App.svelte                # Root with simple page router
│   ├── lib/
│   │   ├── api/
│   │   │   ├── commands.ts       # Typed invoke() wrappers
│   │   │   ├── events.ts         # Typed Channel listeners
│   │   │   └── types.ts          # TS interfaces mirroring Rust structs
│   │   ├── stores/               # Svelte 5 rune-based state
│   │   │   ├── library.svelte.ts
│   │   │   ├── sync.svelte.ts
│   │   │   └── profiles.svelte.ts
│   │   └── components/           # TreeView, ProgressBar, DiffView, etc.
│   └── pages/                    # Library, SyncProfiles, SyncPreview, SyncProgress, Conflicts, Settings
└── src-tauri/                    # Rust backend
    └── src/
        ├── lib.rs                # Tauri Builder, state init, command registration
        ├── error.rs              # Unified AppError enum
        ├── commands/             # IPC surface (library, sync, profile, settings)
        ├── scanner/              # walker.rs, metadata.rs, hasher.rs
        ├── sync/                 # diff.rs, one_way.rs, two_way.rs, conflict.rs, progress.rs
        ├── db/                   # schema.rs, library_repo.rs, profile_repo.rs, sync_state_repo.rs
        └── models/               # track.rs, library.rs, sync_profile.rs, diff.rs, conflict.rs, progress.rs
```

## Key Architectural Decisions

1. **No SvelteKit** - plain Svelte 5 + Vite with a simple conditional-rendering router (no SSR needed for desktop)
2. **Tauri v2 Channels** for progress reporting (ordered, fast, low overhead vs events)
3. **rusqlite with bundled SQLite** - no external deps, simple sync access behind Mutex
4. **Baseline-based two-way sync** - store file state snapshots after each sync to detect which side changed
5. **Lazy hashing** - BLAKE3 computed during diff, not during initial scan
6. **Copy-then-rename** for safe file writes (temp file → fsync → atomic rename)
7. **Transcoding designed-for but not built** - models have extension points for future `TranscodeConfig`

## Two-Way Sync Algorithm

Uses a three-way comparison (source, target, baseline from last sync):
- **Neither changed** → Unchanged
- **Only source changed** → Copy to target
- **Only target changed** → Copy to source
- **Both changed, same hash** → Unchanged (convergent edit)
- **Both changed, different hash** → Conflict (user resolves: keep source/target/both/skip)
- **File deleted on one side, unchanged on other** → Propagate delete
- **File deleted on one side, changed on other** → Conflict
- **First sync** (no baseline) → New files merge; differing files conflict

## Implementation Phases

### Phase 1: Scaffolding + Scanner + Library Browser
1. Scaffold Tauri v2 project with Svelte 5 + TS + Vite
2. Set up Rust module structure and all Cargo dependencies
3. Implement SQLite schema + migrations (`tracks`, `sync_profiles`, `sync_state`, `settings`)
4. Implement scanner pipeline: `walker.rs` → `metadata.rs` → `hasher.rs`
5. Implement `scan_directory` command with Channel progress
6. Implement `get_library_tree` command (builds Artist > Album > Track from DB)
7. Build App.svelte navigation shell and Library.svelte page
8. Build TreeView component and PathPicker (using `@tauri-apps/plugin-dialog`)

**Milestone**: User picks a directory, scans it, browses Artist > Album > Track with metadata.

### Phase 2: Sync Profiles + One-Way Sync
1. Implement profile CRUD (Rust repo + commands + frontend)
2. Build SyncProfiles.svelte with ProfileCard components
3. Implement `sync/diff.rs` one-way diff algorithm
4. Implement `compute_diff` command
5. Build SyncPreview.svelte with DiffView showing adds/removes/updates
6. Implement `sync/one_way.rs` executor with copy-then-rename
7. Implement progress reporting + cancellation (AtomicBool flag)
8. Build SyncProgress.svelte with ProgressBar

**Milestone**: Create profile, preview changes, execute one-way sync with live progress, cancel mid-sync.

### Phase 3: Two-Way Sync + Conflict Resolution
1. Implement `sync_state_repo.rs` for baseline snapshots
2. Implement `sync/two_way.rs` with three-way comparison
3. Implement conflict detection and resolution application
4. Build Conflicts.svelte with ConflictCard components
5. Update execute_sync to accept ConflictResolution list
6. Snapshot state to `sync_state` after successful sync

**Milestone**: Full bidirectional sync with conflict UI.

### Phase 4: Polish + Settings + Edge Cases
1. Build Settings page (hash mode, default sync mode)
2. Add exclude pattern support (glob filtering)
3. Library search command
4. Error handling for disconnected drives, permissions, full disks
5. Handle large libraries (virtual scrolling)
6. Rust unit tests for diff and sync logic

## Data Models

### Rust Structs

**TrackInfo**: path, relativePath, artist, title, album, genre, trackNumber, discNumber, year, durationSecs, bitrateKbps, format (AudioFormat enum), fileSize, modifiedAt, blake3Hash

**LibraryTree**: artists (Vec<ArtistNode>), totalTracks, totalSizeBytes
- **ArtistNode**: name, albums (Vec<AlbumNode>), trackCount
- **AlbumNode**: name, artist, year, tracks (Vec<TrackInfo>), trackCount

**SyncProfile**: id, name, sourcePath, targetPath, mode (OneWay|TwoWay), deleteOrphans, excludePatterns, createdAt, updatedAt, lastSyncedAt

**DiffResult**: entries (Vec<DiffEntry>), additions, deletions, updates, conflicts, unchanged, totalBytesToTransfer
- **DiffEntry**: relativePath, action (CopyToTarget|CopyToSource|DeleteFromTarget|DeleteFromSource|Conflict|Unchanged), sourceInfo, targetInfo, conflictDetail

**ConflictResolution**: relativePath, strategy (KeepSource|KeepTarget|KeepBoth|Skip)

**SyncProgressEvent** (tagged union): Scanning|Hashing|Transferring|Completed|Error|Cancelled

### SQLite Schema

- **tracks**: id, path (unique), relative_path, library_root, artist, title, album, genre, track_number, disc_number, year, duration_secs, bitrate_kbps, format, file_size, modified_at, blake3_hash, scanned_at
- **sync_profiles**: id, name (unique), source_path, target_path, mode, delete_orphans, exclude_patterns (JSON), created_at, updated_at, last_synced_at
- **sync_state**: id, profile_id (FK), relative_path, source_size, source_modified, source_hash, target_size, target_modified, target_hash, snapshot_at; UNIQUE(profile_id, relative_path)
- **settings**: key (PK), value

## Tauri Command API

| Command | Purpose |
|---------|---------|
| `scan_directory(path, on_progress)` | Scan dir, extract metadata, cache in DB |
| `get_library_tree()` | Return Artist > Album > Track tree from DB |
| `get_track_metadata(path)` | Full metadata for one track |
| `search_library(query)` | Search tracks by text |
| `compute_diff(profile_id)` | Compute sync diff for a profile |
| `execute_sync(profile_id, resolutions?, on_progress)` | Execute sync with progress |
| `cancel_sync()` | Cancel active sync |
| `list_profiles()` / `get_profile(id)` | Read profiles |
| `create_profile(data)` / `update_profile(id, data)` / `delete_profile(id)` | Profile CRUD |
| `get_settings()` / `update_settings(settings)` | App settings |

## Verification

1. `cargo build` in src-tauri to verify Rust compiles
2. `npm run tauri dev` to launch the app
3. Test: scan a real music directory, verify library tree renders correctly
4. Test: create one-way sync profile between two folders with test music files
5. Test: preview shows correct diff, execute copies files, progress works
6. Test: modify files on both sides, run two-way sync, verify conflicts detected
7. Test: resolve conflicts, verify correct files propagated
8. Test: cancel mid-sync, verify no partial files and next sync resumes correctly
