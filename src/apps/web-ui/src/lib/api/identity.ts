/**
 * Typed client for the identity API. Every call is same-origin with
 * session cookies; JSON errors surface as thrown Error values.
 */

import { request } from "./request";

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
  return request("/api/identity/users");
}

export async function createUser(
  email: string,
  password: string,
): Promise<{ id: number; email: string; role: string }> {
  return request("/api/identity/users", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
}

export async function setUserStatus(
  id: number,
  disabled: boolean,
): Promise<void> {
  await request(`/api/identity/users/${id}/status`, {
    method: "PATCH",
    body: JSON.stringify({ disabled }),
  });
}

export async function deleteUser(id: number): Promise<void> {
  await request(`/api/identity/users/${id}`, { method: "DELETE" });
}

export async function deleteOwnAccount(): Promise<void> {
  await request("/api/identity/me", { method: "DELETE" });
}

export async function fetchMeProfile(): Promise<MeProfile> {
  return request("/api/identity/me");
}

export async function updateDisplayName(
  displayName: string,
): Promise<MeProfile> {
  return request("/api/identity/me/profile", {
    method: "PATCH",
    body: JSON.stringify({ display_name: displayName }),
  });
}

export async function uploadAvatar(file: File): Promise<void> {
  const form = new FormData();
  form.append("file", file);
  await request("/api/identity/me/avatar", {
    method: "PUT",
    body: form,
  });
}

export async function removeAvatar(): Promise<void> {
  await request("/api/identity/me/avatar", { method: "DELETE" });
}
