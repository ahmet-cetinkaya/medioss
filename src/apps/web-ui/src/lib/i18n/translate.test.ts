import { describe, expect, mock, test } from "bun:test";
import { defineMessages, type Message } from "./message";
import { t, translate } from "./translate";

const plural = (locale: string) => new Intl.PluralRules(locale);

const messages = defineMessages({
  greeting: { en: "Hello, {name}", tr: "Merhaba, {name}" },
  count: {
    en: ({ count }: { count: number }) =>
      `${count} ${plural("en").select(count) === "one" ? "session" : "sessions"}`,
    tr: ({ count }: { count: number }) => `${count} seans`,
  },
});

describe("translate", () => {
  test("returns the requested locale with params", () => {
    expect(translate(messages.greeting, "en", { name: "Ada" })).toBe(
      "Hello, Ada",
    );
    expect(translate(messages.greeting, "tr", { name: "Ada" })).toBe(
      "Merhaba, Ada",
    );
  });

  test("missing or empty Turkish falls back to English (FR-119)", () => {
    const missing = { en: "Save" } as unknown as Message;
    const empty: Message = { en: "Save", tr: "" };
    const emptyFunction: Message = { en: () => "Saved", tr: () => "" };
    expect(translate(missing, "tr")).toBe("Save");
    expect(translate(empty, "tr")).toBe("Save");
    expect(translate(emptyFunction, "tr")).toBe("Saved");
  });

  test("unknown params are left intact", () => {
    expect(translate(messages.greeting, "en", { other: 1 })).toBe(
      "Hello, {name}",
    );
    expect(translate(messages.greeting, "en")).toBe("Hello, {name}");
    expect(
      translate({ en: "{a} and {b}", tr: "{a} ve {b}" }, "tr", { a: "x" }),
    ).toBe("x ve {b}");
  });

  test("never returns an id or property path", () => {
    const broken = {} as unknown as Message;
    for (const locale of ["en", "tr"] as const) {
      const text = translate(broken, locale);
      expect(text).not.toMatch(/^[a-z]+(\.[a-zA-Z0-9]+)+$/);
      expect(text).toBe("");
    }
    expect(translate(undefined as unknown as Message, "tr")).toBe("");
  });

  test("the function form is called with params", () => {
    const en = mock((params: { count: number }) => `en ${params.count}`);
    const tr = mock((params: { count: number }) => `tr ${params.count}`);
    expect(translate({ en, tr }, "tr", { count: 3 })).toBe("tr 3");
    expect(tr).toHaveBeenCalledWith({ count: 3 });
    expect(en).not.toHaveBeenCalled();
    expect(translate(messages.count, "en", { count: 1 })).toBe("1 session");
    expect(translate(messages.count, "en", { count: 2 })).toBe("2 sessions");
  });

  test("t() renders English until a locale getter is registered", () => {
    expect(t(messages.greeting, { name: "Ada" })).toBe("Hello, Ada");
  });
});
