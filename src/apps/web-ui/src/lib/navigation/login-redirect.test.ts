import { beforeEach, describe, expect, test } from "bun:test";
import {
  goToLogin,
  type LocationLike,
  redirectToLogin,
  resetLoginRedirectForTests,
} from "./login-redirect";

function stubLocation(pathname: string, search = "") {
  const calls: string[] = [];
  const loc: LocationLike = {
    pathname,
    search,
    replace: (url) => calls.push(url),
  };
  return { loc, calls };
}

beforeEach(resetLoginRedirectForTests);

describe("redirectToLogin", () => {
  test("replaces once per page with next = pathname + search", () => {
    const { loc, calls } = stubLocation("/player/", "?id=5&t=1");
    expect(redirectToLogin(loc)).toBe(true);
    expect(redirectToLogin(loc)).toBe(true);
    expect(calls).toEqual([
      `/login/?next=${encodeURIComponent("/player/?id=5&t=1")}`,
    ]);
  });

  test("accepts the bare path form", () => {
    const { loc, calls } = stubLocation("/library", "?q=rain");
    redirectToLogin(loc);
    expect(calls).toEqual(["/login/?next=%2Flibrary%3Fq%3Drain"]);
  });

  test.each(["/login/", "/login", "/setup/", "/setup"])(
    "never fires on %p",
    (pathname) => {
      const { loc, calls } = stubLocation(pathname, "?next=%2F");
      expect(redirectToLogin(loc)).toBe(false);
      expect(calls).toEqual([]);
    },
  );

  test("omits next for a path off the allowlist", () => {
    const { loc, calls } = stubLocation("/unknown/", "?x=1");
    redirectToLogin(loc);
    expect(calls).toEqual(["/login/"]);
  });

  test.each([
    ["/player/5/", "?t=3"],
    ["/admin/meditations/12/", ""],
    ["/admin/soundscape/", ""],
  ])("remembers the canonical deep path %p%s", (pathname, search) => {
    const { loc, calls } = stubLocation(pathname, search);
    redirectToLogin(loc);
    expect(calls).toEqual([
      `/login/?next=${encodeURIComponent(pathname + search)}`,
    ]);
  });

  test.each([["/player/abc/"], ["/player/5/extra/"]])(
    "omits next for a malformed deep path %p",
    (pathname) => {
      const { loc, calls } = stubLocation(pathname);
      redirectToLogin(loc);
      expect(calls).toEqual(["/login/"]);
    },
  );
});

describe("goToLogin", () => {
  test("replaces with /login/ and no next, blocking a later redirect", () => {
    const { loc, calls } = stubLocation("/settings/");
    goToLogin(loc);
    expect(redirectToLogin(loc)).toBe(true);
    expect(calls).toEqual(["/login/"]);
  });
});
