import type { DateRangeValue } from "./date-range";

/** The home-program summary the meditation detail response carries: the
 * Player borrows the cover and links to the program by title. */
export interface ProgramSummary {
  id: number;
  title: string;
  has_cover: boolean;
}

/** A program a favorite belongs to, named on the favorites list. */
export interface ProgramLabel {
  id: number;
  title: string;
}

export interface Meditation {
  id: number;
  name: string;
  content_type: string;
  size_bytes: number;
  duration_ms: number;
  created_at: string;
  completed: boolean;
  progress_percent: number;
  favorite: boolean;
  program_id?: number;
  program_has_cover?: boolean;
  program?: ProgramSummary | null;
  /** Every program the meditation belongs to; only the favorites list sends it. */
  programs?: ProgramLabel[];
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

export type StatisticsPreset = "today" | "last30" | "all";
export type StatisticsRange = DateRangeValue<StatisticsPreset>;

export interface StatisticsDay {
  date: string;
  minutes: number;
}

export interface UserStatistics {
  /** The preset name, or "custom" for a from/to request. */
  range: StatisticsPreset | "custom";
  total_minutes: number;
  total_days: number;
  current_streak: number;
  longest_streak: number;
  calendar: StatisticsDay[];
}

export interface ContinueProgram {
  program: Program;
  meditation_id: number;
}

export interface RecentProgram {
  program: Program;
  last_touched_at: string;
}
