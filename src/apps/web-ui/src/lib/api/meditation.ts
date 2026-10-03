/**
 * Typed client for the meditation API. Every call is same-origin with
 * session cookies; JSON errors surface as thrown Error values.
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

/** "100% complete" once every member is completed, else "NN%". */
export function programProgressLabel(
  program: Pick<Program, "progress">,
): string {
  const percent = Math.round(program.progress * 100);
  return program.progress >= 1 ? "Completed" : `${percent}% complete`;
}

/** Same shape for a single meditation's own completion state/percent. */
export function meditationProgressLabel(
  meditation: Pick<Meditation, "completed" | "progress_percent">,
): string {
  return meditation.completed ? "Completed" : `${meditation.progress_percent}%`;
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
  );
}

export async function fetchMeditation(id: number): Promise<Meditation> {
	return request<Meditation>(`/api/meditation/${id}`);
}

export async function fetchLibrary(query?: string): Promise<Meditation[]> {
  const q = query?.trim();
  return request<Meditation[]>(
    `/api/meditation/library${q ? `?q=${encodeURIComponent(q)}` : ""}`,
  );
}

export async function fetchPrograms(): Promise<Program[]> {
  return request<Program[]>("/api/meditation/programs");
}

export async function fetchProgram(id: number): Promise<Program> {
  return request<Program>(`/api/meditation/programs/${id}`);
}

export async function fetchProgramMembers(id: number): Promise<Meditation[]> {
  return request<Meditation[]>(`/api/meditation/programs/${id}/members`);
}

export interface ContinueProgram {
  program: Program;
  meditation_id: number;
}

export async function fetchContinueProgram(): Promise<ContinueProgram | null> {
  return (
    (await request<ContinueProgram | undefined>("/api/meditation/continue")) ??
    null
  );
}

export async function fetchDailySuggestion(): Promise<Meditation | null> {
  return (
    (await request<Meditation | undefined>(
      "/api/meditation/daily-suggestion",
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
  return request<RecentProgram[]>(`/api/meditation/recent-programs${query}`);
}

export async function fetchCategories(): Promise<Category[]> {
  return request<Category[]>("/api/meditation/categories");
}

export async function fetchPlaybackState(id: number): Promise<PlaybackState> {
  return request<PlaybackState>(`/api/meditation/${id}/playback-state`);
}

export async function postPosition(
  id: number,
  positionMs: number,
): Promise<void> {
  await request<void>(`/api/meditation/${id}/playback-state`, {
    method: "POST",
    body: JSON.stringify({ position_ms: positionMs }),
  });
}

export async function postCompletion(
  id: number,
  startedAt: string,
  finishedAt: string,
): Promise<{ session_id: number }> {
  return request(`/api/meditation/${id}/completion`, {
    method: "POST",
    body: JSON.stringify({ started_at: startedAt, finished_at: finishedAt }),
  });
}

export async function putCompleted(
  id: number,
  completed: boolean,
): Promise<void> {
  await request(`/api/meditation/${id}/completed`, {
    method: "PUT",
    body: JSON.stringify({ completed }),
  });
}

export async function fetchNotes(id: number): Promise<Note[]> {
  return request<Note[]>(`/api/meditation/${id}/notes`);
}

export async function createNote(id: number, text: string): Promise<Note> {
  return request(`/api/meditation/${id}/notes`, {
    method: "POST",
    body: JSON.stringify({ text }),
  });
}

export async function updateNote(
  id: number,
  noteId: number,
  text: string,
): Promise<void> {
  await request(`/api/meditation/${id}/notes/${noteId}`, {
    method: "PUT",
    body: JSON.stringify({ text }),
  });
}

export async function deleteNote(id: number, noteId: number): Promise<void> {
  await request(`/api/meditation/${id}/notes/${noteId}`, { method: "DELETE" });
}

export async function fetchMe(): Promise<{
  id: number;
  email: string;
  role: string;
}> {
  return request("/api/identity/me");
}

export async function uploadAudio(
  file: File,
  name: string,
  programId: number,
): Promise<Meditation> {
  const form = new FormData();
  form.append("file", file);
  form.append("name", name);
  form.append("program_id", String(programId));
  const response = await fetch("/api/meditation/audio", {
    method: "POST",
    credentials: "same-origin",
    body: form,
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as {
      error?: string;
    };
    throw new Error(body.error ?? `upload failed (${response.status})`);
  }
  return (await response.json()) as Meditation;
}

export async function createCategory(name: string): Promise<Category> {
  return request("/api/meditation/categories", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
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
  const response = await fetch("/api/meditation/programs", {
    method: "POST",
    credentials: "same-origin",
    body: form,
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as {
      error?: string;
    };
    throw new Error(body.error ?? `create failed (${response.status})`);
  }
  return (await response.json()) as Program;
}

export async function assignCategory(
  programId: number,
  categoryId: number | null,
): Promise<void> {
  await request(`/api/meditation/programs/${programId}/category`, {
    method: "PUT",
    body: JSON.stringify({ category_id: categoryId }),
  });
}

export async function addMember(
  programId: number,
  meditationId: number,
): Promise<void> {
  await request(`/api/meditation/programs/${programId}/members`, {
    method: "POST",
    body: JSON.stringify({ meditation_id: meditationId }),
  });
}

export async function removeMember(
  programId: number,
  meditationId: number,
): Promise<void> {
  await request(
    `/api/meditation/programs/${programId}/members/${meditationId}`,
    {
      method: "DELETE",
    },
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
