// Progress labels for programs and meditations (PB-19).

import { defineMessages } from "../i18n/message";

export const progressMessages = defineMessages({
  completed: { en: "Completed", tr: "Tamamlandı" },
  /** `percent` is already locale-formatted ("40%", "%40"). */
  programPercent: { en: "{percent} complete", tr: "{percent} tamamlandı" },
});
