# Plan: Expandable Global Status Bar

## Summary

The existing `GlobalStatusBar.svelte` shows a compact row of progress indicators (library scan, device sync, profile sync) at the top of the main content area. Clicking a status item currently navigates to the corresponding page. This feature changes that behavior: clicking a status item will instead expand an inline detail panel directly below the compact bar, showing richer progress information. A second click (or a collapse button) collapses it back down. Navigation to the relevant page is moved to a secondary action within the expanded panel.

## Current State Analysis

### Existing GlobalStatusBar (`src/lib/components/GlobalStatusBar.svelte`)
- Renders conditionally when any operation is active (`visible` derived)
- Shows up to 3 status items side-by-side: scan, device sync, profile sync
- Each item is a `<button>` that calls `onNavigate()` to switch pages
- Each item shows a text label + thin 4px progress track (determinate or indeterminate)
- Positioned at the top of `.main-column` in `App.svelte` (line 49), above `<main class="content">`

### Data Available in Stores
- **Library scan** (`libraryStore`): `filesFound`, `filesProcessed`, `currentFile`, `dirsTotal`, `dirsCompleted`, `scanning`
- **Device sync** (`deviceStore`): `diffProgress` (phase, filesFound, filesCompared, totalFiles, currentFile), `syncProgress` (filesCompleted, totalFiles, bytesCompleted, totalBytes, currentFile), `syncPhase`, `selectedDevice`
- **Profile sync** (`syncStore`): `progress` (filesCompleted, totalFiles, bytesCompleted, totalBytes, currentFile), `phase`

### What's Missing for the Feature
- No elapsed time tracking exists in any store
- No expanded/collapsed state
- No detail panel UI

## Implementation Plan

### 1. Add Elapsed Time Tracking to Stores

**File: `src/lib/stores/sync.svelte.ts`**
- Add `syncStartedAt = $state<number | null>(null)` field
- Set `this.syncStartedAt = Date.now()` in `executeSync()` when phase becomes "syncing"
- Reset to `null` in `reset()`

**File: `src/lib/stores/device.svelte.ts`**
- Add `syncStartedAt = $state<number | null>(null)` field
- Set `this.syncStartedAt = Date.now()` in `executeSync()` when phase becomes "syncing"
- Also set in `computeDiff()` for the diff phase timing
- Reset to `null` in `resetSync()`

**File: `src/lib/stores/library.svelte.ts`**
- Add `scanStartedAt = $state<number | null>(null)` field
- Set `this.scanStartedAt = Date.now()` in `scan()` before the try block
- Reset to `null` in the `finally` block (after scan completes)

### 2. Add Expanded State to GlobalStatusBar

**File: `src/lib/components/GlobalStatusBar.svelte`**

Add local component state:
```ts
type ExpandedSection = "scan" | "device" | "profile" | null;
let expandedSection = $state<ExpandedSection>(null);
```

- Clicking a status item toggles `expandedSection` (same item = collapse, different item = switch)
- Auto-collapse when the operation completes (use `$effect` watching `visible` or individual active states)
- When `visible` becomes false, reset `expandedSection` to null

### 3. Modify Compact Bar Click Behavior

**File: `src/lib/components/GlobalStatusBar.svelte`**

Change the `onclick` handlers on each `.status-item` button:
- Instead of `() => onNavigate("library")`, use `() => toggleExpand("scan")`
- Instead of `() => onNavigate("devices")`, use `() => toggleExpand("device")`
- Instead of `() => onNavigate("sync-preview")`, use `() => toggleExpand("profile")`

Add a visual indicator (chevron) that rotates when expanded.

### 4. Create the Expanded Detail Panel

**File: `src/lib/components/GlobalStatusBar.svelte`** (inline, not a separate component)

Render the detail panel conditionally below the compact bar, inside the same component. The panel shows different content depending on `expandedSection`:

#### Scan Detail Panel (`expandedSection === "scan"`)
- Current file being processed (full path, truncated with ellipsis)
- Files processed / files found
- Directories completed / directories total
- Elapsed time (computed from `libraryStore.scanStartedAt`)
- "Go to Library" link button
- Collapse button (chevron up or X)

#### Device Sync Detail Panel (`expandedSection === "device"`)
- Device name
- Phase indicator (scanning / comparing / syncing)
- Current file
- If diffing: files compared / total files
- If syncing: files completed / total files, bytes transferred / total bytes
- Elapsed time
- "Go to Devices" link button
- Collapse button

#### Profile Sync Detail Panel (`expandedSection === "profile"`)
- Current file
- Files completed / total files
- Bytes transferred / total bytes
- Elapsed time
- "Go to Sync Preview" link button
- Collapse button

#### Elapsed Time Display
Add a helper function in the `<script>` block:
```ts
function formatElapsed(startedAt: number | null): string {
  if (!startedAt) return "0:00";
  const seconds = Math.floor((now - startedAt) / 1000);
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}
```

Use a reactive tick to update the elapsed time display. Add a 1-second interval that triggers re-render:
```ts
let now = $state(Date.now());
let intervalId: ReturnType<typeof setInterval> | undefined;

$effect(() => {
  if (expandedSection) {
    intervalId = setInterval(() => { now = Date.now(); }, 1000);
  } else {
    clearInterval(intervalId);
  }
  return () => clearInterval(intervalId);
});
```

Then compute elapsed time reactively using `now` instead of `Date.now()`.

#### Bytes Formatting
Add a local `formatSize` helper inside GlobalStatusBar (same pattern as SyncPreview):
```ts
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
```

### 5. Styling

All styles remain scoped inside `GlobalStatusBar.svelte`.

#### Compact Bar Changes
- Add a subtle visual indicator (chevron via Unicode `\u25BC` / `\u25B2`) on each item when hovered or expanded
- Active/expanded item gets a slightly different background (`var(--bg-tertiary)`)

#### Detail Panel Styles
```css
.detail-panel {
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  padding: 12px 16px;
  font-size: 13px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 24px;
  overflow: hidden;
  animation: expandDown 0.15s ease-out;
}

.detail-panel .current-file {
  grid-column: 1 / -1;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.detail-panel .stat-label {
  color: var(--text-secondary);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-panel .stat-value {
  font-size: 14px;
  font-weight: 500;
}

.detail-actions {
  grid-column: 1 / -1;
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}

@keyframes expandDown {
  from { max-height: 0; opacity: 0; }
  to { max-height: 200px; opacity: 1; }
}
```

### 6. Auto-Collapse Behavior

Add `$effect` blocks:
- When `visible` becomes false, set `expandedSection = null`
- When the specific active operation for the expanded section ends, collapse that section

```ts
$effect(() => {
  if (!visible) expandedSection = null;
});

$effect(() => {
  if (expandedSection === "scan" && !scanActive) expandedSection = null;
  if (expandedSection === "device" && !deviceActive) expandedSection = null;
  if (expandedSection === "profile" && !profileActive) expandedSection = null;
});
```

## Files to Modify

| File | Change |
|------|--------|
| `src/lib/stores/library.svelte.ts` | Add `scanStartedAt` state field |
| `src/lib/stores/sync.svelte.ts` | Add `syncStartedAt` state field |
| `src/lib/stores/device.svelte.ts` | Add `syncStartedAt` state field |
| `src/lib/components/GlobalStatusBar.svelte` | Major changes: expanded state, detail panel, click behavior, styling |

No new files need to be created. No Rust backend changes required. No changes to `App.svelte`.

## Edge Cases and Design Decisions

1. **Multiple simultaneous operations**: The compact bar already shows multiple items side-by-side. Only one can be expanded at a time. Clicking a different item switches the expanded panel.

2. **Operation completes while expanded**: Auto-collapse when the specific operation finishes. The user can still see other active operations in the compact bar.

3. **Navigation still available**: Each expanded panel includes a "Go to [Page]" button, preserving the ability to navigate to the full page. This replaces the previous direct-click-to-navigate behavior.

4. **No scroll interference**: The detail panel sits between the compact bar and `<main>`, so it does not interfere with content scrolling. It simply pushes the content area down slightly.

5. **Elapsed time accuracy**: The 1-second interval only runs while a panel is expanded, avoiding unnecessary work. The `startedAt` timestamps are set when operations begin, so elapsed time is always accurate even if the user expands/collapses multiple times.

6. **Panel height**: The detail panel uses a fixed max-height with overflow hidden. Content is compact enough to never need scrolling within the panel itself.

7. **Keep `onNavigate` prop**: The prop is still needed for the "Go to [Page]" buttons inside the expanded panel. No changes to `App.svelte` are required.
