// Home region and check-in failures (FR-029, FR-111): a recognized server
// reason shows its own text; anything else shows the region's own message.

import type { ServerReason } from "../../api/server-reason";
import type { StringMessage } from "../../i18n/message";
import { serverReasonMessages } from "../../i18n/server-reason.messages";

export function reasonMessage(
  reason: ServerReason,
  fallback: StringMessage,
): StringMessage {
  return reason === "generic" ? fallback : serverReasonMessages[reason];
}
