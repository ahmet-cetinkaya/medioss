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
  /** Key of a bundled glyph, or null. */
  icon_key: string | null;
  /** True when an uploaded icon image exists (GET soundIconUrl). */
  has_icon: boolean;
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
  request<SoundscapeCategory[]>("/api/soundscape/categories", undefined, {
    action: "soundscape.categories.load",
  });
export const createSoundscapeCategory = (name: string) =>
  request<SoundscapeCategory>(
    "/api/soundscape/categories",
    { method: "POST", body: JSON.stringify({ name }) },
    { action: "soundscape.category.create" },
  );
export const renameSoundscapeCategory = (id: number, name: string) =>
  request<SoundscapeCategory>(
    `/api/soundscape/categories/${id}`,
    { method: "PUT", body: JSON.stringify({ name }) },
    { action: "soundscape.category.rename" },
  );
export const deleteSoundscapeCategory = (id: number) =>
  request<void>(
    `/api/soundscape/categories/${id}`,
    { method: "DELETE" },
    { action: "soundscape.category.delete" },
  );

export const fetchSounds = () =>
  request<SoundscapeSound[]>("/api/soundscape/sounds", undefined, {
    action: "soundscape.sounds.load",
  });
export const soundAudioUrl = (id: number) =>
  `/api/soundscape/sounds/${id}/audio`;
export const soundDownloadUrl = (id: number) =>
  `/api/soundscape/sounds/${id}/download`;
export const soundIconUrl = (id: number) => `/api/soundscape/sounds/${id}/icon`;
export interface SoundIconInput {
  iconKey?: string | null;
  iconFile?: File | null;
}
export function uploadSound(
  name: string,
  categoryId: number,
  file: File,
  icon: SoundIconInput = {},
) {
  const form = new FormData();
  form.set("name", name);
  form.set("category_id", String(categoryId));
  form.set("file", file);
  if (icon.iconKey) form.set("icon_key", icon.iconKey);
  if (icon.iconFile) form.set("icon", icon.iconFile);
  return request<SoundscapeSound>(
    "/api/soundscape/sounds",
    { method: "POST", body: form },
    { action: "sound.upload" },
  );
}
/** `iconKey` replaces the stored glyph key; null clears it. */
export const updateSound = (
  id: number,
  name: string,
  categoryId: number,
  iconKey: string | null = null,
) =>
  request<SoundscapeSound>(
    `/api/soundscape/sounds/${id}`,
    {
      method: "PUT",
      body: JSON.stringify({
        name,
        category_id: categoryId,
        icon_key: iconKey,
      }),
    },
    { action: "sound.update" },
  );
export function uploadSoundIcon(id: number, file: File) {
  const form = new FormData();
  form.set("icon", file);
  return request<SoundscapeSound>(
    `/api/soundscape/sounds/${id}/icon`,
    { method: "PUT", body: form },
    { action: "sound.icon.upload" },
  );
}
export const clearSoundIcon = (id: number) =>
  request<SoundscapeSound>(
    `/api/soundscape/sounds/${id}/icon`,
    { method: "DELETE" },
    { action: "sound.icon.clear" },
  );
export const deleteSound = (id: number) =>
  request<void>(
    `/api/soundscape/sounds/${id}`,
    { method: "DELETE" },
    { action: "sound.delete" },
  );

export const fetchSoundscapeMixes = () =>
  request<SoundscapeMix[]>("/api/soundscape/mixes", undefined, {
    action: "mixes.load",
  });
export const fetchSoundscapeMix = (id: number) =>
  request<SoundscapeMix>(`/api/soundscape/mixes/${id}`, undefined, {
    action: "mix.load",
  });
export const createSoundscapeMix = (name: string, items: SoundscapeMixItem[]) =>
  request<SoundscapeMix>(
    "/api/soundscape/mixes",
    { method: "POST", body: JSON.stringify({ name, items }) },
    { action: "mix.create" },
  );
export const updateSoundscapeMix = (
  id: number,
  name: string,
  items: SoundscapeMixItem[],
) =>
  request<SoundscapeMix>(
    `/api/soundscape/mixes/${id}`,
    { method: "PUT", body: JSON.stringify({ name, items }) },
    { action: "mix.update" },
  );
export const deleteSoundscapeMix = (id: number) =>
  request<void>(
    `/api/soundscape/mixes/${id}`,
    { method: "DELETE" },
    { action: "mix.delete" },
  );
