import { expect, test } from "../support/test-fixture";
import { storageStatePath } from "../support/auth";

test.use({ storageState: storageStatePath("admin") });

test("admin deep paths serve their section page", async ({ page }) => {
  // The soundscape section is selected at its own path.
  await page.goto("/admin/soundscape/");
  const soundscapeChip = page
    .locator("fieldset")
    .getByLabel("Soundscape", { exact: true });
  await expect(soundscapeChip).toBeChecked();

  // Program detail deep path (SPA fallback) renders the meditations island
  // with the program selected from the address.
  await page.goto("/admin/meditations/1/");
  const meditationsChip = page
    .locator("fieldset")
    .getByLabel("Meditations", { exact: true });
  await expect(meditationsChip).toBeChecked();
  await expect(
    page.getByRole("heading", { name: /meditations in/i }),
  ).toBeVisible();
});

test("admin tabs omit check icons", async ({ page }) => {
  await page.goto("/admin/");
  const tabs = page.getByRole("group", { name: "Admin section" });
  await expect(tabs).toBeVisible();
  await expect(tabs.locator("svg")).toHaveCount(0);
});
