// Browser suites import `test`/`expect` from here: the network-origin and
// native-dialog guards apply to the default `page` automatically and fail the
// test on teardown. Pages from extra contexts need guardNetwork/guardNativeDialogs.
import { test as base } from "@playwright/test";
import { guardNativeDialogs, type DialogGuard } from "./native-dialog-guard";
import { guardNetwork, type NetworkGuard } from "./network-guard";

export const test = base.extend<{
  networkGuard: NetworkGuard;
  dialogGuard: DialogGuard;
}>({
  networkGuard: [
    async ({ page }, use) => {
      const guard = guardNetwork(page);
      await use(guard);
      guard.assertSameOrigin();
    },
    { auto: true },
  ],
  dialogGuard: [
    async ({ page }, use) => {
      const guard = guardNativeDialogs(page);
      await use(guard);
      guard.assertNone();
    },
    { auto: true },
  ],
});

export { expect } from "@playwright/test";
