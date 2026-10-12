import { describe, expect, test } from "bun:test";
import {
  meditationProgressLabel,
  programProgressLabel,
} from "./progress-label";

// Legacy formulas (src/lib/api/meditation.ts):
//   program:    percent = Math.round(progress * 100);
//               progress >= 1 ? "Completed" : `${percent}% complete`
//   meditation: completed ? "Completed" : `${progress_percent}%`
// So 0.004 rounds to 0 and 0.999 rounds to 100 but is not yet "Completed".

describe("programProgressLabel", () => {
  const cases: [number, string, string][] = [
    [0, "0% complete", "%0 tamamlandı"],
    [0.004, "0% complete", "%0 tamamlandı"],
    [0.5, "50% complete", "%50 tamamlandı"],
    [0.999, "100% complete", "%100 tamamlandı"],
    [1, "Completed", "Tamamlandı"],
  ];

  for (const [progress, en, tr] of cases) {
    test(`${progress} in en and tr`, () => {
      expect(programProgressLabel(progress, "en")).toBe(en);
      expect(programProgressLabel(progress, "tr")).toBe(tr);
    });
  }

  test("returns null while loading", () => {
    expect(programProgressLabel(null, "en")).toBeNull();
    expect(programProgressLabel(undefined, "tr")).toBeNull();
  });
});

describe("meditationProgressLabel", () => {
  test("percent in en and tr", () => {
    const m = { completed: false, progress_percent: 40 };
    expect(meditationProgressLabel(m, "en")).toBe("40%");
    expect(meditationProgressLabel(m, "tr")).toBe("%40");
    const zero = { completed: false, progress_percent: 0 };
    expect(meditationProgressLabel(zero, "en")).toBe("0%");
    expect(meditationProgressLabel(zero, "tr")).toBe("%0");
  });

  test("completed in en and tr", () => {
    const m = { completed: true, progress_percent: 100 };
    expect(meditationProgressLabel(m, "en")).toBe("Completed");
    expect(meditationProgressLabel(m, "tr")).toBe("Tamamlandı");
  });

  test("returns null while loading", () => {
    expect(meditationProgressLabel(null, "en")).toBeNull();
    expect(meditationProgressLabel(undefined, "tr")).toBeNull();
  });
});
