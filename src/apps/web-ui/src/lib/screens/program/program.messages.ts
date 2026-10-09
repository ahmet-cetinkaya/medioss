// Copy for the Program screen (FR-012, FR-038–FR-042).

import { defineMessages } from "../../i18n/message";

export const programMessages = defineMessages({
  start: { en: "Start", tr: "Başla" },
  continue: { en: "Continue", tr: "Devam et" },
  playAgain: { en: "Play again", tr: "Yeniden oynat" },
  unavailable: {
    en: "Nothing to play yet: this program has no sessions.",
    tr: "Henüz oynatılacak bir şey yok: bu programda seans yok.",
  },
  sessionsHeading: { en: "Sessions", tr: "Seanslar" },
  emptyTitle: { en: "No sessions yet", tr: "Henüz seans yok" },
  emptyText: {
    en: "This program has no sessions yet. Explore the Library in the meantime.",
    tr: "Bu programda henüz seans yok. Bu arada Kütüphaneye göz atın.",
  },
  notFoundTitle: { en: "Program not found", tr: "Program bulunamadı" },
  notFoundText: {
    en: "This program does not exist or was removed.",
    tr: "Bu program mevcut değil ya da kaldırılmış.",
  },
  loadFailed: {
    en: "Could not load the program. Try again.",
    tr: "Program yüklenemedi. Yeniden deneyin.",
  },
});
