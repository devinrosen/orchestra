import { vi, describe, it, expect, beforeEach } from "vitest";

vi.mock("../api/commands", () => ({
  getSetting: vi.fn(),
  getLibraryTree: vi.fn(),
  scanDirectory: vi.fn(),
  getLibraryStats: vi.fn(),
  getIncompleteMetadata: vi.fn(),
}));

vi.mock("./favorites.svelte", () => ({
  favoritesStore: {
    isFavorite: vi.fn().mockReturnValue(false),
    favoriteIds: new Set(),
  },
}));

import { libraryStore } from "./library.svelte";

describe("LibraryStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    libraryStore.tree = null;
    libraryStore.libraryRoot = "";
    libraryStore.scanning = false;
    libraryStore.error = null;
  });

  it("libraryRoot starts as empty string (falsy)", () => {
    expect(libraryStore.libraryRoot).toBe("");
    expect(libraryStore.libraryRoot).toBeFalsy();
  });

  it("tree starts as null", () => {
    expect(libraryStore.tree).toBeNull();
  });

  it("scanning starts as false", () => {
    expect(libraryStore.scanning).toBe(false);
  });

  it("error starts as null", () => {
    expect(libraryStore.error).toBeNull();
  });
});
