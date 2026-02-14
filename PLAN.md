# PLAN: Library Statistics

## Overview

Add a "Library Statistics" dashboard page showing aggregate information about the user's music library. The dashboard displays:

- **Total counts**: number of tracks, artists, albums
- **Total library size** (formatted as GB/MB)
- **Total duration** (formatted as hours/minutes)
- **Format breakdown**: count and percentage per audio format (FLAC, MP3, M4A, etc.)
- **Genre distribution**: count and percentage per genre
- **Average bitrate** across the library (kbps)

The statistics are computed via SQL queries on the existing `tracks` table. No new database tables are needed. A new `bitrate` column is added to `tracks` to store per-track bitrate (extracted from lofty during scan).

The page is accessible from the sidebar navigation as "Statistics" and only shows data when a library is loaded.

---

## Backend Changes

### 1. Add `bitrate` field to Track model

**File**: `src-tauri/src/models/track.rs`

Add field to `Track` struct (after `has_album_art`):
```rust
pub bitrate: Option<u32>,  // kbps, from lofty FileProperties::overall_bitrate()
```

### 2. Add `bitrate` column migration

**File**: `src-tauri/src/db/schema.rs`

Add migration after the existing `has_album_art` migration (same pattern):
```rust
let has_bitrate: bool = conn
    .prepare("SELECT COUNT(*) FROM pragma_table_info('tracks') WHERE name='bitrate'")?
    .query_row([], |row| row.get::<_, i64>(0))
    .map(|count| count > 0)?;

if !has_bitrate {
    conn.execute_batch(
        "ALTER TABLE tracks ADD COLUMN bitrate INTEGER;",
    )?;
}
```

### 3. Extract bitrate in scanner

**File**: `src-tauri/src/scanner/metadata.rs`

After the existing `let duration_secs = properties.duration().as_secs_f64();` line, add:
```rust
let bitrate = properties.overall_bitrate();
```

And include `bitrate` in the returned `Track` struct.

### 4. Update `library_repo.rs` -- all SQL and row mappings

**File**: `src-tauri/src/db/library_repo.rs`

- Add `bitrate` to `upsert_track` INSERT/UPDATE columns and params
- Add `bitrate` to SELECT column lists and `Track` construction in all 6 query functions: `get_library_tree`, `search_tracks`, `get_tracks_by_artists`, `get_tracks_by_albums`, `get_tracks_for_device`, `get_incomplete_tracks`
- Column index: `bitrate` becomes index 18 (after `has_album_art` at 17) in all row mappings

### 5. New model structs for statistics

**File**: `src-tauri/src/models/track.rs` (add after existing structs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatStat {
    pub format: String,
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreStat {
    pub genre: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_tracks: usize,
    pub total_artists: usize,
    pub total_albums: usize,
    pub total_size: u64,
    pub total_duration_secs: f64,
    pub avg_bitrate: Option<f64>,
    pub formats: Vec<FormatStat>,
    pub genres: Vec<GenreStat>,
}
```

### 6. New repo function `get_library_stats`

**File**: `src-tauri/src/db/library_repo.rs`

```rust
pub fn get_library_stats(conn: &Connection, library_root: &str) -> Result<LibraryStats, AppError> {
    // Summary row
    let (total_tracks, total_size, total_duration, avg_bitrate): (usize, u64, f64, Option<f64>) =
        conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(file_size), 0), COALESCE(SUM(duration_secs), 0.0),
                    AVG(bitrate)
             FROM tracks WHERE library_root = ?1",
            params![library_root],
            |row| Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, i64>(1)? as u64,
                row.get::<_, f64>(2)?,
                row.get::<_, Option<f64>>(3)?,
            )),
        )?;

    let total_artists: usize = conn.query_row(
        "SELECT COUNT(DISTINCT COALESCE(album_artist, artist, 'Unknown Artist'))
         FROM tracks WHERE library_root = ?1",
        params![library_root],
        |row| row.get::<_, i64>(0).map(|v| v as usize),
    )?;

    let total_albums: usize = conn.query_row(
        "SELECT COUNT(DISTINCT COALESCE(album, 'Unknown Album'))
         FROM tracks WHERE library_root = ?1",
        params![library_root],
        |row| row.get::<_, i64>(0).map(|v| v as usize),
    )?;

    // Format breakdown
    let mut fmt_stmt = conn.prepare(
        "SELECT format, COUNT(*), COALESCE(SUM(file_size), 0)
         FROM tracks WHERE library_root = ?1
         GROUP BY format ORDER BY COUNT(*) DESC",
    )?;
    let formats = fmt_stmt
        .query_map(params![library_root], |row| {
            Ok(FormatStat {
                format: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
                total_size: row.get::<_, i64>(2)? as u64,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Genre breakdown
    let mut genre_stmt = conn.prepare(
        "SELECT COALESCE(genre, 'Unknown'), COUNT(*)
         FROM tracks WHERE library_root = ?1
         GROUP BY COALESCE(genre, 'Unknown') ORDER BY COUNT(*) DESC",
    )?;
    let genres = genre_stmt
        .query_map(params![library_root], |row| {
            Ok(GenreStat {
                genre: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(LibraryStats {
        total_tracks,
        total_artists,
        total_albums,
        total_size,
        total_duration_secs: total_duration,
        avg_bitrate,
        formats,
        genres,
    })
}
```

### 7. New Tauri command

**File**: `src-tauri/src/commands/library.rs`

```rust
#[tauri::command]
pub async fn get_library_stats(
    db: tauri::State<'_, Mutex<Connection>>,
    root: String,
) -> Result<LibraryStats, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::get_library_stats(&conn, &root)
}
```

Add `LibraryStats` to the existing `use crate::models::track::{...}` import.

### 8. Register command

**File**: `src-tauri/src/lib.rs`

Add to `generate_handler![]`:
```rust
commands::library::get_library_stats,
```

---

## Frontend Changes

### 9. TypeScript types

**File**: `src/lib/api/types.ts`

Add at end:
```typescript
export interface FormatStat {
  format: string;
  count: number;
  total_size: number;
}

export interface GenreStat {
  genre: string;
  count: number;
}

export interface LibraryStats {
  total_tracks: number;
  total_artists: number;
  total_albums: number;
  total_size: number;
  total_duration_secs: number;
  avg_bitrate: number | null;
  formats: FormatStat[];
  genres: GenreStat[];
}
```

### 10. Command wrapper

**File**: `src/lib/api/commands.ts`

Add `LibraryStats` to the type import. Add function:
```typescript
export function getLibraryStats(root: string): Promise<LibraryStats> {
  return invoke("get_library_stats", { root });
}
```

### 11. New page: `Statistics.svelte`

**File**: `src/pages/Statistics.svelte`

A dashboard page with these sections:

**Summary cards row**: Six cards showing total tracks, artists, albums, library size, total duration, and average bitrate. Uses `var(--bg-secondary)` background, same border radius as the rest of the app.

**Format breakdown**: A list where each row shows format name (uppercased), count, total size, and a horizontal bar proportional to `count / maxCount`. Sorted by count descending (server-side).

**Genre distribution**: Same layout as format breakdown but showing genre name and count.

Implementation details:
- Uses local `$state` for the stats object, loading state, and error
- Uses `$effect` keyed on `libraryStore.libraryRoot` to reload stats when the library root changes
- Calls `commands.getLibraryStats()` directly (no store needed for a read-only dashboard)
- Utility functions within the component for formatting:
  - `formatBytes(bytes)`: returns "X.X GB" or "X.X MB"
  - `formatDuration(secs)`: returns "Xh Ym" or "Xm Ys"
  - `formatBitrate(kbps)`: returns "X kbps"
- Shows empty state ("No library loaded") when `libraryStore.libraryRoot` is empty
- No store is created for statistics -- the page fetches and holds data locally

Layout:
```
+--------------------------------------------------+
| Library Statistics                                |
+--------------------------------------------------+
| [Tracks: 1,234] [Artists: 89] [Albums: 156]      |
| [Size: 45.2 GB] [Duration: 82h 15m] [Avg: 942k] |
+--------------------------------------------------+
| Format Breakdown           | Genre Distribution   |
| FLAC ████████████ 60% 450  | Rock ██████████ 35%  |
| MP3  ██████ 30% 225        | Jazz █████ 18%       |
| M4A  ██ 10% 75             | Pop  ████ 15%        |
+--------------------------------------------------+
```

### 12. Router integration

**File**: `src/App.svelte`

- Add `"statistics"` to the `Page` type union: `type Page = "library" | "statistics" | "profiles" | ...`
- Import: `import Statistics from "./pages/Statistics.svelte";`
- Add nav item after "Library": `{ page: "statistics", label: "Statistics" }`
- Add render case in the template:
  ```svelte
  {:else if currentPage === "statistics"}
    <Statistics />
  ```

---

## Dead Code

None. All changes are additive. No existing functions become unused.

---

## Test Cases

### Rust unit tests

Add a `stats_tests` module in `src-tauri/src/db/library_repo.rs`:

```rust
#[cfg(test)]
mod stats_tests {
    use super::*;
    use crate::db::schema;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn make_track(
        artist: &str, album: &str, format: &str, genre: &str,
        size: u64, duration: f64, bitrate: Option<u32>,
        file_suffix: &str,
    ) -> Track {
        Track {
            id: None,
            file_path: format!("/music/{}/{}/{}.{}", artist, album, file_suffix, format),
            relative_path: format!("{}/{}/{}.{}", artist, album, file_suffix, format),
            library_root: "/music".to_string(),
            title: Some("Track".to_string()),
            artist: Some(artist.to_string()),
            album_artist: None,
            album: Some(album.to_string()),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some(genre.to_string()),
            duration_secs: Some(duration),
            format: format.to_string(),
            file_size: size,
            modified_at: 1700000000,
            hash: None,
            has_album_art: false,
            bitrate,
        }
    }

    #[test]
    fn test_empty_library_stats() {
        let conn = setup_db();
        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.total_tracks, 0);
        assert_eq!(stats.total_artists, 0);
        assert_eq!(stats.total_albums, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.total_duration_secs, 0.0);
        assert!(stats.avg_bitrate.is_none());
        assert!(stats.formats.is_empty());
        assert!(stats.genres.is_empty());
    }

    #[test]
    fn test_stats_counts_and_totals() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("ArtistA", "Album1", "flac", "Rock", 50_000_000, 300.0, Some(1411), "t1")).unwrap();
        upsert_track(&conn, &make_track("ArtistA", "Album1", "flac", "Rock", 48_000_000, 280.0, Some(1411), "t2")).unwrap();
        upsert_track(&conn, &make_track("ArtistB", "Album2", "mp3", "Jazz", 8_000_000, 240.0, Some(320), "t3")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.total_tracks, 3);
        assert_eq!(stats.total_artists, 2);
        assert_eq!(stats.total_albums, 2);
        assert_eq!(stats.total_size, 106_000_000);
        assert!((stats.total_duration_secs - 820.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_format_breakdown() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t3")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.formats.len(), 2);
        assert_eq!(stats.formats[0].format, "flac");
        assert_eq!(stats.formats[0].count, 2);
        assert_eq!(stats.formats[1].format, "mp3");
        assert_eq!(stats.formats[1].count, 1);
    }

    #[test]
    fn test_stats_genre_breakdown() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Rock", 8_000_000, 240.0, None, "t2")).unwrap();
        upsert_track(&conn, &make_track("C", "C1", "flac", "Jazz", 45_000_000, 300.0, None, "t3")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.genres.len(), 2);
        assert_eq!(stats.genres[0].genre, "Rock");
        assert_eq!(stats.genres[0].count, 2);
        assert_eq!(stats.genres[1].genre, "Jazz");
        assert_eq!(stats.genres[1].count, 1);
    }

    #[test]
    fn test_stats_avg_bitrate() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, Some(1411), "t1")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, Some(320), "t2")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        let avg = stats.avg_bitrate.unwrap();
        assert!((avg - 865.5).abs() < 1.0); // (1411 + 320) / 2
    }

    #[test]
    fn test_stats_avg_bitrate_with_nulls() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, Some(1000), "t1")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t2")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        // AVG ignores NULLs in SQLite, so only the 1000 is counted
        let avg = stats.avg_bitrate.unwrap();
        assert!((avg - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_stats_scoped_to_library_root() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &{
            let mut t = make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t2");
            t.file_path = "/other/B/B1/t2.mp3".to_string();
            t.relative_path = "B/B1/t2.mp3".to_string();
            t.library_root = "/other".to_string();
            t
        }).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.total_tracks, 1);
        assert_eq!(stats.total_artists, 1);
        assert_eq!(stats.formats.len(), 1);
    }
}
```

### Frontend verification

No dedicated frontend unit tests. The frontend is verified via:
- `npm run check` (Svelte + TypeScript type checking) ensures type correctness
- Manual verification that the Statistics page renders correctly

---

## Implementation Steps (ordered)

1. **Add `bitrate` field to `Track`** (`src-tauri/src/models/track.rs`) -- add `pub bitrate: Option<u32>` after `has_album_art`

2. **Add `bitrate` column migration** (`src-tauri/src/db/schema.rs`) -- same pattern as `has_album_art` migration

3. **Extract bitrate in scanner** (`src-tauri/src/scanner/metadata.rs`) -- read `properties.overall_bitrate()`, include in returned Track

4. **Update `library_repo.rs`** -- add `bitrate` to `upsert_track` (INSERT/UPDATE + params), and to SELECT + row mapping in `get_library_tree`, `search_tracks`, `get_tracks_by_artists`, `get_tracks_by_albums`, `get_tracks_for_device`, `get_incomplete_tracks`. Column index 18 in all row mappings.

5. **Add stats model structs** (`src-tauri/src/models/track.rs`) -- `FormatStat`, `GenreStat`, `LibraryStats`

6. **Add `get_library_stats` repo function** (`src-tauri/src/db/library_repo.rs`)

7. **Add `get_library_stats` Tauri command** (`src-tauri/src/commands/library.rs`) -- add import for `LibraryStats`

8. **Register command** (`src-tauri/src/lib.rs`) -- add to `generate_handler![]`

9. **Add TypeScript types** (`src/lib/api/types.ts`) -- `FormatStat`, `GenreStat`, `LibraryStats`

10. **Add command wrapper** (`src/lib/api/commands.ts`) -- `getLibraryStats()`, add `LibraryStats` to import

11. **Create `Statistics.svelte`** (`src/pages/Statistics.svelte`) -- summary cards, format breakdown, genre distribution

12. **Wire into router** (`src/App.svelte`) -- add to Page type, import, nav item, render case

13. **Add Rust tests** in `library_repo.rs` -- the `stats_tests` module above

14. **Verify** -- run `cargo test` from `src-tauri/` and `npm run check` from project root
