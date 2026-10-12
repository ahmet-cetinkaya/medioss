import { describe, expect, test } from "bun:test";
import { serverReasonMessages } from "../../i18n/server-reason.messages";
import { reasonMessage } from "./reason-message";

const fallback = { en: "Could not load", tr: "Yüklenemedi" };

describe("reasonMessage", () => {
  test("generic falls back to the region's own message", () => {
    expect(reasonMessage("generic", fallback)).toBe(fallback);
  });

  test("a recognized reason shows its own text", () => {
    expect(reasonMessage("network", fallback)).toBe(
      serverReasonMessages.network,
    );
    expect(reasonMessage("server", fallback)).toBe(serverReasonMessages.server);
    expect(reasonMessage("notFound", fallback)).toBe(
      serverReasonMessages.notFound,
    );
  });
});
