// Captures the main screens of a seeded instance into docs/screenshots/.
// Usage (via `make demo-screenshots`): E2E_PORT=<port> SESSION_SECRET=<test secret> bun demo/screenshots.ts
import fs from "node:fs";
import path from "node:path";
import { chromium, type BrowserContext } from "@playwright/test";
import { loginAs, storageStatePath, type Role } from "../e2e/support/auth";
import { baseURL, repoRoot } from "../e2e/support/server-env";

type Screen = { name: string; url: string; role: Role | null };

const SCREENS: Screen[] = [
  { name: "login", url: "/login/", role: null },
  { name: "home", url: "/", role: "member" },
  { name: "library", url: "/library/", role: "member" },
  { name: "program", url: "/program/?id=1", role: "member" },
  { name: "player", url: "/player/1/", role: "member" },
  { name: "soundscape", url: "/soundscape/", role: "member" },
  { name: "mood", url: "/mood/", role: "member" },
  { name: "history", url: "/history/", role: "member" },
  { name: "stats", url: "/stats/", role: "member" },
  { name: "settings", url: "/settings/", role: "member" },
  { name: "admin", url: "/admin/", role: "admin" },
];

const VIEWPORTS = {
  desktop: { width: 1280, height: 800 },
  mobile: { width: 390, height: 844 },
} as const;

const SETTLE_MS = 800;
const outDir = path.join(repoRoot, "docs", "screenshots");

async function capture(
  context: BrowserContext,
  screen: Screen,
  viewport: string,
): Promise<void> {
  const page = await context.newPage();
  await page.goto(`${baseURL()}${screen.url}`);
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(SETTLE_MS);
  await page.screenshot({
    path: path.join(outDir, `${screen.name}-${viewport}.png`),
  });
  await page.close();
}

async function main(): Promise<void> {
  fs.mkdirSync(outDir, { recursive: true });
  await (await loginAs("member")).dispose(); // fails early when the instance is not seeded
  const browser = await chromium.launch();
  for (const [viewport, size] of Object.entries(VIEWPORTS)) {
    for (const role of [null, "member", "admin"] as const) {
      const context = await browser.newContext({
        viewport: size,
        locale: "en-US",
        storageState: role ? storageStatePath(role) : undefined,
      });
      for (const screen of SCREENS.filter((s) => s.role === role)) {
        await capture(context, screen, viewport);
      }
      await context.close();
    }
  }
  await browser.close();
  console.log(`screenshots written to ${outDir}`);
}

main().catch((error: Error) => {
  console.error(error.message);
  process.exit(1);
});
