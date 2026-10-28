import { storageStatePath } from "../support/auth";
import { expect, test } from "../support/test-fixture";

// Search results are cards in a wrapping list; without an explicit width each
// collapsed to ~50 px (title wrapped per letter).
const MIN_CARD_WIDTH_PX = 150;

test.use({ storageState: storageStatePath("member") });

test("search results are full-size cards, not squashed tiles", async ({
  page,
}) => {
  await page.goto("/library/");
  await page.getByRole("searchbox").fill("Body Scan");
  await page.getByRole("searchbox").press("Enter");

  const card = page
    .getByRole("region", { name: /Results/ })
    .getByRole("link", { name: /^Body Scan/ });
  await expect(card).toBeVisible();

  const box = await card.boundingBox();
  expect(box!.width).toBeGreaterThanOrEqual(MIN_CARD_WIDTH_PX);
});
