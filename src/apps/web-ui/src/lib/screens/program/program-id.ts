// The Program address id (D10 "Addresses", PB-18): only a positive whole
// number in decimal digits is valid; anything else means "open Library".

/** The `id` query value as a positive integer, or null when missing or invalid. */
export function parseProgramId(search: string): number | null {
  const raw = new URLSearchParams(search).get("id");
  if (raw === null || !/^\d+$/.test(raw)) return null;
  const id = Number(raw);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}
