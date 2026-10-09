// Maps a refused API call to a user-facing reason code (plan D8, spec
// FR-015/FR-118). Bodies carry no machine-readable code, so recognition uses
// the action, the status, and the server's message text (refusal table in the
// plan's "Backend contract"). Screens show `t(serverReasonMessages[reason],
// params)`; `generic` means "show that action's own failure message".

import type { ApiAction, ApiError } from "./api-error";

export const SERVER_REASONS = [
  "emailExists",
  "setupAlreadyCompleted",
  "setupForbidden",
  "lastActiveAdmin",
  "selfDisable",
  "passwordTooShort",
  "emailInvalid",
  "avatarTooLarge",
  "avatarInvalid",
  "descriptionTooLong",
  "categoryInUse",
  "reorderMismatch",
  "duplicateName",
  "unsupportedMedia",
  "fileTooLarge",
  "importMissingFile",
  "importNotExportFile",
  "importUnknownVersion",
  "importNotCsv",
  "importBadColumns",
  "importInvalidRecordedAt",
  "importInvalidScore",
  "importFutureEntry",
  "importNoteTooLong",
  "importEmptyNote",
  "importTooLarge",
  "importInvalid",
  "invalidCredentials",
  "throttled",
  "notFound",
  "network",
  "server",
  "generic",
] as const;

export type ServerReason = (typeof SERVER_REASONS)[number];

/** `params` is set only for `fileTooLarge` (`{ limitMb }`). */
export interface ServerReasonResult {
  reason: ServerReason;
  params?: Record<string, string | number>;
}

export interface ServerReasonContext {
  /** Admin: the target user is the current user (same 400 as last admin). */
  selfDisable?: boolean;
}

// Server-side upload caps in MB (meditation MAX_UPLOAD_BYTES / MAX_COVER_BYTES,
// soundscape MAX_UPLOAD_BYTES); their 413 bodies are empty.
const UPLOAD_LIMIT_MB: Partial<Record<ApiAction, number>> = {
  "program.create": 5,
  "program.cover": 5,
  "audio.upload": 100,
  "sound.upload": 100,
  "sound.icon.upload": 1,
};

const DUPLICATE_NAME_ACTIONS: readonly ApiAction[] = [
  "soundscape.category.create",
  "soundscape.category.rename",
  "category.create",
];

// Import validation errors in the JSON path arrive as "invalid record: <msg>";
// a JSON score error reads "mood score N outside 1-5".
const INVALID_RECORD = "invalid record: ";
const IMPORT_PREFIXES: readonly [string, ServerReason][] = [
  ["missing import file", "importMissingFile"],
  ["not a valid export file", "importNotExportFile"],
  ["unknown export format version", "importUnknownVersion"],
  ["not a valid CSV export", "importNotCsv"],
  ["expected 3 columns", "importBadColumns"],
  ["invalid recorded_at", "importInvalidRecordedAt"],
  ["invalid score", "importInvalidScore"],
  ["mood score ", "importInvalidScore"],
  ["mood entry recorded in the future", "importFutureEntry"],
  ["mood note must be at most 10000 characters", "importNoteTooLong"],
  ["note text must not be empty", "importEmptyNote"],
];

const IDENTITY_MESSAGES = new Map<string, ServerReason>([
  ["email already exists", "emailExists"],
  ["email is invalid", "emailInvalid"],
  ["password must contain at least 8 characters", "passwordTooShort"],
  ["setup already completed", "setupAlreadyCompleted"],
  ["initial setup has already been completed", "setupAlreadyCompleted"],
]);

const LAST_ADMIN = "operation would leave the instance without an active admin";

function importReason(status: number, message: string): ServerReason {
  if (status === 413) return "importTooLarge";
  const text = message.startsWith(INVALID_RECORD)
    ? message.slice(INVALID_RECORD.length)
    : message;
  return (
    IMPORT_PREFIXES.find(([prefix]) => text.startsWith(prefix))?.[1] ??
    "importInvalid"
  );
}

/** Recognizes why the server refused `error`'s action. */
export function serverReason(
  { status, action, serverMessage }: ApiError,
  context: ServerReasonContext = {},
): ServerReasonResult {
  const message = serverMessage.trim();
  if (status === 0) return { reason: "network" };
  if (status >= 500) return { reason: "server" };
  if (status === 404) return { reason: "notFound" };

  if (action === "login" && status === 401)
    return { reason: "invalidCredentials" };
  if (action === "login" && status === 429) return { reason: "throttled" };
  if (action === "setup" && status === 403) return { reason: "setupForbidden" };
  if (action === "setup" && status === 409)
    return { reason: "setupAlreadyCompleted" };
  if (action === "import" && (status === 400 || status === 413))
    return { reason: importReason(status, message) };
  if (action === "soundscape.category.delete" && status === 409)
    return { reason: "categoryInUse" };
  if (DUPLICATE_NAME_ACTIONS.includes(action) && status === 409)
    return { reason: "duplicateName" };
  if (
    action === "program.members.reorder" &&
    status === 400 &&
    message === "meditation_ids must contain exactly the program's meditations"
  )
    return { reason: "reorderMismatch" };
  if (action === "profile.avatar" && status === 413)
    return { reason: "avatarTooLarge" };
  if (action === "profile.avatar" && message === "profile payload is invalid")
    return { reason: "avatarInvalid" };

  if (
    (action === "meditation.description" || action === "audio.upload") &&
    status === 400 &&
    message === "description must be at most 2000 characters"
  )
    return { reason: "descriptionTooLong" };

  const limitMb = UPLOAD_LIMIT_MB[action];
  if (status === 413 && limitMb !== undefined)
    return { reason: "fileTooLarge", params: { limitMb } };
  if (status === 415) return { reason: "unsupportedMedia" };
  if (status === 400 && message === LAST_ADMIN)
    return { reason: context.selfDisable ? "selfDisable" : "lastActiveAdmin" };

  return { reason: IDENTITY_MESSAGES.get(message) ?? "generic" };
}
