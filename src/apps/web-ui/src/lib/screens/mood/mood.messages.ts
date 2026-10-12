// Copy for the Mood screen (FR-055–FR-059, PB-9).

import { defineMessages } from "../../i18n/message";

const entries = (count: number) =>
  new Intl.PluralRules("en").select(count) === "one" ? "entry" : "entries";

export const moodMessages = defineMessages({
  formHeading: { en: "How are you feeling?", tr: "Nasıl hissediyorsunuz?" },
  editHeading: { en: "Edit mood entry", tr: "Ruh hâli kaydını düzenle" },
  scoreLegend: { en: "Mood score", tr: "Ruh hâli puanı" },
  noteLabel: { en: "Note (optional)", tr: "Not (isteğe bağlı)" },
  noteHint: {
    en: "Up to {limit} characters.",
    tr: "En fazla {limit} karakter.",
  },
  saveMood: { en: "Save mood", tr: "Ruh hâlini kaydet" },
  saveChanges: { en: "Save changes", tr: "Değişiklikleri kaydet" },
  saved: { en: "Mood saved", tr: "Ruh hâli kaydedildi" },
  updated: { en: "Mood entry updated", tr: "Ruh hâli kaydı güncellendi" },
  deleted: { en: "Mood entry deleted", tr: "Ruh hâli kaydı silindi" },
  rangeLegend: { en: "Mood range", tr: "Ruh hâli aralığı" },
  rangeToday: { en: "Today", tr: "Bugün" },
  rangeLast7: { en: "Last 7 days", tr: "Son 7 gün" },
  rangeLast30: { en: "Last 30 days", tr: "Son 30 gün" },
  rangeAll: { en: "All time", tr: "Tüm zamanlar" },
  insightsHeading: { en: "Insights", tr: "İçgörüler" },
  entryCount: { en: "Entries", tr: "Kayıt sayısı" },
  meanScore: { en: "Average score", tr: "Ortalama puan" },
  distributionHeading: { en: "Score distribution", tr: "Puan dağılımı" },
  distributionValue: {
    en: (p: { count: number; percent: string }) =>
      `${p.count} ${entries(p.count)} (${p.percent})`,
    tr: (p: { count: number; percent: string }) =>
      `${p.count} kayıt (${p.percent})`,
  },
  trendHeading: { en: "Daily trend", tr: "Günlük eğilim" },
  dailyAverages: { en: "Daily averages", tr: "Günlük ortalamalar" },
  dailyAverage: { en: "{day}: {mean}", tr: "{day}: {mean}" },
  noDataTitle: {
    en: "No mood entries in this range",
    tr: "Bu aralıkta ruh hâli kaydı yok",
  },
  noDataText: {
    en: "Record how you feel to see your average, distribution, and trend.",
    tr: "Ortalamanızı, dağılımınızı ve eğiliminizi görmek için nasıl hissettiğinizi kaydedin.",
  },
  chooseScore: { en: "Choose a score", tr: "Puan seç" },
  entriesHeading: { en: "Entries", tr: "Kayıtlar" },
  noEntriesTitle: {
    en: "No entries in this range",
    tr: "Bu aralıkta kayıt yok",
  },
  noEntriesText: {
    en: "Saved moods appear here with their date, time, and note.",
    tr: "Kaydettiğiniz ruh hâlleri tarihi, saati ve notuyla burada görünür.",
  },
  entryActions: {
    en: "Actions for entry from {date}",
    tr: "{date} kaydı için işlemler",
  },
  entrySource: {
    en: "Mood {score}/5 · {label}",
    tr: "Ruh hâli {score}/5 · {label}",
  },
  editEntry: { en: "Edit entry from {date}", tr: "{date} kaydını düzenle" },
  deleteEntry: { en: "Delete entry from {date}", tr: "{date} kaydını sil" },
  deleteTitle: {
    en: "Delete this mood entry?",
    tr: "Bu ruh hâli kaydı silinsin mi?",
  },
  deleteConsequence: {
    en: "The entry and its note are deleted permanently.",
    tr: "Kayıt ve notu kalıcı olarak silinir.",
  },
  deleteConfirm: { en: "Delete entry", tr: "Kaydı sil" },
  loadFailed: {
    en: "Could not load mood data. Try again.",
    tr: "Ruh hâli verileri yüklenemedi. Yeniden deneyin.",
  },
});
