import { describe, expect, test } from "bun:test";
import { ApiError } from "./api-error";
import { failureMessage, reasonOf } from "./failure";
import { serverReasonMessages } from "../i18n/server-reason.messages";

const fallback = { en: "Did not load", tr: "Yüklenmedi" } as const;

describe("reasonOf", () => {
  test("an ApiError maps to its server reason", () => {
    const error = new ApiError(409, "category.create", "in use");
    expect(reasonOf(error)).toBe("duplicateName");
  });

  test("anything else is generic", () => {
    expect(reasonOf(new Error("offline"))).toBe("generic");
    expect(reasonOf(undefined)).toBe("generic");
  });
});

describe("failureMessage", () => {
  test("a recognized reason shows its own text", () => {
    expect(failureMessage("network", fallback).message).toBe(
      serverReasonMessages.network,
    );
  });

  test("null and generic fall back to the region's message", () => {
    expect(failureMessage(null, fallback).message).toBe(fallback);
    expect(failureMessage("generic", fallback).message).toBe(fallback);
  });

  test("not-found keeps its own message (screens share the wording)", () => {
    expect(failureMessage("notFound", fallback).message).toBe(
      serverReasonMessages.notFound,
    );
  });

  test("an import refusal keeps its specific message", () => {
    const error = new ApiError(400, "import", "not a valid export file: x");
    expect(failureMessage(reasonOf(error), fallback).message).toBe(
      serverReasonMessages.importNotExportFile,
    );
  });

  test("a policy restricts specific messages to its listed reasons", () => {
    expect(
      failureMessage("network", fallback, {
        specificReasons: ["network", "server"],
      }).message,
    ).toBe(serverReasonMessages.network);
    expect(
      failureMessage("notFound", fallback, {
        specificReasons: ["network", "server"],
      }).message,
    ).toBe(fallback);
  });
});
