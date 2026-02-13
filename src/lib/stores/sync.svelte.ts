import type { DiffResult, Conflict, ConflictResolution, ProgressEvent } from "../api/types";
import * as commands from "../api/commands";

export type SyncPhase = "idle" | "computing_diff" | "previewing" | "syncing" | "complete" | "error";

class SyncStore {
  phase = $state<SyncPhase>("idle");
  diffResult = $state<DiffResult | null>(null);
  conflicts = $state<Conflict[]>([]);
  resolutions = $state<ConflictResolution[]>([]);
  progress = $state({
    filesCompleted: 0,
    totalFiles: 0,
    bytesCompleted: 0,
    totalBytes: 0,
    currentFile: "",
  });
  error = $state<string | null>(null);
  syncErrors = $state<{ file: string; error: string }[]>([]);
  syncStartedAt = $state<number | null>(null);

  async computeDiff(profileId: string) {
    this.phase = "computing_diff";
    this.error = null;
    this.diffResult = null;
    this.conflicts = [];
    this.resolutions = [];
    this.syncErrors = [];

    try {
      const [diff, conflicts] = await commands.computeDiff(profileId);
      this.diffResult = diff;
      this.conflicts = conflicts;
      this.resolutions = conflicts.map((c) => ({
        relative_path: c.relative_path,
        resolution: "skip" as const,
      }));
      this.phase = "previewing";
    } catch (e) {
      this.error = String(e);
      this.phase = "error";
    }
  }

  setResolution(relativePath: string, resolution: ConflictResolution["resolution"]) {
    this.resolutions = this.resolutions.map((r) =>
      r.relative_path === relativePath ? { ...r, resolution } : r,
    );
  }

  async executeSync(profileId: string) {
    if (!this.diffResult) return;

    this.phase = "syncing";
    this.syncStartedAt = Date.now();
    this.error = null;
    this.syncErrors = [];
    this.progress = {
      filesCompleted: 0,
      totalFiles: 0,
      bytesCompleted: 0,
      totalBytes: 0,
      currentFile: "",
    };

    try {
      await commands.executeSync(
        profileId,
        this.diffResult,
        this.resolutions,
        (event: ProgressEvent) => {
          switch (event.type) {
            case "sync_started":
              this.progress = {
                ...this.progress,
                totalFiles: event.total_files,
                totalBytes: event.total_bytes,
              };
              break;
            case "sync_progress":
              this.progress = {
                filesCompleted: event.files_completed,
                totalFiles: event.total_files,
                bytesCompleted: event.bytes_completed,
                totalBytes: event.total_bytes,
                currentFile: event.current_file,
              };
              break;
            case "sync_complete":
              this.phase = "complete";
              break;
            case "sync_error":
              this.syncErrors = [...this.syncErrors, { file: event.file, error: event.error }];
              break;
          }
        },
      );
      this.phase = "complete";
    } catch (e) {
      this.error = String(e);
      this.phase = "error";
    }
  }

  async cancel() {
    try {
      await commands.cancelSync();
    } catch (e) {
      // ignore
    }
  }

  reset() {
    this.phase = "idle";
    this.diffResult = null;
    this.conflicts = [];
    this.resolutions = [];
    this.error = null;
    this.syncErrors = [];
    this.syncStartedAt = null;
    this.progress = {
      filesCompleted: 0,
      totalFiles: 0,
      bytesCompleted: 0,
      totalBytes: 0,
      currentFile: "",
    };
  }
}

export const syncStore = new SyncStore();
