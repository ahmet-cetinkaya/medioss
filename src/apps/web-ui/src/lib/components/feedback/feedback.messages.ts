// Copy for the shared feedback and control primitives (T9). Loading and retry
// text live in common.messages.ts (stateMessages, actionMessages).

import { defineMessages } from "../../i18n/message";

/** Visible state text beside a switch (Toggle). */
export const toggleMessages = defineMessages({
  on: { en: "On", tr: "Açık" },
  off: { en: "Off", tr: "Kapalı" },
});

/** Accessible names for the InlineMessage tone icon, so tone is not color only. */
export const toneMessages = defineMessages({
  error: { en: "Error", tr: "Hata" },
  success: { en: "Success", tr: "Başarılı" },
  info: { en: "Note", tr: "Not" },
});
