# Plan: Missing/Incomplete Metadata Report

## Summary

Add a feature that flags tracks missing key metadata fields (title, artist, album, album art) so users can review and fix them. This adds a new Rust backend command to query incomplete tracks from the database, a new TypeScript type and IPC wrapper, and a new `MetadataReport` Svelte component accessible from the Library page.

## Metadata Fields to Check

The "key" metadata fields to flag as missing/incomplete:
- **title** — `NULL` in DB
- **artist** — `NULL` in DB
- **album** — `NULL` in DB
- **album art** — requires reading the audio file (not stored in DB); checked on-demand per track via the existing `get_track_artwork` command

Since album art presence is not stored in the database, the backend query will flag tracks missing title, artist, or album. Album art will be checked lazily on the frontend (or optionally via a dedicated batch command).

**Design decision**: Add a `has_album_art` boolean column to the `tracks` table. This allows the backend to report all four missing-field types in a single query without re-reading every audio file. The scanner already reads each file with `lofty` — we just need to check for pictures at scan time and store the result.

## Backend Changes

### 1. Add `has_album_art` column to tracks table (`db/schema.rs`)

Add a migration that adds a `has_album_art` column (INTEGER, default 0) to the `tracks` table:

```sql
ALTER TABLE tracks ADD COLUMN has_album_art INTEGER NOT NULL DEFAULT 0;
```

Use a conditional migration approach: check if the column exists first, and only add it if missing. This keeps backward compatibility with existing databases.

### 2. Update Track model (`models/track.rs`)

Add `has_album_art: bool` field to the `Track` struct.

### 3. Update scanner metadata extraction (`scanner/metadata.rs`)

In `extract_metadata()`, after reading the tag, check if any pictures are present:

```rust
let has_album_art = tag.map(|t| !t.pictures().is_empty()).unwrap_or(false);
```

Set `track.has_album_art = has_album_art`.

### 4. Update `library_repo.rs`

- Update `upsert_track` to include `has_album_art` in the INSERT and ON CONFLICT UPDATE.
- Update all `SELECT` queries that build `Track` structs to include `has_album_art` (in `get_library_tree`, `search_tracks`, `get_tracks_by_artists`).

### 5. Add `get_incomplete_tracks` query (`db/library_repo.rs`)

New function:

```rust
pub fn get_incomplete_tracks(
    conn: &Connection,
    library_root: &str,
) -> Result<Vec<Track>, AppError>
```

SQL query:
```sql
SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
       track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art
FROM tracks
WHERE library_root = ?1
  AND (title IS NULL OR artist IS NULL OR album IS NULL OR has_album_art = 0)
ORDER BY COALESCE(album_artist, artist) COLLATE NOCASE, album COLLATE NOCASE, track_number
```

### 6. Add IPC command (`commands/library.rs`)

New command:

```rust
#[tauri::command]
pub async fn get_incomplete_tracks(
    db: tauri::State<'_, Mutex<Connection>>,
    root: String,
) -> Result<Vec<Track>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::get_incomplete_tracks(&conn, &root)
}
```

### 7. Register command (`lib.rs`)

Add `commands::library::get_incomplete_tracks` to the `generate_handler![]` macro.

## Frontend Changes

### 1. Update TypeScript types (`src/lib/api/types.ts`)

Add `has_album_art: boolean` to the `Track` interface.

### 2. Add IPC wrapper (`src/lib/api/commands.ts`)

```typescript
export function getIncompleteTracks(root: string): Promise<Track[]> {
  return invoke("get_incomplete_tracks", { root });
}
```

### 3. New component: `MetadataReport.svelte` (`src/lib/components/MetadataReport.svelte`)

A panel/overlay component that:
- Calls `getIncompleteTracks(libraryRoot)` on mount
- Displays a summary banner: "X tracks with missing metadata"
- Groups tracks by what's missing (using badge/tag indicators):
  - Missing title
  - Missing artist
  - Missing album
  - Missing album art
- Each track row shows:
  - File path (relative)
  - Which fields are missing (colored badges)
  - An "Edit" button that opens the existing `MetadataEditor`
- Filter/toggle buttons to show only tracks missing specific fields
- "Refresh" button to re-fetch the list

### 4. Add report trigger to Library page (`src/pages/Library.svelte`)

Add a "Metadata Report" button in the library header actions (next to Rescan / Open Directory). When clicked, it shows the `MetadataReport` component as an overlay or inline panel.

The button should show a warning indicator (e.g., colored dot or count badge) if there are incomplete tracks, fetched after each scan/tree load.

### 5. Library store additions (`src/lib/stores/library.svelte.ts`)

Add to `LibraryStore`:
- `incompleteCount = $state<number>(0)` — count of tracks with missing metadata
- `async loadIncompleteCount(root: string)` — calls `getIncompleteTracks` and sets count (or a lighter-weight count command)

Call `loadIncompleteCount` after `loadTree` completes so the badge is always current.

## Data Flow

```
User clicks "Metadata Report" button in Library header
  -> MetadataReport.svelte mounts
  -> calls getIncompleteTracks(libraryRoot)
  -> invoke("get_incomplete_tracks", { root })
  -> Rust: commands::library::get_incomplete_tracks
  -> db::library_repo::get_incomplete_tracks (SQL query)
  -> Returns Vec<Track> with missing fields
  -> Frontend renders grouped list with missing-field badges
  -> User clicks "Edit" on a track
  -> Opens existing MetadataEditor
  -> After save, re-fetches incomplete list
```

## Files to Create

| File | Description |
|------|-------------|
| `src/lib/components/MetadataReport.svelte` | New report UI component |

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/db/schema.rs` | Add `has_album_art` column migration |
| `src-tauri/src/models/track.rs` | Add `has_album_art` field to `Track` |
| `src-tauri/src/scanner/metadata.rs` | Extract album art presence during scan |
| `src-tauri/src/db/library_repo.rs` | Update all queries for new column; add `get_incomplete_tracks` |
| `src-tauri/src/commands/library.rs` | Add `get_incomplete_tracks` command |
| `src-tauri/src/commands/metadata_cmd.rs` | Update `update_track_metadata` to refresh `has_album_art` after write |
| `src-tauri/src/lib.rs` | Register new command |
| `src/lib/api/types.ts` | Add `has_album_art` to `Track` |
| `src/lib/api/commands.ts` | Add `getIncompleteTracks` wrapper |
| `src/lib/stores/library.svelte.ts` | Add incomplete count tracking |
| `src/pages/Library.svelte` | Add "Metadata Report" button with badge; render MetadataReport overlay |

## Edge Cases & Design Decisions

1. **Album art storage**: Rather than re-reading every file on each report request, we store `has_album_art` at scan time. This makes the report query fast (pure SQL).

2. **Migration safety**: The `ALTER TABLE ADD COLUMN` is idempotent — we check if the column exists first. Existing tracks will default to `has_album_art = 0`, which means they'll appear in the report until the next rescan populates the field. This is acceptable since it encourages a rescan after the update.

3. **Empty strings vs NULL**: The scanner sets fields to `None`/`NULL` when metadata is absent (not empty strings). The SQL query checks for `IS NULL` which is correct.

4. **Performance**: The query filters server-side in SQL, so even large libraries (tens of thousands of tracks) return only the subset with issues. No full-library transfer needed.

5. **After metadata edit**: When the user edits a track via `MetadataEditor` and saves, the `MetadataReport` should re-fetch the list so fixed tracks disappear from the report.

6. **Report re-entrancy with scan**: If a scan is in progress, the report button should be disabled or show a note that results may be stale.

## Implementation Order

1. Backend: schema migration + model update
2. Backend: scanner update to populate `has_album_art`
3. Backend: update `library_repo.rs` queries + add `get_incomplete_tracks`
4. Backend: add command + register in `lib.rs`
5. Frontend: update types + add command wrapper
6. Frontend: add library store tracking
7. Frontend: create `MetadataReport.svelte` component
8. Frontend: integrate into `Library.svelte`
9. Run `cargo test` and `npm run check`
