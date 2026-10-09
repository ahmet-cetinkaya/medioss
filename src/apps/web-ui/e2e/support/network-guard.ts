// Records every request a page makes and checks each origin is the app's own.
import type { Page } from "@playwright/test";
import { baseURL } from "./server-env";

export type RecordedRequest = {
  method: string;
  url: string;
  origin: string;
  resourceType: string;
};

export type NetworkGuard = {
  requests: RecordedRequest[];
  /** Requests whose origin differs from the configured base URL origin. */
  foreign(): RecordedRequest[];
  /** Throws listing every foreign request. */
  assertSameOrigin(): void;
};

// Browser-internal schemes never leave the machine.
const LOCAL_SCHEMES = new Set(["data:", "blob:", "about:"]);

export function guardNetwork(
  page: Page,
  expectedOrigin = new URL(baseURL()).origin,
): NetworkGuard {
  const requests: RecordedRequest[] = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    requests.push({
      method: request.method(),
      url: request.url(),
      origin: LOCAL_SCHEMES.has(url.protocol) ? expectedOrigin : url.origin,
      resourceType: request.resourceType(),
    });
  });
  const foreign = () =>
    requests.filter((request) => request.origin !== expectedOrigin);
  return {
    requests,
    foreign,
    assertSameOrigin() {
      const offenders = foreign();
      if (offenders.length > 0) {
        throw new Error(
          `requests left ${expectedOrigin}:\n${offenders.map((r) => `${r.method} ${r.url}`).join("\n")}`,
        );
      }
    },
  };
}
