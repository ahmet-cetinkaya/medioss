// T14: the shared confirm dialog through the Sign out confirm (plan T10:
// initial focus, focus trap, Escape and focus return, language switch while open).
import { loginApi, storageStatePath, USERS } from "../support/auth";
import { expect, test } from "../support/test-fixture";

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("focus starts on Cancel, Tab stays inside, Escape returns focus to the opener", async ({
    page,
  }) => {
    await page.goto("/stats/");
    await expect(page.locator("main dl dt")).toHaveCount(4);
    const opener = page
      .getByRole("navigation", { name: "Main navigation" })
      .getByRole("button", { name: "Sign out" });
    await opener.click();

    const dialog = page.getByRole("alertdialog", { name: "Sign out?" });
    const cancel = dialog.getByRole("button", { name: "Cancel" });
    const confirm = dialog.getByRole("button", { name: "Sign out" });
    await expect(cancel).toBeFocused();

    await page.keyboard.press("Tab");
    await expect(confirm).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(cancel).toBeFocused();
    await page.keyboard.press("Shift+Tab");
    await expect(confirm).toBeFocused();

    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
    await expect(opener).toBeFocused();
    expect(new URL(page.url()).pathname).toBe("/stats/");
  });
});

test.describe("flagged user (the change-password view has the language switcher)", () => {
  test.use({ storageState: { cookies: [], origins: [] } });

  test("switching the language while the dialog is open re-renders it and keeps it open", async ({
    page,
    context,
  }) => {
    const api = await loginApi(
      USERS.flagged.email,
      USERS.flagged.temporaryPassword,
    );
    await context.addCookies((await api.storageState()).cookies);
    await api.dispose();

    await page.goto("/stats/");
    const opener = page.getByRole("button", { name: "Sign out" });
    await opener.click();
    const dialog = page.getByRole("alertdialog");
    await expect(dialog).toHaveAccessibleName("Sign out?");
    await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();

    // The page behind the modal is not reachable by pointer or keyboard, so
    // switch programmatically through the real control (locale.set).
    await page.evaluate(() =>
      document
        .querySelector<HTMLInputElement>('label[lang="tr"] input')
        ?.click(),
    );

    await expect(dialog).toHaveAccessibleName("Çıkış yapılsın mı?");
    await expect(dialog.getByRole("button", { name: "İptal" })).toBeVisible();
    await expect(
      dialog.getByRole("button", { name: "Çıkış yap" }),
    ).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("lang", "tr");

    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Çıkış yap" })).toBeFocused();
    expect(new URL(page.url()).pathname).toBe("/stats/");
  });
});
