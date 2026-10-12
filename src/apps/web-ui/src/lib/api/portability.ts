import { request } from "./request";

export interface ImportReport {
  mood_created: number;
  mood_duplicates_skipped: number;
  practice_linked: number;
  practice_skipped: number;
  practice_duplicates_skipped: number;
}

export async function importData(file: File): Promise<ImportReport> {
  const body = new FormData();
  body.append("file", file);
  return request<ImportReport>(
    "/api/portability/import",
    { method: "POST", body },
    { action: "import" },
  );
}

/** Same-origin export downloads (attachment responses for `<a href>`). */
export const exportJsonUrl = () => "/api/portability/export/json";
export const exportCsvUrl = () => "/api/portability/export/csv";
