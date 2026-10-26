import type { Meditation } from "./meditation-types";
import { request } from "./request";

/** The signed-in user's favorites, newest-favorited first. */
export async function fetchFavorites(): Promise<Meditation[]> {
  return request<Meditation[]>("/api/meditation/favorites", undefined, {
    action: "favorites.load",
  });
}

export async function favoriteMeditation(id: number): Promise<void> {
  await request(
    `/api/meditation/${id}/favorite`,
    { method: "PUT" },
    { action: "favorite.add" },
  );
}

export async function unfavoriteMeditation(id: number): Promise<void> {
  await request(
    `/api/meditation/${id}/favorite`,
    { method: "DELETE" },
    { action: "favorite.remove" },
  );
}
