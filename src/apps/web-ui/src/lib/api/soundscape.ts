import { request } from "./request";

export interface SoundscapeCategory {
  id: number;
  name: string;
  slug: string;
}

export interface SoundscapeSound {
  id: number;
  name: string;
  category_id: number;
  content_type: string;
  size_bytes: number;
  duration_ms: number;
  created_at: string;
  builtin: boolean;
}

export interface SoundscapeMixItem {
  sound_id: number;
  volume: number;
  position: number;
}

export interface SoundscapeMix {
  id: number;
  name: string;
  items: SoundscapeMixItem[];
  created_at: string;
  updated_at: string;
  unavailable_sounds?: string[];
}

export const fetchSoundscapeCategories = () =>
  request<SoundscapeCategory[]>("/api/soundscape/categories");
export const createSoundscapeCategory = (name: string) =>
  request<SoundscapeCategory>("/api/soundscape/categories", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
export const renameSoundscapeCategory = (id: number, name: string) =>
  request<SoundscapeCategory>(`/api/soundscape/categories/${id}`, {
    method: "PUT",
    body: JSON.stringify({ name }),
  });
export const deleteSoundscapeCategory = (id: number) =>
  request<void>(`/api/soundscape/categories/${id}`, { method: "DELETE" });

export const fetchSounds = () =>
  request<SoundscapeSound[]>("/api/soundscape/sounds");
export const soundAudioUrl = (id: number) =>
  `/api/soundscape/sounds/${id}/audio`;
export function uploadSound(name: string, categoryId: number, file: File) {
  const form = new FormData();
  form.set("name", name);
  form.set("category_id", String(categoryId));
  form.set("file", file);
  return request<SoundscapeSound>("/api/soundscape/sounds", {
    method: "POST",
    body: form,
  });
}
export const updateSound = (id: number, name: string, categoryId: number) =>
  request<SoundscapeSound>(`/api/soundscape/sounds/${id}`, {
    method: "PUT",
    body: JSON.stringify({ name, category_id: categoryId }),
  });
export const deleteSound = (id: number) =>
  request<void>(`/api/soundscape/sounds/${id}`, { method: "DELETE" });

export const fetchSoundscapeMixes = () =>
  request<SoundscapeMix[]>("/api/soundscape/mixes");
export const fetchSoundscapeMix = (id: number) =>
  request<SoundscapeMix>(`/api/soundscape/mixes/${id}`);
export const createSoundscapeMix = (name: string, items: SoundscapeMixItem[]) =>
  request<SoundscapeMix>("/api/soundscape/mixes", {
    method: "POST",
    body: JSON.stringify({ name, items }),
  });
export const updateSoundscapeMix = (
  id: number,
  name: string,
  items: SoundscapeMixItem[],
) =>
  request<SoundscapeMix>(`/api/soundscape/mixes/${id}`, {
    method: "PUT",
    body: JSON.stringify({ name, items }),
  });
export const deleteSoundscapeMix = (id: number) =>
  request<void>(`/api/soundscape/mixes/${id}`, { method: "DELETE" });
