// The last USED saved mix (plan D10): its id is remembered so a screen can
// offer it again. Storage may be unavailable (strict private browsing,
// sandboxed frames); failures fall back silently, like locale-storage.ts.

export const LAST_MIX_STORAGE_KEY = "medioss.lastMix";

export type LastMixStorage = Pick<Storage, "getItem" | "setItem">;

/** The remembered mix id, or null when absent, malformed, or storage fails. */
export function readLastMixId(storage?: LastMixStorage): number | null {
  try {
    const raw = (storage ?? globalThis.localStorage)?.getItem(
      LAST_MIX_STORAGE_KEY,
    );
    const id = Number(raw);
    return raw && Number.isSafeInteger(id) && id > 0 ? id : null;
  } catch {
    return null;
  }
}

export function writeLastMixId(id: number, storage?: LastMixStorage): void {
  try {
    (storage ?? globalThis.localStorage)?.setItem(
      LAST_MIX_STORAGE_KEY,
      String(id),
    );
  } catch {
    // Storage unavailable: nothing is remembered.
  }
}
