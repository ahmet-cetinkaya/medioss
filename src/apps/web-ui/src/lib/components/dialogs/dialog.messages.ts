// Copy for the shared dialogs (T10). Cancel, Save, and Close live in
// common.messages.ts (actionMessages).

import { defineMessages } from "../../i18n/message";

export const dialogMessages = defineMessages({
  name: { en: "Name", tr: "Ad" },
  nameRequired: {
    en: "Enter a name to save.",
    tr: "Kaydetmek için bir ad girin.",
  },
});
