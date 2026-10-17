import type { Note } from "./meditation-types";
import { request } from "./request";

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
