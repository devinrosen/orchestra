import * as commands from "../api/commands";

class FavoritesStore {
  favorites = $state<Set<string>>(new Set());
  loading = $state(false);
  error = $state<string | null>(null);

  private makeKey(entityType: string, entityId: string): string {
    return `${entityType}:${entityId}`;
  }

  isFavorite(entityType: string, entityId: string): boolean {
    return this.favorites.has(this.makeKey(entityType, entityId));
  }

  async load() {
    this.loading = true;
    this.error = null;
    try {
      const all = await commands.listAllFavorites();
      const set = new Set<string>();
      for (const fav of all) {
        set.add(this.makeKey(fav.entity_type, fav.entity_id));
      }
      this.favorites = set;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async toggle(entityType: string, entityId: string) {
    try {
      const isNowFavorited = await commands.toggleFavorite(entityType, entityId);
      const next = new Set(this.favorites);
      const key = this.makeKey(entityType, entityId);
      if (isNowFavorited) {
        next.add(key);
      } else {
        next.delete(key);
      }
      this.favorites = next;
    } catch (e) {
      this.error = String(e);
    }
  }
}

export const favoritesStore = new FavoritesStore();
