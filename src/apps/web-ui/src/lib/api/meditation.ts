/**
 * Typed client for the meditation API. Every call is same-origin with
 * session cookies; failures surface as thrown `ApiError` values.
 */

import { request } from "./request";

export interface Meditation {
  id: number;
  name: string;
  content_type: string;
  size_bytes: number;
  duration_ms: number;
  created_at: string;
  completed: boolean;
  progress_percent: number;
  program_id?: number;
  program_has_cover?: boolean;
  description: string | null;
}

export interface Category {
  id: number;
  name: string;
}

export interface Program {
  id: number;
  title: string;
  description: string | null;
  has_cover: boolean;
  category_id: number | null;
  created_at: string;
  progress: number;
}

export interface Note {
  id: number;
  text: string;
  created_at: string;
  updated_at: string;
}

export interface PlaybackState {
  position_ms: number;
  completed: boolean;
}

export type StatisticsRange = "today" | "last30" | "all";

export interface StatisticsDay {
  date: string;
  minutes: number;
}

export interface UserStatistics {
  range: StatisticsRange;
  total_minutes: number;
  total_days: number;
  current_streak: number;
  longest_streak: number;
  calendar: StatisticsDay[];
}

export function fetchStatistics(
  range: StatisticsRange,
): Promise<UserStatistics> {
  const tzOffsetMinutes = -new Date().getTimezoneOffset();
  return request<UserStatistics>(
    `/api/meditation/statistics?range=${range}&tz_offset_minutes=${tzOffsetMinutes}`,
    undefined,
    { action: "statistics.load" },
  );
}

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

export async function fetchPrograms(): Promise<Program[]> {
  return request<Program[]>("/api/meditation/programs", undefined, {
    action: "programs.load",
  });
}

export async function fetchProgram(id: number): Promise<Program> {
  return request<Program>(`/api/meditation/programs/${id}`, undefined, {
    action: "program.load",
  });
}

export async function fetchProgramMembers(id: number): Promise<Meditation[]> {
  return request<Meditation[]>(
    `/api/meditation/programs/${id}/members`,
    undefined,
    { action: "program.members.load" },
  );
}

export interface ContinueProgram {
  program: Program;
  meditation_id: number;
}

export async function fetchContinueProgram(): Promise<ContinueProgram | null> {
  return (
    (await request<ContinueProgram | undefined>(
      "/api/meditation/continue",
      undefined,
      { action: "continue.load" },
    )) ?? null
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

export interface RecentProgram {
  program: Program;
  last_touched_at: string;
}

export async function fetchRecentPrograms(
  limit?: number,
): Promise<RecentProgram[]> {
  const query = limit !== undefined ? `?limit=${limit}` : "";
  return request<RecentProgram[]>(
    `/api/meditation/recent-programs${query}`,
    undefined,
    { action: "recentPrograms.load" },
  );
}

export async function fetchCategories(): Promise<Category[]> {
  return request<Category[]>("/api/meditation/categories", undefined, {
    action: "categories.load",
  });
}

export async function fetchPlaybackState(id: number): Promise<PlaybackState> {
  return request<PlaybackState>(
    `/api/meditation/${id}/playback-state`,
    undefined,
    { action: "playback.load" },
  );
}

export async function postPosition(
  id: number,
  positionMs: number,
): Promise<void> {
  await request<void>(
    `/api/meditation/${id}/playback-state`,
    { method: "POST", body: JSON.stringify({ position_ms: positionMs }) },
    { action: "playback.position" },
  );
}

export async function postCompletion(
  id: number,
  startedAt: string,
  finishedAt: string,
): Promise<{ session_id: number }> {
  return request(
    `/api/meditation/${id}/completion`,
    {
      method: "POST",
      body: JSON.stringify({ started_at: startedAt, finished_at: finishedAt }),
    },
    { action: "playback.completion" },
  );
}

export async function putCompleted(
  id: number,
  completed: boolean,
): Promise<void> {
  await request(
    `/api/meditation/${id}/completed`,
    { method: "PUT", body: JSON.stringify({ completed }) },
    { action: "meditation.completed" },
  );
}

export async function fetchNotes(id: number): Promise<Note[]> {
  return request<Note[]>(`/api/meditation/${id}/notes`, undefined, {
    action: "notes.load",
  });
}

export async function createNote(id: number, text: string): Promise<Note> {
  return request(
    `/api/meditation/${id}/notes`,
    { method: "POST", body: JSON.stringify({ text }) },
    { action: "note.create" },
  );
}

export async function updateNote(
  id: number,
  noteId: number,
  text: string,
): Promise<void> {
  await request(
    `/api/meditation/${id}/notes/${noteId}`,
    { method: "PUT", body: JSON.stringify({ text }) },
    { action: "note.update" },
  );
}

export async function deleteNote(id: number, noteId: number): Promise<void> {
  await request(
    `/api/meditation/${id}/notes/${noteId}`,
    { method: "DELETE" },
    { action: "note.delete" },
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

export async function renameCategory(id: number, name: string): Promise<void> {
  await request(
    `/api/meditation/categories/${id}`,
    { method: "PUT", body: JSON.stringify({ name }) },
    { action: "category.rename" },
  );
}

export async function deleteCategory(id: number): Promise<void> {
  await request(
    `/api/meditation/categories/${id}`,
    { method: "DELETE" },
    { action: "category.delete" },
  );
}

export async function createCategory(name: string): Promise<Category> {
  return request(
    "/api/meditation/categories",
    { method: "POST", body: JSON.stringify({ name }) },
    { action: "category.create" },
  );
}

export async function createProgram(
  title: string,
  description: string | null,
  cover: File | null,
): Promise<Program> {
  const form = new FormData();
  form.append("title", title);
  if (description) form.append("description", description);
  if (cover) form.append("cover", cover);
  return request<Program>(
    "/api/meditation/programs",
    { method: "POST", body: form },
    { action: "program.create" },
  );
}

export async function updateProgram(
  programId: number,
  title: string,
  description: string | null,
): Promise<void> {
  await request(
    `/api/meditation/programs/${programId}`,
    {
      method: "PUT",
      body: JSON.stringify({ title, description }),
    },
    { action: "program.update" },
  );
}

export async function replaceProgramCover(
  programId: number,
  cover: File,
): Promise<void> {
  const form = new FormData();
  form.append("cover", cover);
  await request(
    `/api/meditation/programs/${programId}/cover`,
    { method: "PUT", body: form },
    { action: "program.cover" },
  );
}

export async function assignCategory(
  programId: number,
  categoryId: number | null,
): Promise<void> {
  await request(
    `/api/meditation/programs/${programId}/category`,
    { method: "PUT", body: JSON.stringify({ category_id: categoryId }) },
    { action: "program.category" },
  );
}

export async function addMember(
  programId: number,
  meditationId: number,
): Promise<void> {
  await request(
    `/api/meditation/programs/${programId}/members`,
    { method: "POST", body: JSON.stringify({ meditation_id: meditationId }) },
    { action: "program.member.add" },
  );
}

export async function removeMember(
  programId: number,
  meditationId: number,
): Promise<void> {
  await request(
    `/api/meditation/programs/${programId}/members/${meditationId}`,
    { method: "DELETE" },
    { action: "program.member.remove" },
  );
}

export async function reorderProgramMeditations(
  programId: number,
  meditationIds: number[],
): Promise<number[]> {
  const result = await request<{ meditation_ids: number[] }>(
    `/api/meditation/programs/${programId}/meditations/order`,
    {
      method: "PUT",
      body: JSON.stringify({ meditation_ids: meditationIds }),
    },
    { action: "program.members.reorder" },
  );
  return result.meditation_ids;
}

/** Same-origin media and download addresses (used by `<img>`, `<audio>`, `<a>`). */
export function programCoverUrl(id: number, version?: number): string {
  const url = `/api/meditation/programs/${id}/cover`;
  return version === undefined ? url : `${url}?v=${version}`;
}
export const meditationAudioUrl = (id: number) => `/api/meditation/${id}/audio`;
export const meditationDownloadUrl = (id: number) =>
  `/api/meditation/${id}/download`;

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
