import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const controls = readFileSync(
  new URL("./player-controls.svelte", import.meta.url),
  "utf8",
);
const cover = readFileSync(
  new URL("./player-cover.svelte", import.meta.url),
  "utf8",
);

const handlerBody = (name: string): string => {
  const start = controls.indexOf(`function ${name}(`);
  return controls.slice(start, controls.indexOf("\n  }\n", start));
};

describe("end of the meditation", () => {
  test("the background mix keeps playing when the audio ends", () => {
    expect(handlerBody("onEnded")).not.toContain("mix.pause");
  });

  test("the mix still pauses on a user pause, but not on the pause fired before ended", () => {
    expect(handlerBody("onPause")).toMatch(/if \(!reachedEnd\) mix\.pause\(\)/);
  });
});

describe("floating player cover", () => {
  test("shows no round cover: the program cover is already the backdrop", () => {
    const floating = cover.slice(
      cover.indexOf("{#if playing}"),
      cover.indexOf("{:else}"),
    );
    expect(floating).not.toContain("rounded-full");
    expect(floating).not.toContain("artwork(");
  });
});
