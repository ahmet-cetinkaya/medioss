/**
 * Typed client for the identity API. Every call is same-origin with
 * session cookies; failures surface as thrown `ApiError` values.
 * The `/me` probes, login, logout, and setup opt out of the 401 redirect.
 */

import { ApiError, readErrorBody } from "./api-error";
import { apiErrorFrom, request, send } from "./request";

export interface IdentityUser {
  id: number;
  email: string;
  role: string;
  must_change_password: boolean;
  created_at: string;
  disabled: boolean;
  display_name: string | null;
  has_avatar: boolean;
}

export interface MeProfile {
  id: number;
  email: string;
  role: string;
  must_change_password: boolean;
  display_name: string | null;
  has_avatar: boolean;
}

export async function fetchUsers(): Promise<IdentityUser[]> {
  return request("/api/identity/users", undefined, { action: "users.list" });
}

export async function createUser(
  email: string,
  password: string,
): Promise<{ id: number; email: string; role: string }> {
  return request(
    "/api/identity/users",
    {
      method: "POST",
      body: JSON.stringify({ email, password }),
    },
    { action: "users.create" },
  );
}

export async function setUserStatus(
  id: number,
  disabled: boolean,
): Promise<void> {
  await request(
    `/api/identity/users/${id}/status`,
    { method: "PATCH", body: JSON.stringify({ disabled }) },
    { action: "users.status" },
  );
}

export async function deleteUser(id: number): Promise<void> {
  await request(
    `/api/identity/users/${id}`,
    { method: "DELETE" },
    { action: "users.delete" },
  );
}

export async function deleteOwnAccount(): Promise<void> {
  await request(
    "/api/identity/me",
    { method: "DELETE" },
    { action: "account.delete" },
  );
}

/** A `/me` probe: a 401 throws `ApiError{status: 401}`, never redirects. */
export async function fetchMeProfile(): Promise<MeProfile> {
  return request("/api/identity/me", undefined, {
    action: "me.load",
    onUnauthorized: "throw",
  });
}

export async function updateDisplayName(
  displayName: string,
): Promise<MeProfile> {
  return request(
    "/api/identity/me/profile",
    { method: "PATCH", body: JSON.stringify({ display_name: displayName }) },
    { action: "profile.name" },
  );
}

export async function uploadAvatar(file: File): Promise<void> {
  const form = new FormData();
  form.append("file", file);
  await request(
    "/api/identity/me/avatar",
    { method: "PUT", body: form },
    { action: "profile.avatar" },
  );
}

export async function removeAvatar(): Promise<void> {
  await request(
    "/api/identity/me/avatar",
    { method: "DELETE" },
    { action: "profile.avatar.remove" },
  );
}

export type LoginResult =
  { ok: true } | { ok: false; throttled: true; retryAfterSeconds: number };

/**
 * Signs in. 429 resolves as throttled: `Retry-After` seconds, else the body's
 * `retry_after`, at least 1. A 401 (wrong, unknown, or disabled user) and
 * every other failure throw `ApiError` with action "login"; never redirects.
 */
export async function login(
  email: string,
  password: string,
): Promise<LoginResult> {
  const response = await send(
    "/api/identity/login",
    { method: "POST", body: JSON.stringify({ email, password }) },
    "login",
  );
  if (response.ok) return { ok: true };
  if (response.status !== 429) throw await apiErrorFrom(response, "login");
  const header = Number.parseFloat(response.headers.get("retry-after") ?? "");
  const { data } = await readErrorBody(response);
  const body = (data as { retry_after?: unknown } | undefined)?.retry_after;
  const seconds = Number.isFinite(header)
    ? header
    : typeof body === "number" && Number.isFinite(body)
      ? body
      : 1;
  return {
    ok: false,
    throttled: true,
    retryAfterSeconds: Math.max(1, Math.ceil(seconds)),
  };
}

/**
 * Ends the session. 204 resolves `{alreadySignedOut: false}`; 401 (no valid
 * session) resolves `{alreadySignedOut: true}`; others throw `ApiError`.
 * Never navigates: the caller calls `goToLogin()`.
 */
export async function logout(): Promise<{ alreadySignedOut: boolean }> {
  try {
    await request(
      "/api/identity/logout",
      { method: "POST" },
      { action: "logout", onUnauthorized: "throw" },
    );
    return { alreadySignedOut: false };
  } catch (error) {
    if (error instanceof ApiError && error.status === 401)
      return { alreadySignedOut: true };
    throw error;
  }
}

/** Sets a new password (min 8 chars); returns the user. */
export async function changePassword(password: string): Promise<MeProfile> {
  return request(
    "/api/identity/me/password",
    { method: "POST", body: JSON.stringify({ password }) },
    { action: "password.change" },
  );
}

export async function fetchSetupStatus(): Promise<{ setup_required: boolean }> {
  return request("/api/identity/setup-status", undefined, {
    action: "setup.status",
    onUnauthorized: "throw",
  });
}

/**
 * Creates the first admin. The secret travels only in the `x-setup-secret`
 * header, as the server expects; it is never stored or logged here.
 */
export async function completeSetup(
  secret: string,
  email: string,
  password: string,
): Promise<void> {
  await request(
    "/api/identity/setup",
    {
      method: "POST",
      headers: { "x-setup-secret": secret },
      body: JSON.stringify({ email, password }),
    },
    { action: "setup", onUnauthorized: "throw" },
  );
}

/** Same-origin avatar image; `version` busts the browser cache on change. */
export const avatarUrl = (id: number, version: string | number) =>
  `/api/identity/avatar/${id}?v=${encodeURIComponent(version)}`;
