// Native alert/confirm/prompt/beforeunload dialogs are forbidden (D6): any one fails the test.
import type { Page } from "@playwright/test";

export type DialogGuard = {
  dialogs: { type: string; message: string }[];
  assertNone(): void;
};

export function guardNativeDialogs(page: Page): DialogGuard {
  const dialogs: { type: string; message: string }[] = [];
  page.on("dialog", (dialog) => {
    dialogs.push({ type: dialog.type(), message: dialog.message() });
    // Dismiss so the page does not hang; the test still fails in assertNone().
    void dialog.dismiss().catch(() => {});
  });
  return {
    dialogs,
    assertNone() {
      if (dialogs.length > 0) {
        throw new Error(
          `native dialog(s) opened: ${dialogs.map((d) => `${d.type}: ${d.message}`).join(" | ")}`,
        );
      }
    },
  };
}
