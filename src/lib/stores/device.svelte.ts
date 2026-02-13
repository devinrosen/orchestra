import type {
  DetectedVolume,
  DeviceWithStatus,
  RegisterDeviceRequest,
  ArtistSummary,
  DiffResult,
  ProgressEvent,
} from "../api/types";
import * as commands from "../api/commands";

export type DeviceSyncPhase =
  | "idle"
  | "computing_diff"
  | "previewing"
  | "syncing"
  | "complete"
  | "error";

class DeviceStore {
  devices = $state<DeviceWithStatus[]>([]);
  detectedVolumes = $state<DetectedVolume[]>([]);
  selectedDeviceId = $state<string | null>(null);
  availableArtists = $state<ArtistSummary[]>([]);
  syncPhase = $state<DeviceSyncPhase>("idle");
  diffResult = $state<DiffResult | null>(null);
  diffProgress = $state({
    phase: "scanning" as "scanning" | "comparing",
    filesFound: 0,
    filesCompared: 0,
    totalFiles: 0,
    currentFile: "",
  });
  syncProgress = $state({
    filesCompleted: 0,
    totalFiles: 0,
    bytesCompleted: 0,
    totalBytes: 0,
    currentFile: "",
  });
  syncErrors = $state<{ file: string; error: string }[]>([]);
  startedAt = $state<number | null>(null);
  error = $state<string | null>(null);
  detecting = $state(false);
  loadingArtists = $state(false);

  selectedDevice = $derived(
    this.devices.find((d) => d.device.id === this.selectedDeviceId) ?? null,
  );

  async loadDevices() {
    try {
      this.devices = await commands.listDevices();
    } catch (e) {
      this.error = String(e);
    }
  }

  async detectVolumes() {
    this.detecting = true;
    this.error = null;
    try {
      this.detectedVolumes = await commands.detectVolumes();
      // Refresh device list — detection may have updated mount paths
      await this.loadDevices();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.detecting = false;
    }
  }

  async registerDevice(request: RegisterDeviceRequest) {
    this.error = null;
    try {
      const device = await commands.registerDevice(request);
      this.devices = [...this.devices, device];
      // Remove from detected since it's now registered
      this.detectedVolumes = this.detectedVolumes.map((v) =>
        v.volume_uuid === request.volume_uuid
          ? { ...v, already_registered: true }
          : v,
      );
      return device;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async deleteDevice(deviceId: string) {
    this.error = null;
    try {
      await commands.deleteDevice(deviceId);
      this.devices = this.devices.filter((d) => d.device.id !== deviceId);
      if (this.selectedDeviceId === deviceId) {
        this.selectedDeviceId = null;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  async loadArtists() {
    this.loadingArtists = true;
    try {
      this.availableArtists = await commands.listArtists();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loadingArtists = false;
    }
  }

  async setArtists(deviceId: string, artists: string[]) {
    this.error = null;
    try {
      await commands.setDeviceArtists(deviceId, artists);
      this.devices = this.devices.map((d) =>
        d.device.id === deviceId
          ? { ...d, selected_artists: artists }
          : d,
      );
    } catch (e) {
      this.error = String(e);
    }
  }

  async computeDiff(deviceId: string) {
    if (this.syncPhase === "computing_diff" || this.syncPhase === "syncing") return;
    this.syncPhase = "computing_diff";
    this.startedAt = Date.now();
    this.error = null;
    this.diffResult = null;
    this.syncErrors = [];
    this.diffProgress = {
      phase: "scanning",
      filesFound: 0,
      filesCompared: 0,
      totalFiles: 0,
      currentFile: "",
    };
    try {
      this.diffResult = await commands.computeDeviceDiff(
        deviceId,
        (event: ProgressEvent) => {
          switch (event.type) {
            case "device_scan_progress":
              this.diffProgress = {
                ...this.diffProgress,
                phase: "scanning",
                filesFound: event.files_found,
                currentFile: event.current_file,
              };
              break;
            case "diff_progress":
              this.diffProgress = {
                ...this.diffProgress,
                phase: "comparing",
                filesCompared: event.files_compared,
                totalFiles: event.total_files,
                currentFile: event.current_file,
              };
              break;
          }
        },
      );
      this.syncPhase = "previewing";
    } catch (e) {
      this.error = String(e);
      this.syncPhase = "error";
    }
  }

  async executeSync(deviceId: string) {
    if (!this.diffResult) return;
    if (this.syncPhase === "syncing") return;
    this.syncPhase = "syncing";
    this.startedAt = Date.now();
    this.error = null;
    this.syncErrors = [];
    this.syncProgress = {
      filesCompleted: 0,
      totalFiles: 0,
      bytesCompleted: 0,
      totalBytes: 0,
      currentFile: "",
    };

    try {
      await commands.executeDeviceSync(
        deviceId,
        this.diffResult,
        (event: ProgressEvent) => {
          switch (event.type) {
            case "sync_started":
              this.syncProgress = {
                ...this.syncProgress,
                totalFiles: event.total_files,
                totalBytes: event.total_bytes,
              };
              break;
            case "sync_progress":
              this.syncProgress = {
                filesCompleted: event.files_completed,
                totalFiles: event.total_files,
                bytesCompleted: event.bytes_completed,
                totalBytes: event.total_bytes,
                currentFile: event.current_file,
              };
              break;
            case "sync_complete":
              this.syncPhase = "complete";
              break;
            case "sync_error":
              this.syncErrors = [
                ...this.syncErrors,
                { file: event.file, error: event.error },
              ];
              break;
          }
        },
      );
      this.syncPhase = "complete";
      // Refresh devices to get updated last_synced_at
      await this.loadDevices();
    } catch (e) {
      this.error = String(e);
      this.syncPhase = "error";
    }
  }

  async cancelSync() {
    try {
      await commands.cancelSync();
    } catch {
      // ignore
    }
  }

  resetSync() {
    this.syncPhase = "idle";
    this.diffResult = null;
    this.syncErrors = [];
    this.startedAt = null;
    this.error = null;
    this.diffProgress = {
      phase: "scanning",
      filesFound: 0,
      filesCompared: 0,
      totalFiles: 0,
      currentFile: "",
    };
    this.syncProgress = {
      filesCompleted: 0,
      totalFiles: 0,
      bytesCompleted: 0,
      totalBytes: 0,
      currentFile: "",
    };
  }

  selectDevice(deviceId: string | null) {
    this.selectedDeviceId = deviceId;
    // Don't reset sync state if an operation is in progress
    if (
      this.syncPhase !== "computing_diff" &&
      this.syncPhase !== "syncing" &&
      this.syncPhase !== "previewing"
    ) {
      this.resetSync();
    }
  }
}

export const deviceStore = new DeviceStore();
