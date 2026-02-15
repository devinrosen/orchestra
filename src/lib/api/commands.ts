import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  LibraryTree,
  LibraryRoot,
  Track,
  SyncProfile,
  CreateProfileRequest,
  UpdateProfileRequest,
  DiffResult,
  Conflict,
  ConflictResolution,
  ProgressEvent,
  DetectedVolume,
  DeviceWithStatus,
  RegisterDeviceRequest,
  ArtistSummary,
  AlbumSelection,
  AlbumSummary,
  TrackMetadataUpdate,
  AlbumArt,
  LibraryStats,
  DuplicateResult,
  Playlist,
  PlaylistWithTracks,
  CreatePlaylistRequest,
  UpdatePlaylistRequest,
  AddTracksRequest,
  RemoveTracksRequest,
  ReorderTracksRequest,
  Favorite,
} from "./types";

export function scanDirectory(
  path: string,
  onProgress: (event: ProgressEvent) => void,
): Promise<number> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onProgress;
  return invoke("scan_directory", { path, onProgress: channel });
}

export function getLibraryTree(root: string): Promise<LibraryTree> {
  return invoke("get_library_tree", { root });
}

export function searchLibrary(query: string): Promise<Track[]> {
  return invoke("search_library", { query });
}

export function createProfile(request: CreateProfileRequest): Promise<SyncProfile> {
  return invoke("create_profile", { request });
}

export function getProfile(id: string): Promise<SyncProfile> {
  return invoke("get_profile", { id });
}

export function listProfiles(): Promise<SyncProfile[]> {
  return invoke("list_profiles");
}

export function updateProfile(request: UpdateProfileRequest): Promise<SyncProfile> {
  return invoke("update_profile", { request });
}

export function deleteProfile(id: string): Promise<void> {
  return invoke("delete_profile", { id });
}

export function computeDiff(
  profileId: string,
): Promise<[DiffResult, Conflict[]]> {
  return invoke("compute_diff", { profileId });
}

export function executeSync(
  profileId: string,
  diffResult: DiffResult,
  conflictResolutions: ConflictResolution[],
  onProgress: (event: ProgressEvent) => void,
): Promise<number> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onProgress;
  return invoke("execute_sync", {
    profileId,
    diffResult,
    conflictResolutions,
    onProgress: channel,
  });
}

export function cancelSync(): Promise<void> {
  return invoke("cancel_sync");
}

export function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export function getAllSettings(): Promise<[string, string][]> {
  return invoke("get_all_settings");
}

export function detectVolumes(): Promise<DetectedVolume[]> {
  return invoke("detect_volumes");
}

export function registerDevice(request: RegisterDeviceRequest): Promise<DeviceWithStatus> {
  return invoke("register_device", { request });
}

export function listDevices(): Promise<DeviceWithStatus[]> {
  return invoke("list_devices");
}

export function deleteDevice(deviceId: string): Promise<void> {
  return invoke("delete_device", { deviceId });
}

export function setDeviceArtists(deviceId: string, artists: string[]): Promise<void> {
  return invoke("set_device_artists", { deviceId, artists });
}

export function computeDeviceDiff(
  deviceId: string,
  onProgress: (event: ProgressEvent) => void,
): Promise<DiffResult> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onProgress;
  return invoke("compute_device_diff", { deviceId, onProgress: channel });
}

export function executeDeviceSync(
  deviceId: string,
  diffResult: DiffResult,
  onProgress: (event: ProgressEvent) => void,
): Promise<number> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onProgress;
  return invoke("execute_device_sync", {
    deviceId,
    diffResult,
    onProgress: channel,
  });
}

export function ejectDevice(deviceId: string): Promise<void> {
  return invoke("eject_device", { deviceId });
}

export function listArtists(): Promise<ArtistSummary[]> {
  return invoke("list_artists");
}

export function setDeviceAlbums(deviceId: string, albums: AlbumSelection[]): Promise<void> {
  return invoke("set_device_albums", { deviceId, albums });
}

export function listAlbums(): Promise<AlbumSummary[]> {
  return invoke("list_albums");
}

export function getTrackArtwork(filePath: string): Promise<AlbumArt | null> {
  return invoke("get_track_artwork", { filePath });
}

export function updateTrackMetadata(updates: TrackMetadataUpdate[]): Promise<Track[]> {
  return invoke("update_track_metadata", { updates });
}

export function getIncompleteTracks(root: string): Promise<Track[]> {
  return invoke("get_incomplete_tracks", { root });
}

export function getLibraryStats(root: string): Promise<LibraryStats> {
  return invoke("get_library_stats", { root });
}

export function createPlaylist(request: CreatePlaylistRequest): Promise<PlaylistWithTracks> {
  return invoke("create_playlist", { request });
}

export function listPlaylists(): Promise<Playlist[]> {
  return invoke("list_playlists");
}

export function getPlaylist(id: string): Promise<PlaylistWithTracks> {
  return invoke("get_playlist", { id });
}

export function updatePlaylist(request: UpdatePlaylistRequest): Promise<Playlist> {
  return invoke("update_playlist", { request });
}

export function deletePlaylist(id: string): Promise<void> {
  return invoke("delete_playlist", { id });
}

export function addTracksToPlaylist(request: AddTracksRequest): Promise<PlaylistWithTracks> {
  return invoke("add_tracks_to_playlist", { request });
}

export function removeTracksFromPlaylist(request: RemoveTracksRequest): Promise<PlaylistWithTracks> {
  return invoke("remove_tracks_from_playlist", { request });
}

export function reorderPlaylist(request: ReorderTracksRequest): Promise<PlaylistWithTracks> {
  return invoke("reorder_playlist", { request });
}

export function exportPlaylist(id: string, format: string, path: string): Promise<void> {
  return invoke("export_playlist", { id, format, path });
}

export function findDuplicates(
  root: string,
  onProgress: (event: ProgressEvent) => void,
): Promise<DuplicateResult> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onProgress;
  return invoke("find_duplicates", { root, onProgress: channel });
}

export function deleteDuplicateTracks(
  trackIds: number[],
  filePaths: string[],
): Promise<number> {
  return invoke("delete_duplicate_tracks", { trackIds, filePaths });
}

export function toggleFavorite(entityType: string, entityId: string): Promise<boolean> {
  return invoke("toggle_favorite", { entityType, entityId });
}

export function isFavorite(entityType: string, entityId: string): Promise<boolean> {
  return invoke("is_favorite", { entityType, entityId });
}

export function listFavorites(entityType: string): Promise<Favorite[]> {
  return invoke("list_favorites", { entityType });
}

export function listAllFavorites(): Promise<Favorite[]> {
  return invoke("list_all_favorites");
}

export function getFavoriteTracks(): Promise<Track[]> {
  return invoke("get_favorite_tracks");
}

export function addLibraryRoot(path: string, label?: string): Promise<void> {
  return invoke("add_library_root", { path, label: label ?? null });
}

export function removeLibraryRoot(path: string): Promise<void> {
  return invoke("remove_library_root", { path });
}

export function listLibraryRoots(): Promise<LibraryRoot[]> {
  return invoke("list_library_roots");
}
