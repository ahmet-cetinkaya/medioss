import { describe, expect, mock, test } from "bun:test";
import { resolveReturnFocus, restoreFocus } from "./return-focus";

const element = (isConnected: boolean) => ({ isConnected, focus: mock() });

describe("resolveReturnFocus", () => {
  test("a connected trigger wins", () => {
    const trigger = element(true);
    expect(resolveReturnFocus(trigger, element(true))).toBe(trigger);
  });

  test("a removed trigger falls back", () => {
    const fallback = element(true);
    expect(resolveReturnFocus(element(false), fallback)).toBe(fallback);
  });

  test("no fallback gives null", () => {
    expect(resolveReturnFocus(element(false))).toBeNull();
    expect(resolveReturnFocus(element(false), null)).toBeNull();
    expect(resolveReturnFocus(null)).toBeNull();
  });

  test("a removed fallback gives null", () => {
    expect(resolveReturnFocus(element(false), element(false))).toBeNull();
  });
});

describe("restoreFocus", () => {
  test("focuses the target and cancels the default", () => {
    const event = { preventDefault: mock() };
    const fallback = element(true);
    restoreFocus(event, element(false), () => fallback);
    expect(fallback.focus).toHaveBeenCalledTimes(1);
    expect(event.preventDefault).toHaveBeenCalledTimes(1);
  });

  test("keeps the default when there is no target", () => {
    const event = { preventDefault: mock() };
    restoreFocus(event, null, () => null);
    expect(event.preventDefault).not.toHaveBeenCalled();
  });
});
