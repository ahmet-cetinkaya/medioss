import { expect, test } from "@playwright/test";
import { storageStatePath } from "../support/auth";

test.use({ storageState: storageStatePath("admin") });

test("admin deep paths serve their section page", async ({ page }) => {
  // The soundscape section is selected at its own path.
  await page.goto("/admin/soundscape/");
  const soundscapeChip = page.getByLabel("Soundscape", { exact: true });
  await expect(soundscapeChip).toBeChecked();

  // Program detail deep path (SPA fallback) renders the meditations island
  // with the program selected from the address.
  await page.goto("/admin/meditations/1/");
  const meditationsChip = page.getByLabel("Meditations", { exact: true });
  await expect(meditationsChip).toBeChecked();
  await expect(
    page.getByRole("heading", { name: /meditations in/i }),
  ).toBeVisible();
});
