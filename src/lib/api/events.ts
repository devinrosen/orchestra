import type { ProgressEvent } from "./types";

export type ProgressCallback = (event: ProgressEvent) => void;

export function createProgressHandler(callbacks: {
  onScanStarted?: (path: string) => void;
  onScanProgress?: (filesFound: number, filesProcessed: number, currentFile: string) => void;
  onScanComplete?: (totalFiles: number, durationMs: number) => void;
  onSyncStarted?: (totalFiles: number, totalBytes: number) => void;
  onSyncProgress?: (filesCompleted: number, totalFiles: number, bytesCompleted: number, totalBytes: number, currentFile: string) => void;
  onSyncComplete?: (filesSynced: number, durationMs: number) => void;
  onSyncError?: (file: string, error: string) => void;
}): ProgressCallback {
  return (event: ProgressEvent) => {
    switch (event.type) {
      case "scan_started":
        callbacks.onScanStarted?.(event.path);
        break;
      case "scan_progress":
        callbacks.onScanProgress?.(event.files_found, event.files_processed, event.current_file);
        break;
      case "scan_complete":
        callbacks.onScanComplete?.(event.total_files, event.duration_ms);
        break;
      case "sync_started":
        callbacks.onSyncStarted?.(event.total_files, event.total_bytes);
        break;
      case "sync_progress":
        callbacks.onSyncProgress?.(event.files_completed, event.total_files, event.bytes_completed, event.total_bytes, event.current_file);
        break;
      case "sync_complete":
        callbacks.onSyncComplete?.(event.files_synced, event.duration_ms);
        break;
      case "sync_error":
        callbacks.onSyncError?.(event.file, event.error);
        break;
    }
  };
}
