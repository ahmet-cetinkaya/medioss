import { describe, expect, test } from "bun:test";
import { groupSoundAttributions, soundGroups } from "./third-party-sounds";

describe("groupSoundAttributions", () => {
  test("groups by category and links each sound to its upstream file", () => {
    const groups = groupSoundAttributions([
      {
        category_name: "Animals",
        name: "cat-purring",
        source:
          "remvze/moodist@main public/sounds/animals/cat-purring.mp3 (MP3→Opus)",
      },
      { category_name: "Animals", name: "birds", source: "unknown origin" },
    ]);
    expect(groups).toEqual([
      {
        category: "Animals",
        sounds: [
          {
            title: "cat purring",
            sourceUrl:
              "https://github.com/remvze/moodist/blob/main/public/sounds/animals/cat-purring.mp3",
          },
          { title: "birds", sourceUrl: null },
        ],
      },
    ]);
  });

  test("covers every bundled sound in the real manifest", () => {
    const total = soundGroups.reduce((sum, g) => sum + g.sounds.length, 0);
    expect(total).toBeGreaterThan(0);
    expect(soundGroups.flatMap((g) => g.sounds).every((s) => s.sourceUrl)).toBe(
      true,
    );
  });
});
