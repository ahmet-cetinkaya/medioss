import type {
  Meditation,
  Program,
  ContinueProgram,
  RecentProgram,
} from "./meditation-types";
import { request } from "./request";

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

export async function fetchContinueProgram(): Promise<ContinueProgram | null> {
  return (
    (await request<ContinueProgram | undefined>(
      "/api/meditation/continue",
      undefined,
      { action: "continue.load" },
    )) ?? null
  );
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

export async function deleteProgram(id: number): Promise<void> {
  await request(
    `/api/meditation/programs/${id}`,
    { method: "DELETE" },
    { action: "program.delete" },
  );
}

export async function removeProgramCover(id: number): Promise<void> {
  await request(
    `/api/meditation/programs/${id}/cover`,
    { method: "DELETE" },
    { action: "program.cover.remove" },
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
