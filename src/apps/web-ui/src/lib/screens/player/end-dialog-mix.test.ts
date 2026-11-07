import { describe, expect, test } from "bun:test";
import { createDialogCloseWatcher } from "./end-dialog-mix";

const watcherWithCounter = () => {
  const calls = { stops: 0 };
  const watch = createDialogCloseWatcher(() => calls.stops++);
  return { calls, watch };
};

describe("createDialogCloseWatcher", () => {
  test("does not stop the mix while the dialog is open", () => {
    const { calls, watch } = watcherWithCounter();
    watch(false);
    watch(true);
    watch(true);
    expect(calls.stops).toBe(0);
  });

  test("stops the mix once when the dialog closes", () => {
    const { calls, watch } = watcherWithCounter();
    watch(false);
    watch(true);
    watch(false);
    watch(false);
    expect(calls.stops).toBe(1);
  });

  test("never stops for the initial closed state", () => {
    const { calls, watch } = watcherWithCounter();
    watch(false);
    expect(calls.stops).toBe(0);
  });

  test("stops again after a second open and close", () => {
    const { calls, watch } = watcherWithCounter();
    for (const open of [true, false, true, false]) watch(open);
    expect(calls.stops).toBe(2);
  });
});
