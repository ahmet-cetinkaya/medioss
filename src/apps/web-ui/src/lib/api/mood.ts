import { request } from "./request";

export type MoodRange = "today" | "last7" | "last30" | "all";
export interface MoodEntry {
  id: number;
  score: number;
  note: string | null;
  recorded_at: string;
  updated_at: string;
}
export interface MoodStatistics {
  entry_count: number;
  average_score: number | null;
  distribution: number[];
  distribution_percentages: number[];
  trend: { date: string; average_score: number }[];
}
export interface MoodHistoryPage {
  items: MoodEntry[];
  next_cursor: string | null;
}
export interface TimelineEvent {
  id: number;
  kind: string;
  title: string;
  score: number | null;
  note: string | null;
  recorded_at: string;
}
export interface TimelinePage {
  items: TimelineEvent[];
  next_cursor: string | null;
}
const query = (range: MoodRange) =>
  `range=${range}&tz_offset_minutes=${-new Date().getTimezoneOffset()}`;
export const listMood = (range: MoodRange, cursor?: string) =>
  request<MoodHistoryPage>(
    `/api/mood?${query(range)}${cursor ? `&cursor=${encodeURIComponent(cursor)}` : ""}`,
  );
export const createMood = (score: number, note?: string) =>
  request<MoodEntry>("/api/mood", {
    method: "POST",
    body: JSON.stringify({ score, note: note || null }),
  });
export const updateMood = (id: number, score: number, note?: string) =>
  request<MoodEntry>(`/api/mood/${id}`, {
    method: "PUT",
    body: JSON.stringify({ score, note: note || null }),
  });
export const deleteMood = (id: number) =>
  request<void>(`/api/mood/${id}`, { method: "DELETE" });
export const moodStatistics = (range: MoodRange) =>
  request<MoodStatistics>(`/api/mood/statistics?${query(range)}`);
export const timeline = (range: MoodRange, cursor?: string) =>
  request<TimelinePage>(
    `/api/timeline?${query(range)}${cursor ? `&cursor=${encodeURIComponent(cursor)}` : ""}`,
  );
