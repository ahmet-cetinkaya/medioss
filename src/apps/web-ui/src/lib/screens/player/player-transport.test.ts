import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

// The Svelte components have no render harness (bun test only), so these read
// the source: they pin the control order and names the human asked for.
const transport = readFileSync(
  new URL("./player-transport.svelte", import.meta.url),
  "utf8",
);

const position = (needle: string) => {
  const index = transport.indexOf(needle);
  expect(index).toBeGreaterThan(-1);
  return index;
};

describe("player transport rows", () => {
  test("row 1 is back, play/pause, forward; row 2 is restart, speed, soundscape", () => {
    const order = [
      "m.skipBack",
      "actionMessages.pause",
      "m.skipForward",
      "m.restart",
      "m.speedButton",
      "m.mixToggle",
    ].map(position);
    expect(order).toEqual([...order].sort((a, b) => a - b));
  });

  test("the soundscape button is icon-only and keeps its accessible name", () => {
    expect(transport).toContain('icon="speaker-high"');
    expect(transport).toMatch(
      /label=\{mixPlaying[\s\S]*?m\.mixToggle[\s\S]*?m\.mixPlaying/,
    );
    expect(transport).not.toMatch(
      /<\/?IconButton[^>]*>\s*\{t\(m\.mixToggle\)\}/,
    );
  });
});
