/**
 * Tauri IPC mock for Playwright.
 *
 * Injected via page.addInitScript() before the app loads.
 * Sets up window.__TAURI_INTERNALS__ with a mock invoke() handler
 * that returns realistic data for all commands called during page load.
 */

export const tauriMockScript = /* js */ `
(function() {
  // ── Mock data ──────────────────────────────────────────────────────────

  const LIBRARY_ROOT = "/Users/demo/Music";

  function makeTrack(id, artist, album, title, trackNum, opts = {}) {
    return {
      id,
      file_path: LIBRARY_ROOT + "/" + artist + "/" + album + "/" + String(trackNum).padStart(2, "0") + " " + title + "." + (opts.format || "flac").toLowerCase(),
      relative_path: artist + "/" + album + "/" + String(trackNum).padStart(2, "0") + " " + title + "." + (opts.format || "flac").toLowerCase(),
      library_root: LIBRARY_ROOT,
      title,
      artist,
      album_artist: artist,
      album,
      track_number: trackNum,
      disc_number: 1,
      year: opts.year || 2020,
      genre: opts.genre || "Rock",
      duration_secs: opts.duration || 240,
      format: (opts.format || "FLAC").toUpperCase(),
      file_size: opts.size || 35_000_000,
      modified_at: 1700000000,
      hash: null,
      has_album_art: true,
      bitrate: opts.bitrate || 900,
    };
  }

  const tracks = [
    // Artist 1: Pink Floyd
    makeTrack(1,  "Pink Floyd", "The Dark Side of the Moon", "Speak to Me",        1, { year: 1973, genre: "Progressive Rock", duration: 68 }),
    makeTrack(2,  "Pink Floyd", "The Dark Side of the Moon", "Breathe",            2, { year: 1973, genre: "Progressive Rock", duration: 169 }),
    makeTrack(3,  "Pink Floyd", "The Dark Side of the Moon", "Time",               3, { year: 1973, genre: "Progressive Rock", duration: 413 }),
    makeTrack(4,  "Pink Floyd", "The Dark Side of the Moon", "Money",              4, { year: 1973, genre: "Progressive Rock", duration: 382 }),
    makeTrack(5,  "Pink Floyd", "The Dark Side of the Moon", "Us and Them",        5, { year: 1973, genre: "Progressive Rock", duration: 469 }),
    makeTrack(6,  "Pink Floyd", "Wish You Were Here",        "Shine On You Crazy Diamond (Parts I-V)", 1, { year: 1975, genre: "Progressive Rock", duration: 810 }),
    makeTrack(7,  "Pink Floyd", "Wish You Were Here",        "Welcome to the Machine",                 2, { year: 1975, genre: "Progressive Rock", duration: 450 }),
    makeTrack(8,  "Pink Floyd", "Wish You Were Here",        "Wish You Were Here",                     3, { year: 1975, genre: "Progressive Rock", duration: 334 }),
    // Artist 2: Radiohead
    makeTrack(9,  "Radiohead", "OK Computer",     "Airbag",           1, { year: 1997, genre: "Alternative Rock", duration: 284 }),
    makeTrack(10, "Radiohead", "OK Computer",     "Paranoid Android", 2, { year: 1997, genre: "Alternative Rock", duration: 386 }),
    makeTrack(11, "Radiohead", "OK Computer",     "Karma Police",     3, { year: 1997, genre: "Alternative Rock", duration: 264 }),
    makeTrack(12, "Radiohead", "OK Computer",     "No Surprises",     4, { year: 1997, genre: "Alternative Rock", duration: 229 }),
    makeTrack(13, "Radiohead", "In Rainbows",     "15 Step",          1, { year: 2007, genre: "Alternative Rock", duration: 237 }),
    makeTrack(14, "Radiohead", "In Rainbows",     "Reckoner",         2, { year: 2007, genre: "Alternative Rock", duration: 290 }),
    makeTrack(15, "Radiohead", "In Rainbows",     "House of Cards",   3, { year: 2007, genre: "Alternative Rock", duration: 326 }),
    // Artist 3: Daft Punk
    makeTrack(16, "Daft Punk", "Discovery",               "One More Time",           1, { year: 2001, genre: "Electronic", format: "MP3", duration: 320, bitrate: 320, size: 9_500_000 }),
    makeTrack(17, "Daft Punk", "Discovery",               "Aerodynamic",             2, { year: 2001, genre: "Electronic", format: "MP3", duration: 212, bitrate: 320, size: 6_300_000 }),
    makeTrack(18, "Daft Punk", "Discovery",               "Harder Better Faster Stronger", 3, { year: 2001, genre: "Electronic", format: "MP3", duration: 231, bitrate: 320, size: 6_900_000 }),
    makeTrack(19, "Daft Punk", "Random Access Memories",  "Get Lucky",               1, { year: 2013, genre: "Electronic", duration: 369 }),
    makeTrack(20, "Daft Punk", "Random Access Memories",  "Instant Crush",           2, { year: 2013, genre: "Electronic", duration: 337 }),
    makeTrack(21, "Daft Punk", "Random Access Memories",  "Lose Yourself to Dance",  3, { year: 2013, genre: "Electronic", duration: 353 }),
  ];

  // Group tracks into artist > album tree
  function buildTree(trackList) {
    const artistMap = {};
    for (const t of trackList) {
      const aKey = t.album_artist || t.artist || "Unknown Artist";
      if (!artistMap[aKey]) artistMap[aKey] = {};
      const albKey = t.album || "Unknown Album";
      if (!artistMap[aKey][albKey]) artistMap[aKey][albKey] = { name: albKey, year: t.year, tracks: [] };
      artistMap[aKey][albKey].tracks.push(t);
    }
    const artists = Object.keys(artistMap).sort().map(name => ({
      name,
      albums: Object.values(artistMap[name]),
    }));
    return { root: LIBRARY_ROOT, artists, total_tracks: trackList.length };
  }

  const libraryTree = buildTree(tracks);

  const libraryStats = {
    total_tracks: tracks.length,
    total_artists: 3,
    total_albums: 5,
    total_size: tracks.reduce((s, t) => s + t.file_size, 0),
    total_duration_secs: tracks.reduce((s, t) => s + (t.duration_secs || 0), 0),
    avg_bitrate: 780,
    formats: [
      { format: "FLAC", count: 18, total_size: 18 * 35_000_000 },
      { format: "MP3",  count: 3,  total_size: 3 * 7_500_000 },
    ],
    genres: [
      { genre: "Progressive Rock", count: 8 },
      { genre: "Alternative Rock",  count: 7 },
      { genre: "Electronic",        count: 6 },
    ],
  };

  const playlists = [
    { id: "pl-1", name: "Chill Vibes",    created_at: 1700000000, updated_at: 1700100000 },
    { id: "pl-2", name: "Road Trip",      created_at: 1700000000, updated_at: 1700200000 },
    { id: "pl-3", name: "Late Night",     created_at: 1700000000, updated_at: 1700300000 },
  ];

  const syncProfiles = [
    {
      id: "prof-1", name: "Laptop Backup", source_path: "/Users/demo/Music",
      target_path: "/Volumes/Backup/Music", sync_mode: "one_way",
      exclude_patterns: ["*.tmp"], created_at: 1700000000, last_synced_at: 1700500000,
    },
    {
      id: "prof-2", name: "NAS Sync", source_path: "/Users/demo/Music",
      target_path: "/Volumes/NAS/Music", sync_mode: "two_way",
      exclude_patterns: [], created_at: 1700000000, last_synced_at: null,
    },
  ];

  const devices = [
    {
      device: {
        id: "dev-1", name: "iPhone 15", volume_uuid: "uuid-1",
        volume_name: "iPhone", mount_path: "/Volumes/iPhone",
        capacity_bytes: 128_000_000_000, music_folder: "Music",
        created_at: 1700000000, last_synced_at: 1700400000,
      },
      connected: true,
      selected_artists: ["Pink Floyd", "Radiohead"],
      selected_albums: [],
    },
    {
      device: {
        id: "dev-2", name: "SD Card", volume_uuid: "uuid-2",
        volume_name: "MUSIC_SD", mount_path: null,
        capacity_bytes: 64_000_000_000, music_folder: "Music",
        created_at: 1700000000, last_synced_at: null,
      },
      connected: false,
      selected_artists: [],
      selected_albums: [],
    },
  ];

  const allSettings = [
    ["library_root", LIBRARY_ROOT],
    ["library_view_mode", "artist"],
    ["theme", "dark"],
  ];

  const artistSummaries = [
    { name: "Daft Punk",  album_count: 2, track_count: 6,  total_size: 6 * 15_000_000 },
    { name: "Pink Floyd", album_count: 2, track_count: 8,  total_size: 8 * 35_000_000 },
    { name: "Radiohead",  album_count: 2, track_count: 7,  total_size: 7 * 35_000_000 },
  ];

  const albumSummaries = [
    { artist_name: "Daft Punk",  album_name: "Discovery",              track_count: 3, total_size: 22_700_000, year: 2001 },
    { artist_name: "Daft Punk",  album_name: "Random Access Memories", track_count: 3, total_size: 105_000_000, year: 2013 },
    { artist_name: "Pink Floyd", album_name: "The Dark Side of the Moon", track_count: 5, total_size: 175_000_000, year: 1973 },
    { artist_name: "Pink Floyd", album_name: "Wish You Were Here",     track_count: 3, total_size: 105_000_000, year: 1975 },
    { artist_name: "Radiohead",  album_name: "In Rainbows",            track_count: 3, total_size: 105_000_000, year: 2007 },
    { artist_name: "Radiohead",  album_name: "OK Computer",            track_count: 4, total_size: 140_000_000, year: 1997 },
  ];

  // ── IPC handler ────────────────────────────────────────────────────────

  let callbackId = 0;
  const callbacks = {};

  const commandHandlers = {
    get_setting: (args) => {
      const entry = allSettings.find(([k]) => k === args.key);
      return entry ? entry[1] : null;
    },
    get_all_settings: () => allSettings,
    get_library_tree: () => libraryTree,
    get_incomplete_tracks: () => [],
    get_library_stats: () => libraryStats,
    list_playlists: () => playlists,
    list_profiles: () => syncProfiles,
    list_devices: () => devices,
    detect_volumes: () => [],
    list_artists: () => artistSummaries,
    list_albums: () => albumSummaries,
    set_setting: () => null,
    search_library: () => [],
    get_track_artwork: () => null,
    toggle_favorite: () => true,
    is_favorite: () => false,
    list_favorites: () => [],
    list_all_favorites: () => [],
    get_favorite_tracks: () => [],
  };

  window.__TAURI_INTERNALS__ = {
    invoke: async function(cmd, args) {
      // Handle plugin commands (e.g. "plugin:dialog|open")
      if (cmd.startsWith("plugin:")) {
        return null;
      }
      const handler = commandHandlers[cmd];
      if (handler) {
        return handler(args || {});
      }
      console.warn("[tauri-mock] unhandled command:", cmd, args);
      return null;
    },

    transformCallback: function(callback, once) {
      const id = callbackId++;
      callbacks[id] = { callback, once };
      return id;
    },

    convertFileSrc: function(filePath, protocol) {
      return "https://asset.localhost/" + encodeURIComponent(filePath);
    },

    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { label: "main" },
    },
  };
})();
`;
