import { describe, expect, test } from "bun:test";
import { loginDestination } from "./login-destination";

const ORIGIN = "http://localhost:3000";

describe("loginDestination", () => {
  test("no next goes Home", () => {
    expect(loginDestination("", ORIGIN)).toBe("/");
    expect(loginDestination("?other=1", ORIGIN)).toBe("/");
  });

  test("a screen address with its query is kept", () => {
    expect(loginDestination("?next=%2Fstats%2F", ORIGIN)).toBe("/stats/");
    expect(loginDestination("?next=/player/?id=1", ORIGIN)).toBe(
      "/player/?id=1",
    );
    expect(loginDestination("?next=%2Fplayer%3Fid%3D7", ORIGIN)).toBe(
      "/player/?id=7",
    );
  });

  test("foreign, protocol-relative, and non-screen addresses go Home", () => {
    for (const next of [
      "//evil.com",
      "%2F%2Fevil.com",
      "https://evil.com/",
      "/\\evil.com",
      "/login/",
      "/setup/",
      "/api/identity/me",
      "javascript:void(0)",
    ])
      expect(loginDestination(`?next=${next}`, ORIGIN)).toBe("/");
  });

  test("the value is decoded once only", () => {
    expect(loginDestination("?next=%252F%252Fevil.com", ORIGIN)).toBe("/");
  });
});
