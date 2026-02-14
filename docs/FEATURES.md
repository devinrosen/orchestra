# Feature Requests

Statuses: `[ ]` not started · `[designed]` plan exists · `[implemented]` code done, awaiting test/merge · `[done]` tested and merged

## Library

- [done] **Rescan library for updates** — Once a library is loaded, there's no way to re-scan it to pick up new/changed/deleted files without restarting the app. Add a "Rescan" button that re-scans the current `libraryRoot` and updates the database incrementally.
- [ ] **Duplicate detection** — Scan the library for duplicate tracks by content hash or metadata similarity, and provide options to review and remove duplicates.
- [done] **Missing/incomplete metadata report** — Flag tracks that are missing key metadata fields (title, artist, album, album art) so users can review and fix them.
- [ ] **Auto-fetch album art** — Automatically look up and download album artwork from online sources (MusicBrainz, Cover Art Archive) for tracks or albums missing art.
- [ ] **File organization/renaming** — Auto-rename and move files into a folder structure based on metadata patterns (e.g. `Artist/Album/01 - Title.flac`) with a preview before applying.
- [done] **Multiple library view modes** — In addition to the current Artist > Album > Track tree, support alternative browse modes: by Album, by Genre, and by Folder. A segmented toggle at the top of the library view switches between modes. Preference persists across restarts. Folder view includes a play button to queue all tracks in a folder.
- [done] **Library statistics** — Dashboard showing format breakdown, genre distribution, total library size, number of artists/albums/tracks, and average bitrate.

## UI / UX

- [done] **Expandable global status bar** — The global progress bar is compact and useful, but clicking it should expand an inline detail panel (not a full page navigation) showing: current file, files completed/total, bytes transferred, elapsed time, and an option to collapse back down.
- [done] **Song and album metadata viewer/editor** — View and edit metadata (title, artist, album artist, album, track number, disc number, year, genre, album art) for individual tracks or in bulk for an album. Changes should write back to the audio files via lofty and update the database.
- [done] **Shared track row component** — Extract the duplicated track row markup (play button, track number, title, duration, format, size) from TreeView, AlbumListView, GenreTreeView, and FolderTreeView into a reusable `TrackRow.svelte` component.
- [ ] **Keyboard shortcuts** — Navigate the library tree, trigger scan/sync, and open editors without using the mouse. Configurable key bindings.

## Playback

- [done] **Play music by song or album** — Add audio playback support so users can play individual tracks or full albums directly from the library view.
- [ ] **Playlist support** — Create, manage, and reorder playlists within the app, with export to M3U/PLS formats for use in other players.

## Device Sync

- [done] **Sync by album** — In addition to selecting artists, allow selecting individual albums to sync to a device. Useful for syncing a few albums from an artist with a large discography without pulling everything.
- [done] **Eject/unmount device button** — Add an eject button to the device card so users can safely unmount a connected device directly from the app without switching to Finder or the system tray.

## CD Ripping & Conversion

- [ ] **Rip CD to library** — Detect an inserted audio CD, read its table of contents, look up track/album metadata (e.g. via MusicBrainz/CDDB), rip tracks to a chosen format (FLAC, MP3, AAC, etc.), tag the output files with metadata and album art, and add them to the library.
- [ ] **Audio format conversion** — Convert existing library tracks between formats (e.g. FLAC to MP3/AAC) with configurable quality settings, preserving metadata tags and album art.
- [ ] **ReplayGain tagging** — Calculate and write ReplayGain tags (track and album gain) for volume normalization across the library.

## Sync Profiles

_(No open requests)_
