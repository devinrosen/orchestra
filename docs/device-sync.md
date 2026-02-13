# Device Sync

Sync selected artists from your library to a DAP (Digital Audio Player) that mounts as a USB volume.

## How It Works

Device sync is **one-way** (library is authoritative) and **artist-filtered**. It's separate from the directory-based sync profiles.

### Setup

1. Connect your DAP via USB — it mounts as a volume under `/Volumes/`
2. Navigate to the **Devices** page and click **Detect Devices**
3. Register the device (identified by its macOS Volume UUID for stable recognition across reconnections)
4. Configure which artists to sync via the **Artist Picker**

### Sync Flow: Diff -> Preview -> Execute

**Diff phase** scans the device and compares it against your selected library tracks:

1. **Device scan** — walks the device filesystem, skipping hidden/system directories (`.Spotlight-V100`, `.Trashes`, etc.). Progress shows files found in real time.
2. **Comparison** — for each file, determines the action:
   - **Library-only** -> Add (no hashing needed)
   - **Device-only** -> Remove (artist was deselected or track deleted from library)
   - **Both exist, same size + mtime** -> Unchanged (no hashing needed)
   - **Both exist, size or mtime differ** -> Hash both copies with BLAKE3 to confirm whether content actually changed. If hashes match, it's unchanged; if they differ, it's an Update.

**Preview** shows a summary of adds/removes/updates with total bytes to transfer.

**Execute** copies files to the device using a safe copy-then-rename pattern, with progress reporting and cancellation support.

### Hashing Strategy

Hashing is **lazy** — it only happens when size or mtime differ between the library and device copies.

- The library-side hash may already be cached in the `tracks` table (from a previous diff). If not, the local SSD copy is hashed on demand.
- **Device-side hashes are cached** in the `device_file_cache` table. When a device file needs hashing, the cache is checked first — if the file's size and mtime match the cached entry, the stored hash is reused, skipping the slow USB read entirely.
- The cache is updated at two points:
  - **After diff** — any new hashes computed during comparison are saved immediately, so even if you don't execute the sync, re-running the diff won't re-hash the same files.
  - **After sync** — copied files get the source hash recorded with their new on-device size/mtime; removed files are deleted from the cache.
- On first sync (everything is new), there is **zero hashing** — all files are `Add` entries.
- On subsequent syncs with no changes, size+mtime comparison short-circuits and there is also **zero hashing**.
- On re-diffs after a successful sync, the cache covers all device files, so hashing is skipped unless a file was modified outside the app.

### File Safety

- Files are written using **copy-then-rename**: data goes to a `.tmp_sync` file, fsync'd, then atomically renamed. Source modification time is preserved.
- Before each file operation, the sync checks that the device is still mounted. If disconnected mid-sync, it stops with a `DeviceDisconnected` error.
- Cancellation is supported at any point via an `AtomicBool` cancel token.
- Empty parent directories are cleaned up after file removals.

### Device Identification

Devices are identified by their **macOS Volume UUID**, retrieved via `diskutil info -plist`. This is stable across reconnections — even if the mount path changes, the app recognizes the same device.

### Database Tables

- `devices` — registered devices with name, volume UUID, mount path, capacity, music folder path
- `device_artist_selections` — which artists are selected for each device (many-to-many)
- `device_file_cache` — cached BLAKE3 hashes for files on each device, keyed by `(device_id, relative_path)`, with `file_size` and `modified_at` used to validate cache freshness. Cascades on device deletion.

### Folder Structure

The device uses the same `Artist/Album/Track` folder structure as the library. The `music_folder` setting on each device specifies a relative subfolder on the device to use as the sync root (empty = volume root).
