import type { OrganizePreviewResult, OrganizeApplyResult, ProgressEvent } from "../api/types";
import * as commands from "../api/commands";

const DEFAULT_PATTERN = "{artist}/{album}/{track_number} - {title}.{ext}";

class OrganizeStore {
  pattern = $state<string>(DEFAULT_PATTERN);
  previewing = $state(false);
  applying = $state(false);
  preview = $state<OrganizePreviewResult | null>(null);
  progress = $state({ completed: 0, total: 0, currentFile: "" });
  result = $state<OrganizeApplyResult | null>(null);
  error = $state<string | null>(null);

  async init() {
    try {
      const saved = await commands.getSetting("organize_pattern");
      if (saved) {
        this.pattern = saved;
      }
    } catch (_) {
      // non-critical
    }
  }

  async previewOrganize(libraryRoot: string) {
    this.previewing = true;
    this.error = null;
    this.preview = null;
    this.result = null;
    try {
      await commands.setSetting("organize_pattern", this.pattern);
      this.preview = await commands.previewOrganize(libraryRoot, this.pattern);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.previewing = false;
    }
  }

  async applyOrganize(libraryRoot: string, excludedIds?: Set<number>) {
    if (!this.preview) return;

    const itemsToMove = this.preview.items.filter(
      (item) =>
        item.status.type === "Ok" &&
        (!excludedIds || !excludedIds.has(item.track_id)),
    );
    if (itemsToMove.length === 0) return;

    this.applying = true;
    this.error = null;
    this.result = null;
    this.progress = { completed: 0, total: itemsToMove.length, currentFile: "" };

    try {
      const applyItems = itemsToMove.map((item) => ({
        track_id: item.track_id,
        current_file_path: libraryRoot.replace(/\/$/, "") + "/" + item.current_relative_path,
        proposed_relative_path: item.proposed_relative_path,
      }));

      this.result = await commands.applyOrganize(
        libraryRoot,
        applyItems,
        (event: ProgressEvent) => {
          if (event.type === "organize_progress") {
            this.progress = {
              completed: event.completed,
              total: event.total,
              currentFile: event.current_file,
            };
          }
        },
      );
    } catch (e) {
      this.error = String(e);
    } finally {
      this.applying = false;
    }
  }

  resetPreview() {
    this.preview = null;
    this.result = null;
    this.error = null;
    this.progress = { completed: 0, total: 0, currentFile: "" };
  }
}

export const organizeStore = new OrganizeStore();
