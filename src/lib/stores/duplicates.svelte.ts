import type { DuplicateResult, ProgressEvent } from "../api/types";
import * as commands from "../api/commands";

export type DupDetectionPhase = "idle" | "hashing" | "complete" | "error";

class DuplicatesStore {
  phase = $state<DupDetectionPhase>("idle");
  hashProgress = $state({ filesHashed: 0, totalFiles: 0, currentFile: "" });
  startedAt = $state<number | null>(null);
  result = $state<DuplicateResult | null>(null);
  error = $state<string | null>(null);

  async run(root: string): Promise<void> {
    if (this.phase !== "idle") return;

    this.phase = "hashing";
    this.startedAt = Date.now();
    this.error = null;
    this.result = null;
    this.hashProgress = { filesHashed: 0, totalFiles: 0, currentFile: "" };

    try {
      const result = await commands.findDuplicates(root, (event: ProgressEvent) => {
        switch (event.type) {
          case "hash_started":
            this.hashProgress = {
              ...this.hashProgress,
              totalFiles: event.total,
            };
            break;
          case "hash_progress":
            this.hashProgress = {
              filesHashed: event.files_hashed,
              totalFiles: event.total_files,
              currentFile: event.current_file,
            };
            break;
        }
      });
      this.result = result;
      this.phase = "complete";
    } catch (e) {
      this.error = String(e);
      this.phase = "error";
    }
  }

  reset(): void {
    this.phase = "idle";
    this.hashProgress = { filesHashed: 0, totalFiles: 0, currentFile: "" };
    this.startedAt = null;
    this.result = null;
    this.error = null;
  }
}

export const duplicatesStore = new DuplicatesStore();
