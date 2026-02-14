export interface Track {
  id: number | null;
  file_path: string;
  relative_path: string;
  library_root: string;
  title: string | null;
  artist: string | null;
  album_artist: string | null;
  album: string | null;
  track_number: number | null;
  disc_number: number | null;
  year: number | null;
  genre: string | null;
  duration_secs: number | null;
  format: string;
  file_size: number;
  modified_at: number;
  hash: string | null;
  has_album_art: boolean;
  bitrate: number | null;
}

export interface AlbumNode {
  name: string;
  year: number | null;
  tracks: Track[];
}

export interface ArtistNode {
  name: string;
  albums: AlbumNode[];
}

export interface LibraryTree {
  root: string;
  artists: ArtistNode[];
  total_tracks: number;
}

export type LibraryViewMode = "artist" | "album" | "genre" | "folder";

export interface AlbumEntry {
  name: string;
  artist: string;
  year: number | null;
  tracks: Track[];
}

export interface GenreNode {
  name: string;
  albums: AlbumEntry[];
}

export interface FolderNode {
  name: string;
  path: string;
  children: FolderNode[];
  tracks: Track[];
}

export interface TrackMetadataUpdate {
  file_path: string;
  title?: string | null;
  artist?: string | null;
  album_artist?: string | null;
  album?: string | null;
  track_number?: number | null;
  disc_number?: number | null;
  year?: number | null;
  genre?: string | null;
}

export interface AlbumArt {
  data: string;
  mime_type: string;
}

export type SyncMode = "one_way" | "two_way";

export interface SyncProfile {
  id: string;
  name: string;
  source_path: string;
  target_path: string;
  sync_mode: SyncMode;
  exclude_patterns: string[];
  created_at: number;
  last_synced_at: number | null;
}

export interface CreateProfileRequest {
  name: string;
  source_path: string;
  target_path: string;
  sync_mode: SyncMode;
  exclude_patterns: string[];
}

export interface UpdateProfileRequest {
  id: string;
  name?: string;
  source_path?: string;
  target_path?: string;
  sync_mode?: SyncMode;
  exclude_patterns?: string[];
}

export type DiffAction = "add" | "remove" | "update" | "unchanged" | "conflict";
export type DiffDirection = "source_to_target" | "target_to_source" | "both";

export interface DiffEntry {
  relative_path: string;
  action: DiffAction;
  direction: DiffDirection;
  source_size: number | null;
  target_size: number | null;
  source_hash: string | null;
  target_hash: string | null;
  source_modified: number | null;
  target_modified: number | null;
}

export interface DiffResult {
  profile_id: string;
  entries: DiffEntry[];
  total_add: number;
  total_remove: number;
  total_update: number;
  total_conflict: number;
  total_unchanged: number;
  bytes_to_transfer: number;
}

export type ConflictType = "both_modified" | "deleted_and_modified" | "first_sync_differs";
export type Resolution = "keep_source" | "keep_target" | "keep_both" | "skip";

export interface Conflict {
  relative_path: string;
  conflict_type: ConflictType;
  source_hash: string | null;
  target_hash: string | null;
  source_modified: number | null;
  target_modified: number | null;
  source_size: number | null;
  target_size: number | null;
}

export interface ConflictResolution {
  relative_path: string;
  resolution: Resolution;
}

export interface DetectedVolume {
  volume_uuid: string;
  volume_name: string;
  mount_path: string;
  capacity_bytes: number;
  free_bytes: number;
  bus_protocol: string;
  already_registered: boolean;
}

export interface Device {
  id: string;
  name: string;
  volume_uuid: string;
  volume_name: string;
  mount_path: string | null;
  capacity_bytes: number | null;
  music_folder: string;
  created_at: number;
  last_synced_at: number | null;
}

export interface AlbumSelection {
  artist_name: string;
  album_name: string;
}

export interface AlbumSummary {
  artist_name: string;
  album_name: string;
  track_count: number;
  total_size: number;
  year: number | null;
}

export interface DeviceWithStatus {
  device: Device;
  connected: boolean;
  selected_artists: string[];
  selected_albums: AlbumSelection[];
}

export interface RegisterDeviceRequest {
  name: string;
  volume_uuid: string;
  volume_name: string;
  mount_path: string;
  capacity_bytes: number | null;
  music_folder: string;
}

export interface ArtistSummary {
  name: string;
  album_count: number;
  track_count: number;
  total_size: number;
}

export type ProgressEvent =
  | { type: "scan_started"; path: string }
  | { type: "scan_progress"; files_found: number; files_processed: number; current_file: string; dirs_total: number; dirs_completed: number }
  | { type: "scan_tree_updated"; new_dirs: number; removed_dirs: number; new_tracks: number }
  | { type: "scan_complete"; total_files: number; duration_ms: number }
  | { type: "device_scan_progress"; files_found: number; current_file: string }
  | { type: "diff_progress"; files_compared: number; total_files: number; current_file: string }
  | { type: "diff_complete"; total_entries: number }
  | { type: "sync_started"; total_files: number; total_bytes: number }
  | { type: "sync_progress"; files_completed: number; total_files: number; bytes_completed: number; total_bytes: number; current_file: string }
  | { type: "sync_complete"; files_synced: number; duration_ms: number }
  | { type: "sync_error"; file: string; error: string };

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

export interface Playlist {
  id: string;
  name: string;
  created_at: number;
  updated_at: number;
}

export interface PlaylistWithTracks {
  playlist: Playlist;
  tracks: Track[];
}

export interface CreatePlaylistRequest {
  name: string;
}

export interface UpdatePlaylistRequest {
  id: string;
  name?: string;
}

export interface AddTracksRequest {
  playlist_id: string;
  track_ids: number[];
}

export interface ReorderTracksRequest {
  playlist_id: string;
  track_ids: number[];
}

export interface RemoveTracksRequest {
  playlist_id: string;
  track_ids: number[];
}

export type DuplicateMatchType = "content_hash" | "metadata_similarity";

export interface DuplicateGroup {
  match_type: DuplicateMatchType;
  match_key: string;
  tracks: Track[];
}

export interface DuplicateResult {
  groups: DuplicateGroup[];
  total_duplicate_tracks: number;
  total_wasted_bytes: number;
}
