// Test-only users, their credentials, and per-role storage state.
// These credentials exist only on throwaway QA data dirs under .agents/qa/.
import fs from "node:fs";
import path from "node:path";
import { expectOk, getJson, newApi, postJson, type Api } from "./api-client";
import { authDir, baseURL, sessionSecret } from "./server-env";

export const USERS = {
  admin: {
    email: "admin@example.test",
    password: "admin-e2e-pass-1",
    displayName: "Ayşe Admin",
  },
  member: {
    email: "member@example.test",
    temporaryPassword: "member-e2e-temp-1",
    password: "member-e2e-pass-1",
  },
  flagged: {
    email: "flagged@example.test",
    temporaryPassword: "flagged-e2e-temp-1",
  },
} as const;

export type Role = "admin" | "member";

export type UserDto = {
  id: number;
  email: string;
  role: "admin" | "member";
  must_change_password: boolean;
  display_name: string | null;
  has_avatar: boolean;
};

export const storageStatePath = (role: Role): string =>
  path.join(authDir(), `${role}.json`);

/**
 * Registers the initial admin. Uses Node's fetch instead of Playwright's request
 * context on purpose: Playwright call logs echo request headers into error
 * output, and this request carries the setup secret header.
 */
export async function setupAdmin(): Promise<UserDto> {
  const response = await fetch(`${baseURL()}/api/identity/setup`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      origin: new URL(baseURL()).origin,
      "x-setup-secret": sessionSecret(),
    },
    body: JSON.stringify({
      email: USERS.admin.email,
      password: USERS.admin.password,
    }),
  });
  if (response.status !== 201) {
    throw new Error(
      `setup failed: HTTP ${response.status} ${(await response.text()).slice(0, 200)}`,
    );
  }
  return (await response.json()) as UserDto;
}

/** Signs in through the API; the returned context carries the session cookie. */
export async function loginApi(email: string, password: string): Promise<Api> {
  const api = await newApi();
  await expectOk(
    await api.post("/api/identity/login", { data: { email, password } }),
    `login ${email}`,
  );
  return api;
}

export async function loginAs(role: Role): Promise<Api> {
  return loginApi(USERS[role].email, USERS[role].password);
}

/** Logs in as `role` and writes its storage state for browser contexts. */
export async function saveStorageState(role: Role): Promise<string> {
  const api = await loginAs(role);
  fs.mkdirSync(authDir(), { recursive: true });
  const file = storageStatePath(role);
  await api.storageState({ path: file });
  await api.dispose();
  return file;
}

/**
 * Idempotent on a fresh instance: sets up the admin when needed (or reuses one
 * a `fresh` UI test created with the same credentials), then makes sure the
 * member exists with its permanent password. Returns both users.
 */
export async function ensureUsers(): Promise<{
  admin: UserDto;
  member: UserDto;
}> {
  const anonymous = await newApi();
  const status = await getJson<{ setup_required: boolean }>(
    anonymous,
    "/api/identity/setup-status",
  );
  await anonymous.dispose();
  if (status.setup_required) await setupAdmin();

  const admin = await loginAs("admin").catch((error: Error) => {
    throw new Error(
      `admin login failed; the instance was not set up by this harness (${error.message})`,
    );
  });
  await expectOk(
    await admin.patch("/api/identity/me/profile", {
      data: { display_name: USERS.admin.displayName },
    }),
    "set admin display name",
  );
  const adminUser = await getJson<UserDto>(admin, "/api/identity/me");
  const users = await getJson<UserDto[]>(admin, "/api/identity/users");

  if (!users.some((user) => user.email === USERS.member.email)) {
    await postJson(admin, "/api/identity/users", {
      email: USERS.member.email,
      password: USERS.member.temporaryPassword,
    });
    const first = await loginApi(
      USERS.member.email,
      USERS.member.temporaryPassword,
    );
    await postJson(first, "/api/identity/me/password", {
      password: USERS.member.password,
    });
    await first.dispose();
  }
  await admin.dispose();

  const member = await loginAs("member");
  const memberUser = await getJson<UserDto>(member, "/api/identity/me");
  await member.dispose();
  return { admin: adminUser, member: memberUser };
}
