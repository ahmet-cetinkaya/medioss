// Copy for the shared date-range popover (History and Statistics).

import { defineMessages } from "../../i18n/message";

export const rangePopoverMessages = defineMessages({
  /** Trigger accessible name: the control, then the current choice. */
  triggerLabel: { en: "Date range: {range}", tr: "Tarih aralığı: {range}" },
  presetsLegend: { en: "Quick ranges", tr: "Hızlı aralıklar" },
  customHeading: { en: "Custom range", tr: "Özel aralık" },
  from: { en: "From", tr: "Başlangıç" },
  to: { en: "To", tr: "Bitiş" },
  apply: { en: "Apply", tr: "Uygula" },
  customLabel: { en: "{from} – {to}", tr: "{from} – {to}" },
  errorOrder: {
    en: "The start date must not be after the end date.",
    tr: "Başlangıç tarihi bitiş tarihinden sonra olamaz.",
  },
  errorTooLong: {
    en: "Pick at most {days} days.",
    tr: "En fazla {days} gün seçin.",
  },
  errorMissing: {
    en: "Pick both dates.",
    tr: "İki tarihi de seçin.",
  },
});
