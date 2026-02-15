# Feature Requests

Statuses: `[ ]` not started · `[designed]` plan exists · `[implemented]` code done, awaiting test/merge · `[done]` tested and merged

## Library

- [done] **Rescan library for updates** — Once a library is loaded, there's no way to re-scan it to pick up new/changed/deleted files without restarting the app. Add a "Rescan" button that re-scans the current `libraryRoot` and updates the database incrementally.
- [done] **Duplicate detection** — Scan the library for duplicate tracks by content hash or metadata similarity, and provide options to review and remove duplicates.
- [ ] **Duplicate detection progress in GlobalStatusBar** — The BLAKE3 hashing phase of duplicate detection can take a while on large libraries but currently only shows progress inside the modal. Surface it in the GlobalStatusBar (like scan and sync do) so users can navigate away while hashing runs and return to view results when complete.
- [done] **Missing/incomplete metadata report** — Flag tracks that are missing key metadata fields (title, artist, album, album art) so users can review and fix them.
- [done] **Split Library into Browse / Manage tabs** — The Library page currently mixes browsing (search, tree view, playback) with management actions (Metadata Report, Duplicates, Rescan, Open Directory) in one crowded header. Split into two sub-tabs: **Browse** (default) shows just the search bar, view mode toggle, and the tree/list with playback controls. **Manage** collects library maintenance: Open Directory, Rescan, Metadata Report, Duplicate detection, and library info (root path, track count). Future management features (auto-fetch art, file renaming, format conversion) go here too.
- [ ] **Auto-fetch album art** — Automatically look up and download album artwork from online sources (MusicBrainz, Cover Art Archive) for tracks or albums missing art.
- [ ] **File organization/renaming** — Auto-rename and move files into a folder structure based on metadata patterns (e.g. `Artist/Album/01 - Title.flac`) with a preview before applying.
- [done] **Multiple library view modes** — In addition to the current Artist > Album > Track tree, support alternative browse modes: by Album, by Genre, and by Folder. A segmented toggle at the top of the library view switches between modes. Preference persists across restarts. Folder view includes a play button to queue all tracks in a folder.
- [done] **Library statistics** — Dashboard showing format breakdown, genre distribution, total library size, number of artists/albums/tracks, and average bitrate.
- [done] **Contextual library search** — Make the search bar filter contextually based on the active view mode. In Artist view, search matches artists and shows them with their albums still expandable. In Album view, search matches albums with tracks still expandable. In Genre view, search matches genres. In Folder view, search matches folder names. The current search only filters individual tracks — this would filter at the top-level grouping instead, preserving the tree structure beneath matches.
- [ ] **Smart playlists** — Rule-based auto-playlists that update dynamically (e.g., "genre = Jazz AND year > 2000", "added in last 30 days", "bitrate > 256 AND format = FLAC"). Define rules in a builder UI, results are computed from SQLite queries. Combine with ratings/favorites for powerful personal curation.
- [ ] **Import playlists (M3U/PLS)** — Import M3U and PLS playlist files by matching file paths against the library database. Allows users to bring playlists from other players when migrating to the app.
- [ ] **Recently added / recently played** — Quick-access section showing tracks scanned in the last N days and recently played tracks. Requires a `played_at` timestamp (or a separate `play_history` table) updated each time a track starts playing.
- [done] **Favorites** — Heart-toggle to favorite artists, albums, or tracks. A dedicated Favorites page in the left sidebar shows all favorited items in three collapsible sections (Artists, Albums, Tracks). Library browse views get a "Favorites" filter toggle to show only favorited items in the current view mode. Requires a `favorites` table keyed by `(entity_type, entity_id)` to support all three entity types.
- [ ] **Multi-library support** — Support multiple library roots simultaneously. The DB schema already keys tracks by `library_root`, so the main work is UI for managing multiple roots and showing a combined or per-root view in the library browser.

## UI / UX

- [done] **Expandable global status bar** — The global progress bar is compact and useful, but clicking it should expand an inline detail panel (not a full page navigation) showing: current file, files completed/total, bytes transferred, elapsed time, and an option to collapse back down.
- [done] **Song and album metadata viewer/editor** — View and edit metadata (title, artist, album artist, album, track number, disc number, year, genre, album art) for individual tracks or in bulk for an album. Changes should write back to the audio files via lofty and update the database.
- [done] **Shared track row component** — Extract the duplicated track row markup (play button, track number, title, duration, format, size) from TreeView, AlbumListView, GenreTreeView, and FolderTreeView into a reusable `TrackRow.svelte` component.
- [ ] **Keyboard shortcuts** — Navigate the library tree, trigger scan/sync, and open editors without using the mouse. Configurable key bindings.
- [done] **UI skins (light/dark mode)** — Support light and dark color themes with a toggle in Settings. Respect the OS-level appearance preference by default, with an option to override. Implement via CSS custom properties so all components inherit the active theme. Persist the user's choice across restarts.

## Playback

- [done] **Play music by song or album** — Add audio playback support so users can play individual tracks or full albums directly from the library view.
- [done] **Playlist support** — Create, manage, and reorder playlists within the app, with export to M3U/PLS formats for use in other players.
- [done] **Play queue viewer** — Add a button in the player bar to open a panel showing the current play queue (upcoming songs, current track highlighted). Works when playing an album or a playlist. Users can see what's coming next, skip ahead by clicking a track, and reorder or remove items from the queue.
- [done] **Playback visualization** — Real-time audio visualizations that react to the currently playing track. Include multiple modes: waveform, frequency spectrum (bar graph), and a circular/radial visualizer. Rendered via Canvas or WebGL in a toggleable panel above the player bar. Should use the Web Audio API's AnalyserNode to tap into the audio stream without affecting playback.
- [done] **Equalizer** — Graphic EQ using Web Audio API `BiquadFilterNode` bands inserted between the audio source and destination. The AnalyserNode pipeline is already wired up for the visualizer, so adding filter nodes is straightforward. Presets (flat, bass boost, vocal, etc.) plus manual per-band adjustment. Could share panel space with the visualizer.
- [ ] **Crossfade playback** — Smooth audio transitions between consecutive tracks. Use a second `HTMLAudioElement` that fades in as the current one fades out during the last N seconds of a track. Configurable crossfade duration (0–12 seconds) in Settings. Important for live albums and DJ-style listening.
- [ ] **Lyrics display** — Show lyrics for the currently playing track in a toggleable panel. First read embedded lyrics from audio file tags (via lofty), then optionally fetch from an external API. Synced (timed) lyrics scroll automatically with playback; plain lyrics display as static text.
- [ ] **Scrobbling (Last.fm integration)** — Submit play history to Last.fm when a track has been playing for >50% of its duration or >4 minutes. Requires Last.fm API key and user authentication (OAuth). Stores pending scrobbles locally for offline resilience and batch-submits when connectivity returns.

## Device Sync

- [done] **Sync by album** — In addition to selecting artists, allow selecting individual albums to sync to a device. Useful for syncing a few albums from an artist with a large discography without pulling everything.
- [done] **Eject/unmount device button** — Add an eject button to the device card so users can safely unmount a connected device directly from the app without switching to Finder or the system tray.

## CD Ripping & Conversion

- [ ] **Rip CD to library** — Detect an inserted audio CD, read its table of contents, look up track/album metadata (e.g. via MusicBrainz/CDDB), rip tracks to a chosen format (FLAC, MP3, AAC, etc.), tag the output files with metadata and album art, and add them to the library.
- [ ] **Audio format conversion** — Convert existing library tracks between formats (e.g. FLAC to MP3/AAC) with configurable quality settings, preserving metadata tags and album art.
- [ ] **ReplayGain tagging** — Calculate and write ReplayGain tags (track and album gain) for volume normalization across the library.

## Sync Profiles

_(No open requests)_
