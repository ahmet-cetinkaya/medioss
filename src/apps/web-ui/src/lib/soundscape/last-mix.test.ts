import { describe, expect, test } from "bun:test";
import { readLastMixId, writeLastMixId, type LastMixStorage } from "./last-mix";

const memory = (): LastMixStorage & { data: Map<string, string> } => {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => void data.set(key, value),
  };
};

const throwing: LastMixStorage = {
  getItem: () => {
    throw new Error("denied");
  },
  setItem: () => {
    throw new Error("denied");
  },
};

describe("last mix", () => {
  test("round-trips under medioss.lastMix", () => {
    const storage = memory();
    expect(readLastMixId(storage)).toBeNull();
    writeLastMixId(7, storage);
    expect(storage.data.get("medioss.lastMix")).toBe("7");
    expect(readLastMixId(storage)).toBe(7);
  });

  test("malformed values read as null", () => {
    for (const bad of ["", "abc", "-1", "0", "1.5"]) {
      const storage = memory();
      storage.setItem("medioss.lastMix", bad);
      expect(readLastMixId(storage)).toBeNull();
    }
  });

  test("throwing storage neither throws nor remembers", () => {
    expect(readLastMixId(throwing)).toBeNull();
    expect(() => writeLastMixId(3, throwing)).not.toThrow();
  });

  test("missing global storage is silent", () => {
    expect(readLastMixId()).toBeNull();
    expect(() => writeLastMixId(3)).not.toThrow();
  });
});
