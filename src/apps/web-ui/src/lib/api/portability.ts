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
  const response = await fetch("/api/portability/import", {
    method: "POST",
    credentials: "same-origin",
    body,
  });
  if (!response.ok) {
    const payload = (await response.json().catch(() => ({}))) as {
      error?: string;
    };
    throw new Error(payload.error ?? `import failed (${response.status})`);
  }
  return (await response.json()) as ImportReport;
}
