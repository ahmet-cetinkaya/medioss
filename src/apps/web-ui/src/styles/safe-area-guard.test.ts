import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) =>
  readFileSync(new URL(path, import.meta.url), "utf8");

// The bottom bar ignores the browser's safe-area inset (see --safe-area-bottom
// in global.css). That is only right while the page does not draw edge to edge.
describe("--safe-area-bottom", () => {
  test("is 0px unless the page opts in to viewport-fit=cover", () => {
    const optsIn = /viewport-fit\s*=\s*cover/.test(
      read("../layouts/Base.astro"),
    );
    const token = /--safe-area-bottom:\s*([^;]+);/
      .exec(read("./global.css"))?.[1]
      ?.trim();
    expect(token).toBeDefined();
    if (optsIn) expect(token).toBe("env(safe-area-inset-bottom)");
    else expect(token).toBe("0px");
  });

  test("nothing else reads env(safe-area-inset-bottom) directly", () => {
    const layout = read("../layouts/AppLayout.astro");
    expect(layout).not.toContain("env(safe-area-inset-bottom)");
  });
});
