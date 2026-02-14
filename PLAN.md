# Plan: Shared Track Row Component

## Overview

Extract the duplicated track row markup (play button, track number, title, duration, format, size) from TreeView, AlbumListView, GenreTreeView, and FolderTreeView into a reusable `TrackRow.svelte` component. Also extract the duplicated `formatDuration` and `formatSize` utility functions into a shared module.

## Current State

### Duplicated track row markup

All four views render an almost-identical block of HTML for each track. Here is the canonical version from **TreeView.svelte** (lines 89-110):

```svelte
{#each album.tracks as track}
  {@const isPlaying = playerStore.currentTrack?.file_path === track.file_path}
  <div class="track-row" class:now-playing={isPlaying}>
    {#if onPlayTrack}
      <button
        class="track-play-btn"
        onclick={(e) => { e.stopPropagation(); onPlayTrack(track, album.tracks); }}
        title="Play track"
      >&#9654;</button>
    {/if}
    <button
      class="track-node"
      onclick={() => onEditTrack?.(track)}
      title="Edit track metadata"
    >
      <span class="track-num">{track.track_number ?? "-"}</span>
      <span class="track-title">{track.title ?? track.relative_path}</span>
      <span class="track-duration">{formatDuration(track.duration_secs)}</span>
      <span class="track-format">{track.format.toUpperCase()}</span>
      <span class="track-size">{formatSize(track.file_size)}</span>
    </button>
  </div>
{/each}
```

**AlbumListView.svelte** (lines 72-93) and **GenreTreeView.svelte** (lines 90-110) are character-for-character identical to this.

**FolderTreeView.svelte** (lines 75-96) has one difference in the title display:
```svelte
<span class="track-title">{track.title ?? track.relative_path.split("/").pop()}</span>
```
Instead of `track.relative_path`, it uses `track.relative_path.split("/").pop()` to show just the filename.

### Duplicated utility functions

`formatDuration` and `formatSize` are identically defined in all four files:

```typescript
function formatDuration(secs: number | null): string {
  if (secs == null) return "--:--";
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
```

### Duplicated CSS

All four files contain identical CSS rules for: `.track-row`, `.track-row.now-playing`, `.track-row.now-playing .track-title`, `.track-play-btn`, `.track-row:hover .track-play-btn`, `.track-play-btn:hover`, `.track-node`, `.track-node:hover`, `.track-num`, `.track-title`, `.track-duration`, `.track-format`, `.track-size` (approximately 65 lines of CSS each).

### Playback integration

Each view imports `playerStore` from `../stores/player.svelte` and uses `playerStore.currentTrack?.file_path === track.file_path` to determine if a track is currently playing. The play button calls an `onPlayTrack` callback (passed as a prop) with `(track, siblingTracks)` -- the sibling tracks array varies by context:
- TreeView/AlbumListView/GenreTreeView: `album.tracks`
- FolderTreeView: `node.tracks`

The edit button calls `onEditTrack?.(track)`.

## Component Design

### New file: `src/lib/components/TrackRow.svelte`

**Props interface:**

```typescript
let {
  track,
  siblingTracks,
  titleFallback,
  onPlay,
  onEdit,
}: {
  track: Track;
  siblingTracks: Track[];
  titleFallback?: string;
  onPlay?: (track: Track, siblingTracks: Track[]) => void;
  onEdit?: (track: Track) => void;
} = $props();
```

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `track` | `Track` | Yes | The track to display |
| `siblingTracks` | `Track[]` | Yes | Sibling tracks passed to play callback (e.g. album tracks or folder tracks) |
| `titleFallback` | `string` | No | Override for the fallback text when `track.title` is null. Defaults to `track.relative_path`. FolderTreeView will pass `track.relative_path.split("/").pop()`. |
| `onPlay` | `(track, siblings) => void` | No | If provided, a play button is shown |
| `onEdit` | `(track) => void` | No | Click handler for the track row body |

**Internal state:**
- Derives `isPlaying` from `playerStore.currentTrack?.file_path === track.file_path`
- Imports `formatDuration` and `formatSize` from `../utils/format`

**Markup:**
The component renders the `<div class="track-row">` block containing the optional play button and the track info button, identical to the current duplicated markup.

**CSS:**
All track-row-related styles move into TrackRow.svelte's `<style>` block. Since Svelte styles are component-scoped, there are no conflicts with parent components.

### New file: `src/lib/utils/format.ts`

Extract `formatDuration` and `formatSize` into a shared utility module. TrackRow.svelte will import them, and any other component that needs them can too.

```typescript
export function formatDuration(secs: number | null): string {
  if (secs == null) return "--:--";
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
```

## Frontend Changes

### TrackRow.svelte (new)

Full component as described in Component Design above.

### TreeView.svelte

1. Add import: `import TrackRow from "./TrackRow.svelte";`
2. Remove `import { playerStore }` (no longer used directly)
3. Remove `formatDuration` and `formatSize` function definitions
4. Replace lines 89-110 (the `{#each album.tracks as track}` block) with:
   ```svelte
   {#each album.tracks as track}
     <TrackRow
       {track}
       siblingTracks={album.tracks}
       onPlay={onPlayTrack}
       onEdit={onEditTrack}
     />
   {/each}
   ```
5. Remove all track-row-related CSS (`.track-row`, `.track-play-btn`, `.track-node`, `.track-num`, `.track-title`, `.track-duration`, `.track-format`, `.track-size` and their variants -- approximately lines 213-292)

### AlbumListView.svelte

1. Add import: `import TrackRow from "./TrackRow.svelte";`
2. Remove `import { playerStore }` (no longer used directly)
3. Remove `formatDuration` and `formatSize` function definitions
4. Replace lines 72-93 with:
   ```svelte
   {#each album.tracks as track}
     <TrackRow
       {track}
       siblingTracks={album.tracks}
       onPlay={onPlayTrack}
       onEdit={onEditTrack}
     />
   {/each}
   ```
5. Remove all track-row-related CSS (approximately lines 193-272)

### GenreTreeView.svelte

1. Add import: `import TrackRow from "./TrackRow.svelte";`
2. Remove `import { playerStore }` (no longer used directly)
3. Remove `formatDuration` and `formatSize` function definitions
4. Replace lines 90-110 with:
   ```svelte
   {#each album.tracks as track}
     <TrackRow
       {track}
       siblingTracks={album.tracks}
       onPlay={onPlayTrack}
       onEdit={onEditTrack}
     />
   {/each}
   ```
5. Remove all track-row-related CSS (approximately lines 219-298)

### FolderTreeView.svelte

1. Add import: `import TrackRow from "./TrackRow.svelte";`
2. Remove `import { playerStore }` (no longer used directly)
3. Remove `formatDuration` and `formatSize` function definitions
4. Replace lines 75-96 with:
   ```svelte
   {#each node.tracks as track}
     <TrackRow
       {track}
       siblingTracks={node.tracks}
       titleFallback={track.title ? undefined : track.relative_path.split("/").pop() ?? track.relative_path}
       onPlay={onPlayTrack}
       onEdit={onEditTrack}
     />
   {/each}
   ```
   Note the `titleFallback` prop is only needed here to preserve the filename-only fallback behavior.
5. Remove all track-row-related CSS (approximately lines 191-270)

## Dead Code

After the refactor, the following become unused and should be removed from each parent view:

| Item | Files | Reason |
|------|-------|--------|
| `import { playerStore }` | All 4 views | `isPlaying` check moves into TrackRow |
| `formatDuration()` | All 4 views | Moved to `src/lib/utils/format.ts` |
| `formatSize()` | All 4 views | Moved to `src/lib/utils/format.ts` |
| `.track-row` CSS (and all variants) | All 4 views | Moved into TrackRow.svelte |
| `.track-play-btn` CSS (and all variants) | All 4 views | Moved into TrackRow.svelte |
| `.track-node` CSS (and all variants) | All 4 views | Moved into TrackRow.svelte |
| `.track-num` CSS | All 4 views | Moved into TrackRow.svelte |
| `.track-title` CSS | All 4 views | Moved into TrackRow.svelte |
| `.track-duration` CSS | All 4 views | Moved into TrackRow.svelte |
| `.track-format` CSS | All 4 views | Moved into TrackRow.svelte |
| `.track-size` CSS | All 4 views | Moved into TrackRow.svelte |

No existing components or modules become entirely unused.

## Test Cases

1. **Type checking**: Run `npm run check` -- all four refactored views and the new TrackRow.svelte must pass TypeScript/Svelte type checking with zero errors.
2. **Visual regression** (manual): Open the library page in each view mode (Artist, Album, Genre, Folder) and verify:
   - Track rows display identically to before the refactor
   - Track number, title, duration, format badge, and size all render correctly
   - The "now playing" highlight (accent color background) still appears on the currently playing track
3. **Play button**: Click a track's play button in each view mode. Confirm playback starts and the play button only appears on hover.
4. **Edit button**: Click a track row body in each view mode. Confirm the metadata editor opens for that track.
5. **Folder view title fallback**: In Folder view, verify that tracks without a `title` show just the filename (not the full relative path).
6. **Rust tests**: Run `cargo test` from `src-tauri/` -- backend is unaffected but confirms no regressions.

## Implementation Steps

1. Create `src/lib/utils/format.ts` with `formatDuration` and `formatSize` exports.
2. Create `src/lib/components/TrackRow.svelte` with the props interface, markup, and styles as specified above. Import `formatDuration`/`formatSize` from `../utils/format` and `playerStore` from `../stores/player.svelte`.
3. Refactor **TreeView.svelte**: import TrackRow, replace track row block, remove dead code (playerStore import, format functions, track CSS).
4. Refactor **AlbumListView.svelte**: same as step 3.
5. Refactor **GenreTreeView.svelte**: same as step 3.
6. Refactor **FolderTreeView.svelte**: same as step 3, plus pass `titleFallback` prop.
7. Run `npm run check` and fix any type errors.
8. Run `cargo test` from `src-tauri/` to confirm backend is unaffected.
9. Update `docs/FEATURES.md` to mark the feature as `[implemented]`.
