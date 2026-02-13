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

export type ProgressEvent =
  | { type: "scan_started"; path: string }
  | { type: "scan_progress"; files_found: number; files_processed: number; current_file: string }
  | { type: "scan_complete"; total_files: number; duration_ms: number }
  | { type: "diff_progress"; files_compared: number; total_files: number }
  | { type: "diff_complete"; total_entries: number }
  | { type: "sync_started"; total_files: number; total_bytes: number }
  | { type: "sync_progress"; files_completed: number; total_files: number; bytes_completed: number; total_bytes: number; current_file: string }
  | { type: "sync_complete"; files_synced: number; duration_ms: number }
  | { type: "sync_error"; file: string; error: string };
