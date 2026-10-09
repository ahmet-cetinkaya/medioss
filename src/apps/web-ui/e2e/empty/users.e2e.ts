// `empty` project: admin plus member and no content (SC-007).
import { expect, test } from "@playwright/test";
import { getJson, healthStatus, newApi } from "../support/api-client";
import {
  ensureUsers,
  loginAs,
  saveStorageState,
  USERS,
  type UserDto,
} from "../support/auth";

test("health endpoint is ok", async () => {
  const api = await newApi();
  expect(await healthStatus(api)).toEqual({ status: "ok" });
  await api.dispose();
});

test("admin and member exist and sign in; storage state saved per role", async () => {
  const { admin, member } = await ensureUsers();
  expect(admin).toMatchObject({
    email: USERS.admin.email,
    role: "admin",
    must_change_password: false,
  });
  expect(member).toMatchObject({
    email: USERS.member.email,
    role: "member",
    must_change_password: false,
  });
  expect(member.display_name).toBeNull();
  await saveStorageState("admin");
  await saveStorageState("member");
});

test("instance has no content yet", async () => {
  const member = await loginAs("member");
  expect(await getJson<unknown[]>(member, "/api/meditation/programs")).toEqual(
    [],
  );
  expect(await getJson<unknown[]>(member, "/api/meditation/library")).toEqual(
    [],
  );
  expect(
    await getJson<unknown[]>(member, "/api/meditation/categories"),
  ).toEqual([]);
  expect(await getJson<unknown[]>(member, "/api/soundscape/mixes")).toEqual([]);
  expect(
    (await getJson<{ items: unknown[] }>(member, "/api/mood?limit=1")).items,
  ).toEqual([]);
  await member.dispose();

  const admin = await loginAs("admin");
  const users = await getJson<UserDto[]>(admin, "/api/identity/users");
  expect(users.map((user) => user.email).sort()).toEqual([
    USERS.admin.email,
    USERS.member.email,
  ]);
  await admin.dispose();
});
