import { describe, expect, test } from "bun:test";
import { ApiError } from "../../api/api-error";
import { serverReasonMessages } from "../../i18n/server-reason.messages";
import { settingsFailure } from "./settings-failure";
import { dataMessages, deleteAccountMessages } from "./settings.messages";

const LAST_ADMIN = "operation would leave the instance without an active admin";

describe("settingsFailure", () => {
  test("a recognized reason shows its message", () => {
    const error = new ApiError(400, "account.delete", LAST_ADMIN);
    expect(settingsFailure(error, deleteAccountMessages.failed)).toEqual({
      message: serverReasonMessages.lastActiveAdmin,
      params: undefined,
    });
    const bad = new ApiError(400, "import", "not a valid export file: x");
    expect(settingsFailure(bad, dataMessages.importFailed).message).toBe(
      serverReasonMessages.importNotExportFile,
    );
  });

  test("a generic refusal or a non-API error shows the fallback", () => {
    const fallback = deleteAccountMessages.failed;
    const error = new ApiError(400, "account.delete", "something else");
    expect(settingsFailure(error, fallback)).toEqual({ message: fallback });
    expect(settingsFailure(new Error("boom"), fallback)).toEqual({
      message: fallback,
    });
  });
});
