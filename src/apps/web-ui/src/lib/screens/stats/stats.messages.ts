// Copy for the Statistics screen (FR-063–FR-065).

import { defineMessages } from "../../i18n/message";

export const statsMessages = defineMessages({
  rangeLegend: { en: "Statistics range", tr: "İstatistik aralığı" },
  rangeToday: { en: "Today", tr: "Bugün" },
  rangeLast30: { en: "Last 30 days", tr: "Son 30 gün" },
  rangeAll: { en: "All time", tr: "Tüm zamanlar" },
  summaryHeading: { en: "Summary", tr: "Özet" },
  totalMinutes: { en: "Total minutes", tr: "Toplam dakika" },
  totalDays: { en: "Total practice days", tr: "Toplam pratik günü" },
  currentStreak: { en: "Current streak", tr: "Güncel seri" },
  longestStreak: { en: "Longest streak", tr: "En uzun seri" },
  daysHeading: { en: "Practiced days", tr: "Pratik yapılan günler" },
  noPracticeTitle: {
    en: "No practice in this range",
    tr: "Bu aralıkta pratik yok",
  },
  noPracticeText: {
    en: "Completed meditations appear here with their date and minutes.",
    tr: "Tamamlanan meditasyonlar tarihleri ve dakikalarıyla burada görünür.",
  },
  openLibrary: { en: "Go to Library", tr: "Kütüphaneye git" },
  loadFailed: {
    en: "Could not load statistics. Try again.",
    tr: "İstatistikler yüklenemedi. Yeniden deneyin.",
  },
});
