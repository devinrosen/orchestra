import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  LibraryTree,
  Track,
  SyncProfile,
  CreateProfileRequest,
  UpdateProfileRequest,
  DiffResult,
  Conflict,
  ConflictResolution,
  ProgressEvent,
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
