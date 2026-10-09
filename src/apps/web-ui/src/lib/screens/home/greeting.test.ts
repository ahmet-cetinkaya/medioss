import { describe, expect, test } from "bun:test";
import { greeting, greetingBand } from "./greeting";

const at = (hours: number, minutes: number) =>
  new Date(2026, 9, 4, hours, minutes);

describe("greetingBand", () => {
  test("switches at 05:00, 12:00 and 18:00 (the later band applies)", () => {
    expect(greeting(at(4, 59), null).band).toBe("evening");
    expect(greeting(at(5, 0), null).band).toBe("morning");
    expect(greeting(at(11, 59), null).band).toBe("morning");
    expect(greeting(at(12, 0), null).band).toBe("afternoon");
    expect(greeting(at(17, 59), null).band).toBe("afternoon");
    expect(greeting(at(18, 0), null).band).toBe("evening");
    expect(greeting(at(0, 0), null).band).toBe("evening");
    expect(greeting(at(23, 59), null).band).toBe("evening");
  });

  test("covers every hour", () => {
    const bands = Array.from({ length: 24 }, (_, hour) => greetingBand(hour));
    expect(bands.filter((band) => band === "morning")).toHaveLength(7);
    expect(bands.filter((band) => band === "afternoon")).toHaveLength(6);
    expect(bands.filter((band) => band === "evening")).toHaveLength(11);
  });
});

describe("greeting name", () => {
  const user = {
    email: "ayse@example.test",
    display_name: "Ayşe" as string | null,
  };

  test("includes the display name as entered", () => {
    expect(greeting(at(8, 0), user).name).toBe("Ayşe");
    expect(greeting(at(8, 0), { display_name: "ayşe Yılmaz" }).name).toBe(
      "ayşe Yılmaz",
    );
  });

  test("is absent without a display name, and never the email", () => {
    for (const display_name of [null, "", "   "]) {
      const result = greeting(at(8, 0), { ...user, display_name });
      expect(result.name).toBeNull();
      expect(JSON.stringify(result)).not.toContain(user.email);
    }
    expect(greeting(at(8, 0), null).name).toBeNull();
  });
});
