// The Player address id (PB-18): a positive integer, else null (opens Library).
// Canonical form is the path `/player/<id>/`; the legacy `?id=` still works.

function toId(raw: string | null): number | null {
  if (!raw || !/^\d+$/.test(raw)) return null;
  const id = Number(raw);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

export function parseMeditationId(
  search: string,
  pathname = "",
): number | null {
  const fromPath = /^\/player\/([^/]+)\/?$/.exec(pathname);
  if (fromPath) return toId(fromPath[1]);
  return toId(new URLSearchParams(search).get("id"));
}
