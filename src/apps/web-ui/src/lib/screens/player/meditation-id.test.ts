import { expect, test } from "bun:test";
import { parseMeditationId } from "./meditation-id";

test("accepts a positive integer id", () => {
  expect(parseMeditationId("?id=7")).toBe(7);
  expect(parseMeditationId("?x=1&id=12")).toBe(12);
});

test("rejects a missing, non-numeric, zero, or unsafe id (PB-18)", () => {
  for (const search of [
    "",
    "?id=",
    "?id=abc",
    "?id=0",
    "?id=-3",
    "?id=1.5",
    "?id=1e3",
    "?id=%201",
    "?id=99999999999999999999",
  ])
    expect(parseMeditationId(search)).toBeNull();
});

test("reads the id from the /player/<id>/ path, legacy ?id= as fallback", () => {
  expect(parseMeditationId("", "/player/7/")).toBe(7);
  expect(parseMeditationId("", "/player/7")).toBe(7);
  expect(parseMeditationId("?id=3", "/player/")).toBe(3);
  expect(parseMeditationId("", "/player/")).toBeNull();
  expect(parseMeditationId("", "/player/abc/")).toBeNull();
  expect(parseMeditationId("", "/player/0/")).toBeNull();
});
