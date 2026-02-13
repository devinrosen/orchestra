import type { ArtistNode, Track, AlbumEntry, GenreNode, FolderNode } from "../api/types";

export function flattenTree(artists: ArtistNode[]): Track[] {
  const tracks: Track[] = [];
  for (const artist of artists) {
    for (const album of artist.albums) {
      for (const track of album.tracks) {
        tracks.push(track);
      }
    }
  }
  return tracks;
}

export function groupByAlbum(tracks: Track[]): AlbumEntry[] {
  const map = new Map<string, AlbumEntry>();

  for (const track of tracks) {
    const artist = track.album_artist ?? track.artist ?? "Unknown Artist";
    const album = track.album ?? "Unknown Album";
    const key = `${artist}\0${album}`;

    let entry = map.get(key);
    if (!entry) {
      entry = { name: album, artist, year: track.year, tracks: [] };
      map.set(key, entry);
    }
    entry.tracks.push(track);
    if (track.year != null && entry.year == null) {
      entry.year = track.year;
    }
  }

  const entries = Array.from(map.values());
  entries.sort((a, b) => a.name.localeCompare(b.name));

  for (const entry of entries) {
    entry.tracks.sort((a, b) => {
      const da = a.disc_number ?? 1;
      const db = b.disc_number ?? 1;
      if (da !== db) return da - db;
      const ta = a.track_number ?? 0;
      const tb = b.track_number ?? 0;
      return ta - tb;
    });
  }

  return entries;
}

export function groupByGenre(tracks: Track[]): GenreNode[] {
  const genreMap = new Map<string, Map<string, AlbumEntry>>();

  for (const track of tracks) {
    const genre = track.genre ?? "Unknown Genre";
    const artist = track.album_artist ?? track.artist ?? "Unknown Artist";
    const album = track.album ?? "Unknown Album";
    const albumKey = `${artist}\0${album}`;

    let albumMap = genreMap.get(genre);
    if (!albumMap) {
      albumMap = new Map();
      genreMap.set(genre, albumMap);
    }

    let entry = albumMap.get(albumKey);
    if (!entry) {
      entry = { name: album, artist, year: track.year, tracks: [] };
      albumMap.set(albumKey, entry);
    }
    entry.tracks.push(track);
    if (track.year != null && entry.year == null) {
      entry.year = track.year;
    }
  }

  const nodes: GenreNode[] = [];
  for (const [name, albumMap] of genreMap) {
    const albums = Array.from(albumMap.values());
    albums.sort((a, b) => a.artist.localeCompare(b.artist) || a.name.localeCompare(b.name));
    for (const album of albums) {
      album.tracks.sort((a, b) => {
        const da = a.disc_number ?? 1;
        const db = b.disc_number ?? 1;
        if (da !== db) return da - db;
        return (a.track_number ?? 0) - (b.track_number ?? 0);
      });
    }
    nodes.push({ name, albums });
  }

  nodes.sort((a, b) => {
    if (a.name === "Unknown Genre") return 1;
    if (b.name === "Unknown Genre") return -1;
    return a.name.localeCompare(b.name);
  });

  return nodes;
}

export function groupByFolder(tracks: Track[]): FolderNode {
  const root: FolderNode = { name: "/", path: "", children: [], tracks: [] };

  for (const track of tracks) {
    const parts = track.relative_path.split("/");
    const fileName = parts.pop()!;
    let current = root;

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      let child = current.children.find((c) => c.name === part);
      if (!child) {
        child = { name: part, path: parts.slice(0, i + 1).join("/"), children: [], tracks: [] };
        current.children.push(child);
      }
      current = child;
    }

    current.tracks.push(track);
  }

  sortFolderNode(root);
  return root;
}

function sortFolderNode(node: FolderNode): void {
  node.children.sort((a, b) => a.name.localeCompare(b.name));
  node.tracks.sort((a, b) => {
    const na = a.track_number ?? 0;
    const nb = b.track_number ?? 0;
    if (na !== nb) return na - nb;
    return a.relative_path.localeCompare(b.relative_path);
  });
  for (const child of node.children) {
    sortFolderNode(child);
  }
}
