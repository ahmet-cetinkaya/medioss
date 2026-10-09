// Turns a refused Settings action into a Message to show (plan D8): the
// recognized server reason, or the action's own failure message when the
// reason is generic. Never server text.

import { ApiError } from "../../api/api-error";
import { serverReason } from "../../api/server-reason";
import type { Message, MessageParams } from "../../i18n/message";
import { serverReasonMessages } from "../../i18n/server-reason.messages";

export type SettingsFailure = { message: Message; params?: MessageParams };

export function settingsFailure(
  error: unknown,
  fallback: Message,
): SettingsFailure {
  if (!(error instanceof ApiError)) return { message: fallback };
  const { reason, params } = serverReason(error);
  return reason === "generic"
    ? { message: fallback }
    : { message: serverReasonMessages[reason], params };
}
