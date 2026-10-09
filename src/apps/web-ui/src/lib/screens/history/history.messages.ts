// Copy for the History screen (FR-060–FR-062).

import { defineMessages } from "../../i18n/message";

export const historyMessages = defineMessages({
  rangeLegend: { en: "History range", tr: "Geçmiş aralığı" },
  rangeToday: { en: "Today", tr: "Bugün" },
  rangeLast7: { en: "Last 7 days", tr: "Son 7 gün" },
  rangeLast30: { en: "Last 30 days", tr: "Son 30 gün" },
  rangeAll: { en: "All time", tr: "Tüm zamanlar" },
  notesOnly: { en: "Notes only", tr: "Yalnızca notlar" },
  timelineHeading: { en: "Timeline", tr: "Zaman çizelgesi" },
  moodEvent: {
    en: "Mood entry: {score} of 5",
    tr: "Ruh hâli kaydı: 5 üzerinden {score}",
  },
  noteEvent: { en: "Meditation note: {title}", tr: "Meditasyon notu: {title}" },
  completedEvent: {
    en: "Completed meditation: {title}",
    tr: "Tamamlanan meditasyon: {title}",
  },
  otherEvent: { en: "Event: {title}", tr: "Olay: {title}" },
  emptyTitle: {
    en: "No history for {range}",
    tr: "{range} için geçmiş yok",
  },
  emptyFilterAll: {
    en: "Filter: {range}, all events. Meditate or log your mood to see it here.",
    tr: "Filtre: {range}, tüm olaylar. Burada görmek için meditasyon yapın veya ruh hâlinizi kaydedin.",
  },
  emptyFilterNotes: {
    en: "Filter: {range}, notes only. Add a note to a meditation or a mood entry to see it here.",
    tr: "Filtre: {range}, yalnızca notlar. Burada görmek için bir meditasyona veya ruh hâli kaydına not ekleyin.",
  },
  openLibrary: { en: "Go to Library", tr: "Kütüphaneye git" },
  openMood: { en: "Go to Mood", tr: "Ruh hâline git" },
  loadedMore: {
    en: (p: { count: number }) =>
      `${p.count} more ${p.count === 1 ? "event" : "events"} loaded`,
    tr: (p: { count: number }) => `${p.count} olay daha yüklendi`,
  },
  endReached: {
    en: "End of history",
    tr: "Geçmişin sonu",
  },
  loadFailed: {
    en: "Could not load your history. Try again.",
    tr: "Geçmişiniz yüklenemedi. Yeniden deneyin.",
  },
  loadMoreFailed: {
    en: "Could not load more events. Try again.",
    tr: "Daha fazla olay yüklenemedi. Yeniden deneyin.",
  },
});
