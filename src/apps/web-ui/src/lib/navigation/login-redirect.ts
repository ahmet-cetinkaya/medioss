// The single redirect point to Login (plan D2 "Redirect helper"; spec FR-009).
// No browser access at module top level: the location defaults lazily.

import { directoryPath, isScreenAddress } from "./routes";

export type LocationLike = {
  readonly pathname: string;
  readonly search: string;
  replace(url: string): void;
};

const AUTH_PATHS = ["/login/", "/setup/"];
const LOGIN_PATH = "/login/";

let redirected = false;

/**
 * Replaces the page with Login, remembering the current screen as `next`.
 * Fires at most once per page load and never on Login or Setup. Returns
 * whether the page is leaving for Login (now or by an earlier redirect);
 * `false` means it was skipped because this is Login or Setup.
 */
export function redirectToLogin(loc: LocationLike = window.location): boolean {
  if (redirected) return true;
  const path = directoryPath(loc.pathname);
  if (AUTH_PATHS.includes(path)) return false;
  redirected = true;
  loc.replace(
    isScreenAddress(path)
      ? `${LOGIN_PATH}?next=${encodeURIComponent(loc.pathname + loc.search)}`
      : LOGIN_PATH,
  );
  return true;
}

/**
 * Sign-out and own-account deletion: Login with no `next`. Also blocks a later
 * `redirectToLogin` (e.g. a racing 401) from adding a `next`.
 */
export function goToLogin(loc: LocationLike = window.location): void {
  redirected = true;
  loc.replace(LOGIN_PATH);
}

/** Test-only: simulates a fresh page load. */
export function resetLoginRedirectForTests(): void {
  redirected = false;
}
