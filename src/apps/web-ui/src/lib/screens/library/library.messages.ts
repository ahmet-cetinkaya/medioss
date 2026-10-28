// Copy for the Library screen (FR-031–FR-037).

import { defineMessages } from "../../i18n/message";

export const libraryMessages = defineMessages({
  filterLegend: { en: "Show programs", tr: "Programları göster" },
  all: { en: "All", tr: "Tümü" },
  filterCompleted: { en: "Completed", tr: "Bitirilenler" },
  filterInProgress: { en: "In progress", tr: "Devam edenler" },
  filterNotStarted: { en: "Not started", tr: "Hiç başlanmamışlar" },
  filterFavorites: { en: "Favorites", tr: "Favoriler" },
  favoritesFailed: {
    en: "Could not load your favorites. Try again.",
    tr: "Favorileriniz yüklenemedi. Yeniden deneyin.",
  },
  favoritesEmptyTitle: { en: "No favorites yet", tr: "Henüz favori yok" },
  favoritesEmptyText: {
    en: "Tap the heart in the Player to save a meditation here.",
    tr: "Bir meditasyonu buraya kaydetmek için Oynatıcı'daki kalbe dokunun.",
  },
  favoriteProgram: { en: "Program: {names}", tr: "Program: {names}" },
  favoritePrograms: { en: "Programs: {names}", tr: "Programlar: {names}" },
  filterEmptyTitle: {
    en: "No programs here",
    tr: "Burada program yok",
  },
  filterEmptyText: {
    en: "No programs match this filter. Pick another one to see more.",
    tr: "Bu filtreyle eşleşen program yok. Daha fazlasını görmek için başka bir filtre seçin.",
  },
  uncategorized: { en: "Uncategorized", tr: "Kategorisiz" },
  programsLabel: { en: "Programs", tr: "Programlar" },
  emptyTitle: { en: "No programs yet", tr: "Henüz program yok" },
  emptyText: {
    en: "The host has not added any programs yet.",
    tr: "Sunucu yöneticisi henüz program eklemedi.",
  },
  openAdmin: { en: "Add programs in Admin", tr: "Yönetim'de program ekle" },
  resultsHeading: {
    en: "Results for “{query}”",
    tr: "“{query}” için sonuçlar",
  },
  resultCount: {
    en: ({ count }: { count: number }) =>
      new Intl.PluralRules("en").select(count) === "one"
        ? `${count} meditation found`
        : `${count} meditations found`,
    tr: ({ count }: { count: number }) => `${count} meditasyon bulundu`,
  },
  noMatchesTitle: {
    en: "No meditations match “{query}”",
    tr: "“{query}” ile eşleşen meditasyon yok",
  },
  noMatchesText: {
    en: "Try another word, or clear the search to browse programs.",
    tr: "Başka bir kelime deneyin ya da programlara göz atmak için aramayı temizleyin.",
  },
  moreActionsFor: {
    en: "More actions for {name}",
    tr: "{name} için diğer işlemler",
  },
  downloadNamed: { en: "Download {name}", tr: "{name} indir" },
  loadFailed: {
    en: "Could not load the library. Try again.",
    tr: "Kütüphane yüklenemedi. Yeniden deneyin.",
  },
  searchFailed: {
    en: "Could not search the library. Try again.",
    tr: "Kütüphanede arama yapılamadı. Yeniden deneyin.",
  },
});
