// Copy for the Home screen (FR-022–FR-030, FR-108–FR-111).

import { defineMessages } from "../../i18n/message";

type Named = { named: number };

export const homeMessages = defineMessages({
  greetingMorning: {
    en: ({ named }: Named) => (named ? "Good morning," : "Good morning"),
    tr: ({ named }: Named) => (named ? "Günaydın," : "Günaydın"),
  },
  greetingAfternoon: {
    en: ({ named }: Named) => (named ? "Good afternoon," : "Good afternoon"),
    tr: ({ named }: Named) => (named ? "İyi günler," : "İyi günler"),
  },
  greetingEvening: {
    en: ({ named }: Named) => (named ? "Good evening," : "Good evening"),
    tr: ({ named }: Named) => (named ? "İyi akşamlar," : "İyi akşamlar"),
  },

  recentHeading: { en: "Recent programs", tr: "Son programlar" },
  recentEmptyTitle: { en: "No recent programs", tr: "Son program yok" },
  recentEmptyText: {
    en: "Programs you listen to appear here.",
    tr: "Dinlediğiniz programlar burada görünür.",
  },
  recentFailed: {
    en: "Could not load recent programs.",
    tr: "Son programlar yüklenemedi.",
  },

  continueHeading: { en: "Continue", tr: "Devam et" },
  continueEmptyTitle: {
    en: "Nothing in progress",
    tr: "Devam eden bir şey yok",
  },
  continueEmptyText: {
    en: "Start a program from the Library.",
    tr: "Kütüphaneden bir programa başlayın.",
  },
  continueFailed: {
    en: "Could not load the program to continue.",
    tr: "Devam edilecek program yüklenemedi.",
  },
  playNext: { en: "Continue with {title}", tr: "Devam et: {title}" },

  suggestionHeading: { en: "Today's suggestion", tr: "Günün önerisi" },
  suggestionFailed: {
    en: "Could not load today's suggestion.",
    tr: "Günün önerisi yüklenemedi.",
  },

  noContentTitle: {
    en: "The host has not added meditations yet",
    tr: "Sunucu yöneticisi henüz meditasyon eklemedi",
  },
  noContentText: {
    en: "Meditations appear here once they are added.",
    tr: "Meditasyonlar eklendiğinde burada görünür.",
  },
  openLibrary: { en: "Go to Library", tr: "Kütüphaneye git" },
  openAdmin: { en: "Go to Admin", tr: "Yönetime git" },

  checkInHeading: { en: "Quick check-in", tr: "Hızlı kayıt" },
  checkInHint: {
    en: "Log how you feel right now",
    tr: "Şu an nasıl hissettiğinizi kaydedin",
  },
  scoreLegend: {
    en: "How are you feeling?",
    tr: "Nasıl hissediyorsunuz?",
  },
  noteLabel: { en: "Note (optional)", tr: "Not (isteğe bağlı)" },
  noteCount: {
    en: "{count} of {limit} characters",
    tr: "{count} / {limit} karakter",
  },
  noteTooLong: {
    en: "The note is {over} characters over the {limit} limit. Shorten it to save.",
    tr: "Not, {limit} karakter sınırını {over} karakter aşıyor. Kaydetmek için kısaltın.",
  },
  saved: { en: "Check-in saved.", tr: "Kayıt kaydedildi." },
  saveFailed: {
    en: "Could not save your check-in. Your score and note are kept.",
    tr: "Kaydınız kaydedilemedi. Puanınız ve notunuz korundu.",
  },

  moodHeading: { en: "Recent mood", tr: "Son ruh hâli" },
  moodScore: { en: "{score}/5 · {label}", tr: "{score}/5 · {label}" },
  moodEmptyTitle: { en: "No check-ins yet", tr: "Henüz kayıt yok" },
  moodEmptyText: {
    en: "Your first check-in will appear here.",
    tr: "İlk kaydınız burada görünecek.",
  },
  checkInNow: { en: "Check in now", tr: "Şimdi kaydet" },
  moodFailed: {
    en: "Could not load your recent mood entries.",
    tr: "Son ruh hâli kayıtlarınız yüklenemedi.",
  },
  openMood: { en: "Open Mood", tr: "Ruh hâlini aç" },
});
