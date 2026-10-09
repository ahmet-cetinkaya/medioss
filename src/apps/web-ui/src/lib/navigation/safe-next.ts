// Validates the remembered `next` address (plan D2 "Safe next"; spec FR-010, SC-003).

import { directoryPath, SCREEN_ALLOWLIST } from "./routes";

const MAX_LENGTH = 2048;
// Backslash, C0 controls, DEL, and any whitespace.
const FORBIDDEN = /[\\\u0000-\u001f\u007f\s]/;

/**
 * Returns `canonicalPath + search` for a same-origin screen address, else `null`
 * (meaning Home). `raw` comes from `URLSearchParams.get`, already decoded once;
 * it is never decoded again, and never echoed: the result is rebuilt.
 */
export function safeNext(raw: string | null, origin: string): string | null {
  if (!raw || raw.length > MAX_LENGTH) return null;
  if (raw[0] !== "/" || raw[1] === "/" || raw[1] === "\\") return null;
  if (FORBIDDEN.test(raw)) return null;

  let url: URL;
  try {
    url = new URL(raw, origin);
    if (url.origin !== new URL(origin).origin) return null;
  } catch {
    return null;
  }

  const path = directoryPath(url.pathname);
  if (!SCREEN_ALLOWLIST.includes(path)) return null;
  return path + url.search;
}
