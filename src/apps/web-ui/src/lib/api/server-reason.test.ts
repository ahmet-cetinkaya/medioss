import { describe, expect, test } from "bun:test";
import { serverReasonMessages } from "../i18n/server-reason.messages";
import { ApiError, type ApiAction } from "./api-error";
import { SERVER_REASONS, serverReason } from "./server-reason";

const reason = (
  action: ApiAction,
  status: number,
  message = "",
  context?: { selfDisable?: boolean },
) => serverReason(new ApiError(status, action, message), context);

const LAST_ADMIN = "operation would leave the instance without an active admin";

describe("serverReason (refusal table)", () => {
  test("409 duplicate email → emailExists", () => {
    expect(reason("users.create", 409, "email already exists")).toEqual({
      reason: "emailExists",
    });
  });

  test("409 setup → setupAlreadyCompleted; 403 setup → setupForbidden", () => {
    expect(reason("setup", 409, "setup already completed").reason).toBe(
      "setupAlreadyCompleted",
    );
    expect(reason("setup", 409).reason).toBe("setupAlreadyCompleted");
    expect(reason("setup", 403, "forbidden").reason).toBe("setupForbidden");
  });

  test("400 last admin → lastActiveAdmin; caller context → selfDisable", () => {
    expect(reason("users.status", 400, LAST_ADMIN).reason).toBe(
      "lastActiveAdmin",
    );
    expect(reason("account.delete", 400, LAST_ADMIN).reason).toBe(
      "lastActiveAdmin",
    );
    expect(
      reason("users.status", 400, LAST_ADMIN, { selfDisable: true }).reason,
    ).toBe("selfDisable");
  });

  test("400 password too short → passwordTooShort", () => {
    const message = "password must contain at least 8 characters";
    expect(reason("password.change", 400, message).reason).toBe(
      "passwordTooShort",
    );
    expect(reason("users.create", 400, message).reason).toBe(
      "passwordTooShort",
    );
  });

  test("400 invalid email → emailInvalid", () => {
    expect(reason("setup", 400, "email is invalid").reason).toBe(
      "emailInvalid",
    );
  });

  test("avatar 413 → avatarTooLarge; 400 bad image → avatarInvalid", () => {
    expect(reason("profile.avatar", 413, "payload too large").reason).toBe(
      "avatarTooLarge",
    );
    expect(
      reason("profile.avatar", 400, "profile payload is invalid").reason,
    ).toBe("avatarInvalid");
  });

  test("too-long meditation description 400 → descriptionTooLong", () => {
    const body = "description must be at most 2000 characters";
    expect(reason("meditation.description", 400, body).reason).toBe(
      "descriptionTooLong",
    );
    expect(reason("audio.upload", 400, body).reason).toBe("descriptionTooLong");
  });

  test("reorder mismatch 400 → reorderMismatch", () => {
    expect(
      reason(
        "program.members.reorder",
        400,
        "meditation_ids must contain exactly the program's meditations",
      ).reason,
    ).toBe("reorderMismatch");
  });

  test("soundscape category: DELETE 409 → categoryInUse, POST/PUT 409 → duplicateName", () => {
    expect(reason("soundscape.category.delete", 409).reason).toBe(
      "categoryInUse",
    );
    expect(reason("soundscape.category.create", 409).reason).toBe(
      "duplicateName",
    );
    expect(reason("soundscape.category.rename", 409).reason).toBe(
      "duplicateName",
    );
    expect(
      reason(
        "category.rename",
        409,
        "a category with that name already exists",
      ),
    ).toEqual({
      reason: "duplicateName",
    });
  });

  test("415 → unsupportedMedia", () => {
    const message = "the uploaded bytes are not a supported media format";
    expect(reason("audio.upload", 415, message).reason).toBe(
      "unsupportedMedia",
    );
    expect(reason("program.create", 415).reason).toBe("unsupportedMedia");
    expect(reason("sound.upload", 415).reason).toBe("unsupportedMedia");
  });

  test("413 uploads → fileTooLarge with the server's limit", () => {
    expect(reason("program.create", 413)).toEqual({
      reason: "fileTooLarge",
      params: { limitMb: 5 },
    });
    expect(reason("program.cover", 413).params).toEqual({ limitMb: 5 });
    expect(reason("audio.upload", 413)).toEqual({
      reason: "fileTooLarge",
      params: { limitMb: 100 },
    });
    expect(reason("sound.upload", 413)).toEqual({
      reason: "fileTooLarge",
      params: { limitMb: 100 },
    });
    expect(reason("sound.icon.upload", 413)).toEqual({
      reason: "fileTooLarge",
      params: { limitMb: 1 },
    });
  });

  test.each([
    ["missing import file", "importMissingFile"],
    [
      "not a valid export file: expected value at line 1",
      "importNotExportFile",
    ],
    [
      "unknown export format version 9 (this instance understands 1)",
      "importUnknownVersion",
    ],
    ["not a valid CSV export: bad row", "importNotCsv"],
    ["expected 3 columns, found 2", "importBadColumns"],
    ["invalid recorded_at 'x': bad", "importInvalidRecordedAt"],
    ["invalid score 'x': bad", "importInvalidScore"],
    ["mood score 7 outside 1-5", "importInvalidScore"],
    ["invalid record: mood entry recorded in the future", "importFutureEntry"],
    [
      "invalid record: mood note must be at most 10000 characters",
      "importNoteTooLong",
    ],
    ["invalid record: note text must not be empty", "importEmptyNote"],
    ["mood entry recorded in the future", "importFutureEntry"],
    ["something else entirely", "importInvalid"],
    ["", "importInvalid"],
  ])("import 400 %p → %s", (message, expected) => {
    expect(reason("import", 400, message).reason).toBe(expected as never);
  });

  test("import 413 → importTooLarge", () => {
    expect(reason("import", 413, "import file exceeds 10 MB").reason).toBe(
      "importTooLarge",
    );
  });

  test("login 401 → invalidCredentials; 429 → throttled", () => {
    expect(reason("login", 401, "invalid credentials").reason).toBe(
      "invalidCredentials",
    );
    expect(reason("login", 429, "too many attempts").reason).toBe("throttled");
  });

  test("404 → notFound", () => {
    expect(
      reason("meditation.load", 404, "the requested resource was not found")
        .reason,
    ).toBe("notFound");
    expect(reason("mood.update", 404, "internal error").reason).toBe(
      "notFound",
    );
  });

  test("0 → network; 5xx → server", () => {
    expect(reason("library.load", 0).reason).toBe("network");
    expect(reason("library.load", 500, "internal error").reason).toBe("server");
    expect(reason("sound.upload", 503).reason).toBe("server");
  });

  test("anything else → generic", () => {
    expect(reason("mood.create", 400, "score out of range").reason).toBe(
      "generic",
    );
    expect(reason("users.list", 403, "forbidden").reason).toBe("generic");
    expect(reason("mix.create", 409).reason).toBe("generic");
    expect(reason("note.create", 413).reason).toBe("generic");
    expect(reason("generic", 400, "constructor").reason).toBe("generic");
  });
});

describe("serverReasonMessages", () => {
  test("every ServerReason has a message, and nothing else", () => {
    expect(Object.keys(serverReasonMessages).sort()).toEqual(
      [...SERVER_REASONS].sort(),
    );
  });

  test("fileTooLarge takes the limit placeholder", () => {
    expect(serverReasonMessages.fileTooLarge.en).toContain("{limitMb}");
    expect(serverReasonMessages.fileTooLarge.tr).toContain("{limitMb}");
  });
});
