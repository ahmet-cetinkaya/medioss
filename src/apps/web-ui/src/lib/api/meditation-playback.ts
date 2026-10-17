import type { PlaybackState } from "./meditation-types";
import { request } from "./request";

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
