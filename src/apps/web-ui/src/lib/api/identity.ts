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
