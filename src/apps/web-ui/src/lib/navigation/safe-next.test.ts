import { describe, expect, test } from "bun:test";
import { safeNext } from "./safe-next";

const ORIGIN = "http://localhost:3000";

describe("safeNext", () => {
  test.each([
    ["/", "/"],
    ["/library/?q=rain", "/library/?q=rain"],
    ["/library?q=rain", "/library/?q=rain"],
    ["/player/?id=5", "/player/?id=5"],
    ["/player?id=1", "/player/?id=1"],
    ["/admin/?section=soundscape", "/admin/?section=soundscape"],
    ["/player/5/", "/player/5/"],
    ["/player/5", "/player/5/"],
    ["/player/5/?t=3", "/player/5/?t=3"],
    ["/admin/meditations/", "/admin/meditations/"],
    ["/admin/meditations/12/", "/admin/meditations/12/"],
    ["/admin/soundscape/", "/admin/soundscape/"],
    ["/./library/", "/library/"],
    ["/%2e%2e/admin/", "/admin/"],
    ["/library/../admin/", "/admin/"],
  ])("accepts %p as %p", (raw, expected) => {
    expect(safeNext(raw, ORIGIN)).toBe(expected);
  });

  test.each([
    ["//evil.com"],
    ["/\\evil.com"],
    ["/\t/evil.com"],
    ["/\\t/evil.com"],
    ["/%2F%2Fevil.com"],
    ["https://evil.com/"],
    ["javascript:alert(1)"],
    ["/login/"],
    ["/login"],
    ["/setup/"],
    ["/api/meditation/1/download"],
    ["/_astro/x.js"],
    ["/unknown/"],
    ["/player/abc/"],
    ["/player/5/extra/"],
    ["/admin/meditations/12/extra/"],
    ["/player/../../../etc/"],
    [""],
    [`/library/?q=${"a".repeat(2049 - "/library/?q=".length)}`],
    [" /library/"],
    ["/library/\u0000"],
    ["/library/\u007f"],
    ["/library/?q=a b"],
    ["/library/?q=a\u00a0b"],
    ["/%5C"],
    ["/library/..%2F"],
    ["/ "],
    ["/\ufeff"],
  ])("rejects %p", (raw) => {
    expect(safeNext(raw, ORIGIN)).toBeNull();
  });

  test("rejects null", () => {
    expect(safeNext(null, ORIGIN)).toBeNull();
  });

  test("the 2049-char case is exactly over the limit", () => {
    const at = `/library/?q=${"a".repeat(2048 - "/library/?q=".length)}`;
    expect(at).toHaveLength(2048);
    expect(safeNext(at, ORIGIN)).toBe(at);
    expect(safeNext(`${at}a`, ORIGIN)).toBeNull();
  });

  test("drops the hash and never echoes a non-canonical raw input", () => {
    const raw = "/library?q=rain#top";
    const result = safeNext(raw, ORIGIN);
    expect(result).toBe("/library/?q=rain");
    expect(result).not.toContain(raw);
    expect(result).not.toContain("#");
  });
});
