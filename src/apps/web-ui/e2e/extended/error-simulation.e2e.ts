// T25: error simulation (SC-005). A 404, a 500, and an aborted request on each
// screen's primary data route show an error or not-found state, never a login
// redirect; the URL stays on the screen.
import type { Page } from "@playwright/test";
import { storageStatePath, type Role } from "../support/auth";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";

type Failure = "404" | "500" | "abort";
const FAILURES: readonly Failure[] = ["404", "500", "abort"];

type Case = {
  name: string;
  role: Role;
  path: () => string;
  /** Pathnames whose GET is failed. */
  routes: () => (string | RegExp)[];
};

const seedProgram = () => readSeed().programs.withCover;

const CASES: readonly Case[] = [
  {
    name: "Player",
    role: "member",
    path: () => `/player/?id=${seedProgram().meditations[0].id}`,
    routes: () => [/^\/api\/meditation\/\d+$/],
  },
  {
    name: "Admin",
    role: "admin",
    path: () => "/admin/",
    routes: () => ["/api/identity/users"],
  },
  {
    name: "Library",
    role: "member",
    path: () => "/library/",
    routes: () => ["/api/meditation/programs"],
  },
  {
    name: "Program",
    role: "member",
    path: () => `/program/?id=${seedProgram().id}`,
    routes: () => [`/api/meditation/programs/${seedProgram().id}`],
  },
  {
    name: "Program members",
    role: "member",
    path: () => `/program/?id=${seedProgram().id}`,
    routes: () => [`/api/meditation/programs/${seedProgram().id}/members`],
  },
  {
    name: "Mood list",
    role: "member",
    path: () => "/mood/",
    routes: () => ["/api/mood"],
  },
  {
    name: "Mood statistics",
    role: "member",
    path: () => "/mood/",
    routes: () => ["/api/mood/statistics"],
  },
  {
    name: "History",
    role: "member",
    path: () => "/history/",
    routes: () => ["/api/timeline"],
  },
  {
    name: "Statistics",
    role: "member",
    path: () => "/stats/",
    routes: () => ["/api/meditation/statistics"],
  },
];

const matches = (route: string | RegExp, pathname: string) =>
  typeof route === "string" ? route === pathname : route.test(pathname);

async function fail(page: Page, c: Case, failure: Failure): Promise<void> {
  await page.route(
    (url) => c.routes().some((r) => matches(r, url.pathname)),
    (route) =>
      route.request().method() !== "GET"
        ? route.continue()
        : failure === "abort"
          ? route.abort("failed")
          : route.fulfill({
              status: Number(failure),
              json: { error: "simulated" },
            }),
  );
}

for (const role of ["member", "admin"] as const) {
  test.describe(`${role} screens`, () => {
    test.use({ storageState: storageStatePath(role) });

    for (const c of CASES.filter((x) => x.role === role)) {
      for (const failure of FAILURES) {
        test(`${c.name} ${failure} shows an error state, not login (SC-005)`, async ({
          page,
        }) => {
          await fail(page, c, failure);
          const target = new URL(c.path(), "http://x");
          await page.goto(c.path());
          // An error or not-found state: a Retry action, an alert, or a
          // "not found" heading. Anything else (blank, login) fails here.
          const state = page
            .getByRole("button", { name: "Retry" })
            .or(page.getByRole("alert"))
            .or(page.getByRole("heading", { name: /not found/i }));
          await expect(state.first()).toBeVisible();
          await expect(page.locator("main")).not.toContainText("simulated");
          await expect(page.locator("main")).toBeVisible();
          expect(new URL(page.url()).pathname).toBe(target.pathname);
        });
      }
    }
  });
}
