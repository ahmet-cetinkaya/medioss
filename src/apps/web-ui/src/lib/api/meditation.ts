/**
 * Typed client for the meditation API. Every call is same-origin with
 * session cookies; JSON errors surface as thrown Error values.
 */

export interface Meditation {
  id: number;
  name: string;
  content_type: string;
  size_bytes: number;
  duration_ms: number;
  created_at: string;
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
  progress: number | null;
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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    credentials: "same-origin",
    ...init,
    headers: {
      ...(init?.body ? { "content-type": "application/json" } : {}),
      ...init?.headers,
    },
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as {
      error?: string;
    };
    throw new Error(body.error ?? `request failed (${response.status})`);
  }
  // 204 is not the only empty-body success: handlers like add-member
  // answer 201 with no payload, so parse only when a body exists.
  if (
    response.status === 204 ||
    response.headers.get("content-length") === "0"
  ) {
    return undefined as T;
  }
  const text = await response.text();
  if (!text) return undefined as T;
  return JSON.parse(text) as T;
}

export async function fetchLibrary(): Promise<Meditation[]> {
  return request<Meditation[]>("/api/meditation/library");
}

export async function fetchPrograms(): Promise<Program[]> {
  return request<Program[]>("/api/meditation/programs");
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
