import { describe, expect, test } from "bun:test";
import { randomMix } from "./random-mix";

describe("randomMix", () => {
  test("empty library gives an empty mix", () => {
    expect(randomMix([])).toEqual([]);
  });

  test("fewer than 3 sounds gives all of them", () => {
    expect(
      randomMix(["a", "b"])
        .map((e) => e.sound)
        .sort(),
    ).toEqual(["a", "b"]);
  });

  test("picks 3 distinct sounds at volume 50", () => {
    const sounds = ["a", "b", "c", "d", "e"];
    for (let i = 0; i < 50; i += 1) {
      const mix = randomMix(sounds);
      expect(mix).toHaveLength(3);
      expect(new Set(mix.map((e) => e.sound)).size).toBe(3);
      expect(mix.every((e) => e.volume === 50)).toBe(true);
    }
  });

  test("is driven by rand, even at the 1 edge", () => {
    expect(
      randomMix(["a", "b", "c", "d"], () => 0).map((e) => e.sound),
    ).toEqual(["a", "b", "c"]);
    expect(
      randomMix(["a", "b", "c", "d"], () => 0.999999).map((e) => e.sound),
    ).toEqual(["d", "c", "b"]);
    expect(randomMix(["a"], () => 1).map((e) => e.sound)).toEqual(["a"]);
  });
});
