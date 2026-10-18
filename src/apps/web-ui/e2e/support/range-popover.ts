// Drives the shared date-range popover (History, Statistics): the trigger
// opens it, the presets are a radio group inside, picking one closes it.

import { expect, type Locator, type Page } from "@playwright/test";

/** The trigger pill; its name is "Date range: <current choice>". */
export const rangeTrigger = (page: Page): Locator =>
  page.getByRole("button", { name: /^Date range:/ });

export async function openRangePopover(page: Page): Promise<Locator> {
  await rangeTrigger(page).click();
  const panel = page.getByRole("dialog");
  await expect(panel).toBeVisible();
  return panel;
}

/** Opens the popover and picks a preset by its visible label. */
export async function pickPreset(page: Page, label: string): Promise<void> {
  const panel = await openRangePopover(page);
  await panel.locator("label").filter({ hasText: label }).click();
  await expect(panel).toBeHidden();
}

/** Opens the popover, fills both dates and applies. */
export async function pickCustomRange(
  page: Page,
  from: string,
  to: string,
): Promise<void> {
  const panel = await openRangePopover(page);
  await panel.getByLabel("From").fill(from);
  await panel.getByLabel("To").fill(to);
  await panel.getByRole("button", { name: "Apply" }).click();
}
