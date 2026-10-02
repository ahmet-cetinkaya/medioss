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
