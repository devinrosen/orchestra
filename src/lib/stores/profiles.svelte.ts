import type { SyncProfile, CreateProfileRequest, UpdateProfileRequest } from "../api/types";
import * as commands from "../api/commands";

class ProfilesStore {
  profiles = $state<SyncProfile[]>([]);
  selectedProfile = $state<SyncProfile | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async load() {
    this.loading = true;
    this.error = null;
    try {
      this.profiles = await commands.listProfiles();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async create(request: CreateProfileRequest) {
    this.error = null;
    try {
      const profile = await commands.createProfile(request);
      this.profiles = [profile, ...this.profiles];
      return profile;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async update(request: UpdateProfileRequest) {
    this.error = null;
    try {
      const updated = await commands.updateProfile(request);
      this.profiles = this.profiles.map((p) => (p.id === updated.id ? updated : p));
      if (this.selectedProfile?.id === updated.id) {
        this.selectedProfile = updated;
      }
      return updated;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async remove(id: string) {
    this.error = null;
    try {
      await commands.deleteProfile(id);
      this.profiles = this.profiles.filter((p) => p.id !== id);
      if (this.selectedProfile?.id === id) {
        this.selectedProfile = null;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  select(profile: SyncProfile | null) {
    this.selectedProfile = profile;
  }
}

export const profilesStore = new ProfilesStore();
