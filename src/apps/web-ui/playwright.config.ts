// E2E harness (plan D7). Run from src/apps/web-ui with SESSION_SECRET set:
//   bunx playwright test                 # make run on :3000, fresh data dir
//   E2E_PORT=3101 bunx playwright test   # isolated: own static dir + release binary
// Policy (docs/ARCHITECTURE.md, Frontend conventions): e2e stays MINIMAL.
// The default run is the core suite (e2e/core/) — a handful of flows most
// likely to break. Retired suites live in e2e/extended/ (testIgnore'd).
import { defineConfig, devices } from "@playwright/test";
import { baseURL, outputDir, webServer } from "./e2e/support/server-env";

export default defineConfig({
  testDir: "./e2e",
  testMatch: "**/*.e2e.ts",
  testIgnore: "**/extended/**",
  outputDir: outputDir(),
  // One shared server and data dir per run: keep suites sequential.
  workers: 1,
  fullyParallel: false,
  forbidOnly: true,
  retries: 0,
  reporter: [["list"]],
  use: {
    ...devices["Desktop Chrome"],
    baseURL: baseURL(),
    // No traces, HAR, or video: the setup secret header must never reach disk.
    trace: "off",
    video: "off",
    screenshot: "only-on-failure",
  },
  projects: [
    { name: "fresh", testDir: "./e2e/fresh" },
    { name: "empty", testDir: "./e2e/empty", dependencies: ["fresh"] },
    { name: "seed", testDir: "./e2e/seed", dependencies: ["empty"] },
    // Core flows run against the seeded instance. The historical seeded
    // suites are retired into e2e/extended/ (see the policy note above).
    { name: "core", testDir: "./e2e/core", dependencies: ["seed"] },
    { name: "seeded", testDir: "./e2e/seeded", dependencies: ["seed"] },
  ],
  webServer: webServer(),
});
