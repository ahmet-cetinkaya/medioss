// Where Login sends a signed-in user (plan D2 "Login page"; FR-081, SC-003).
// Pure: the caller passes `location.search` and `location.origin`.

import { safeNext } from "../../navigation/safe-next";
import { ROUTES } from "../../navigation/routes";

/** The validated `next` screen address, else Home. */
export function loginDestination(search: string, origin: string): string {
  return (
    safeNext(new URLSearchParams(search).get("next"), origin) ??
    ROUTES.home.path
  );
}
