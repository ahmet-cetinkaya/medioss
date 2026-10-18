import { rangeQuery } from "./date-range";
import type { StatisticsRange, UserStatistics } from "./meditation-types";
import { request } from "./request";

export function fetchStatistics(
  range: StatisticsRange,
): Promise<UserStatistics> {
  const tzOffsetMinutes = -new Date().getTimezoneOffset();
  return request<UserStatistics>(
    `/api/meditation/statistics?${rangeQuery(range)}&tz_offset_minutes=${tzOffsetMinutes}`,
    undefined,
    { action: "statistics.load" },
  );
}
