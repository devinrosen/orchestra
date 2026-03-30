import { vi, describe, it, expect, beforeEach } from "vitest";
import type { SyncProfile } from "../api/types";

vi.mock("../api/commands", () => ({
  listProfiles: vi.fn(),
  createProfile: vi.fn(),
  updateProfile: vi.fn(),
  deleteProfile: vi.fn(),
}));

import * as commands from "../api/commands";
import { profilesStore } from "./profiles.svelte";

const mockProfile: SyncProfile = {
  id: "test-id-1",
  name: "Test Profile",
  source_path: "/source",
  target_path: "/target",
  sync_mode: "one_way",
  exclude_patterns: [],
  created_at: 1700000000,
  last_synced_at: null,
};

describe("ProfilesStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    profilesStore.profiles = [];
    profilesStore.error = null;
    profilesStore.loading = false;
    profilesStore.selectedProfile = null;
  });

  it("starts with empty profiles and no error", () => {
    expect(profilesStore.profiles).toEqual([]);
    expect(profilesStore.error).toBeNull();
    expect(profilesStore.loading).toBe(false);
  });

  it("create appends to profiles on success", async () => {
    vi.mocked(commands.createProfile).mockResolvedValue(mockProfile);

    const result = await profilesStore.create({
      name: "Test Profile",
      source_path: "/source",
      target_path: "/target",
      sync_mode: "one_way",
      exclude_patterns: [],
    });

    expect(result).toEqual(mockProfile);
    expect(profilesStore.profiles).toHaveLength(1);
    expect(profilesStore.profiles[0]).toEqual(mockProfile);
    expect(profilesStore.error).toBeNull();
  });

  it("create sets error on command failure", async () => {
    vi.mocked(commands.createProfile).mockRejectedValue(
      new Error("Database error"),
    );

    const result = await profilesStore.create({
      name: "Fail",
      source_path: "/src",
      target_path: "/tgt",
      sync_mode: "one_way",
      exclude_patterns: [],
    });

    expect(result).toBeNull();
    expect(profilesStore.error).toBeTruthy();
  });

  it("remove deletes profile from list", async () => {
    profilesStore.profiles = [mockProfile];
    vi.mocked(commands.deleteProfile).mockResolvedValue(undefined);

    await profilesStore.remove("test-id-1");

    expect(profilesStore.profiles).toHaveLength(0);
    expect(profilesStore.error).toBeNull();
  });
});
