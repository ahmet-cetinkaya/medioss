// One owner for the two questions every screen asks after a refused action
// (plan D8): which reason was this, and what text do we show for it?
//
// `reasonOf` maps a thrown error to a ServerReason ("generic" for anything
// that is not an ApiError). `failureMessage` picks the message: a recognized
// non-generic reason shows its own text; everything else falls back to the
// region's own message. Screens must show serverReason text for specific
// refusals (FR-015) and their own text otherwise — these functions encode that
// rule once.

import { ApiError } from "./api-error";
import { serverReason, type ServerReason } from "./server-reason";
import type { Message, MessageParams } from "../i18n/message";
import { serverReasonMessages } from "../i18n/server-reason.messages";

export const NETWORK_OR_SERVER_REASONS = ["network", "server"] as const;

export function reasonOf(error: unknown): ServerReason {
  return error instanceof ApiError ? serverReason(error).reason : "generic";
}

export function failureMessage(
  reason: ServerReason | null,
  fallback: Message,
  { specificReasons }: { specificReasons?: readonly ServerReason[] } = {},
): { message: Message; params?: MessageParams } {
  if (
    !reason ||
    reason === "generic" ||
    (specificReasons && !specificReasons.includes(reason))
  )
    return { message: fallback };
  return { message: serverReasonMessages[reason] };
}
