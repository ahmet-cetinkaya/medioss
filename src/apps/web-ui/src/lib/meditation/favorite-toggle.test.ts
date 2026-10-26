import { describe, expect, test } from "bun:test";
import { toggleFavoriteOptimistically } from "./favorite-toggle";

describe("toggleFavoriteOptimistically", () => {
  test("applies the new state before the server answers and keeps it", async () => {
    const shown: boolean[] = [];
    const sent: boolean[] = [];
    await toggleFavoriteOptimistically(
      false,
      (value) => shown.push(value),
      async (next) => void sent.push(next),
    );
    expect(shown).toEqual([true]);
    expect(sent).toEqual([true]);
  });

  test("rolls back and rethrows when the server call fails", async () => {
    const shown: boolean[] = [];
    const failure = new Error("refused");
    await expect(
      toggleFavoriteOptimistically(
        true,
        (value) => shown.push(value),
        async () => {
          throw failure;
        },
      ),
    ).rejects.toBe(failure);
    expect(shown).toEqual([false, true]);
  });
});
