// Playwright has no option for prefers-reduced-transparency, so set it over
// Chromium CDP. `Emulation.setEmulatedMedia` replaces every emulated feature,
// so reduced motion can be set in the same call; a later `page.emulateMedia`
// call would reset these features.
import type { CDPSession, Page } from "@playwright/test";

export async function emulateReducedTransparency(
  page: Page,
  options: {
    transparency?: "reduce" | "no-preference";
    reducedMotion?: "reduce" | "no-preference";
  } = {},
): Promise<CDPSession> {
  const features = [
    {
      name: "prefers-reduced-transparency",
      value: options.transparency ?? "reduce",
    },
  ];
  if (options.reducedMotion)
    features.push({
      name: "prefers-reduced-motion",
      value: options.reducedMotion,
    });
  const session = await page.context().newCDPSession(page);
  await session.send("Emulation.setEmulatedMedia", { features });
  return session;
}

/** Reduced motion through Playwright's own API. */
export async function emulateReducedMotion(page: Page): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
}
