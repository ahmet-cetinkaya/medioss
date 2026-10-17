import type { Meditation } from "./meditation-types";
import { request } from "./request";

export async function fetchMeditation(id: number): Promise<Meditation> {
  return request<Meditation>(`/api/meditation/${id}`, undefined, {
    action: "meditation.load",
  });
}

export async function fetchLibrary(query?: string): Promise<Meditation[]> {
  const q = query?.trim();
  return request<Meditation[]>(
    `/api/meditation/library${q ? `?q=${encodeURIComponent(q)}` : ""}`,
    undefined,
    { action: "library.load" },
  );
}

export async function fetchDailySuggestion(): Promise<Meditation | null> {
  return (
    (await request<Meditation | undefined>(
      "/api/meditation/daily-suggestion",
      undefined,
      { action: "suggestion.load" },
    )) ?? null
  );
}

export async function uploadAudio(
  file: File,
  name: string,
  programId: number,
  description: string | null = null,
): Promise<Meditation> {
  const form = new FormData();
  form.append("file", file);
  form.append("name", name);
  form.append("program_id", String(programId));
  if (description) form.append("description", description);
  return request<Meditation>(
    "/api/meditation/audio",
    { method: "POST", body: form },
    { action: "audio.upload" },
  );
}

export async function updateMeditationDescription(
  id: number,
  description: string | null,
): Promise<Meditation> {
  return request<Meditation>(
    `/api/meditation/meditations/${id}`,
    { method: "PUT", body: JSON.stringify({ description }) },
    { action: "meditation.description" },
  );
}

/** Prefills an editable name from a file name: strip the extension, break
 * separator characters, Title Case each word (spec FR-024). */
export function titleCaseFromFile(fileName: string): string {
  const stem = fileName.replace(/\.[^.]+$/, "");
  return stem
    .split(/[-_.\s]+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}
