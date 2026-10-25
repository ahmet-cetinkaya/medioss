// Shared UI copy: navigation, screen titles, generic actions and states (plan D3).

import { defineMessages } from "./message";

export const brandMessages = defineMessages({
  productName: { en: "MediOSS", tr: "MediOSS" },
});

export const navGroupMessages = defineMessages({
  practice: { en: "Practice", tr: "Pratik" },
  progress: { en: "Progress", tr: "İlerleme" },
  account: { en: "Account", tr: "Hesap" },
  project: { en: "MediOSS", tr: "MediOSS" },
});

export const navMessages = defineMessages({
  home: { en: "Home", tr: "Ana sayfa" },
  library: { en: "Library", tr: "Kütüphane" },
  soundscape: { en: "Soundscape", tr: "Ses Manzarası" },
  mood: { en: "Mood", tr: "Ruh hâli" },
  history: { en: "History", tr: "Geçmiş" },
  statistics: { en: "Statistics", tr: "İstatistikler" },
  settings: { en: "Settings", tr: "Ayarlar" },
  about: { en: "About", tr: "Hakkında" },
  admin: { en: "Admin", tr: "Yönetim" },
  signOut: { en: "Sign out", tr: "Çıkış yap" },
  more: { en: "More", tr: "Daha fazla" },
});

export const screenTitleMessages = defineMessages({
  home: { en: "Home", tr: "Ana sayfa" },
  library: { en: "Library", tr: "Kütüphane" },
  program: { en: "Program", tr: "Program" },
  player: { en: "Player", tr: "Oynatıcı" },
  soundscape: { en: "Soundscape", tr: "Ses Manzarası" },
  mood: { en: "Mood", tr: "Ruh hâli" },
  history: { en: "History", tr: "Geçmiş" },
  statistics: { en: "Statistics", tr: "İstatistikler" },
  settings: { en: "Settings", tr: "Ayarlar" },
  about: { en: "About", tr: "Hakkında" },
  admin: { en: "Admin", tr: "Yönetim" },
  login: { en: "Sign in", tr: "Giriş yap" },
  setup: { en: "Setup", tr: "Kurulum" },
  changePassword: { en: "Change password", tr: "Şifreyi değiştir" },
});

export const actionMessages = defineMessages({
  save: { en: "Save", tr: "Kaydet" },
  cancel: { en: "Cancel", tr: "İptal" },
  retry: { en: "Retry", tr: "Yeniden dene" },
  delete: { en: "Delete", tr: "Sil" },
  rename: { en: "Rename", tr: "Yeniden adlandır" },
  download: { en: "Download", tr: "İndir" },
  loadMore: { en: "Load more", tr: "Daha fazla yükle" },
  clearSearch: { en: "Clear search", tr: "Aramayı temizle" },
  seeAll: { en: "See all", tr: "Tümünü gör" },
  close: { en: "Close", tr: "Kapat" },
  backToLibrary: { en: "Back to Library", tr: "Kütüphaneye dön" },
  backToHome: { en: "Back to Home", tr: "Ana sayfaya dön" },
  edit: { en: "Edit", tr: "Düzenle" },
  add: { en: "Add", tr: "Ekle" },
  remove: { en: "Remove", tr: "Kaldır" },
  upload: { en: "Upload", tr: "Yükle" },
  copy: { en: "Copy", tr: "Kopyala" },
  play: { en: "Play", tr: "Oynat" },
  pause: { en: "Pause", tr: "Duraklat" },
  confirm: { en: "Confirm", tr: "Onayla" },
  continue: { en: "Continue", tr: "Devam et" },
  more: { en: "More", tr: "Daha fazla" },
  signOut: { en: "Sign out", tr: "Çıkış yap" },
});

export const stateMessages = defineMessages({
  loading: { en: "Loading…", tr: "Yükleniyor…" },
  somethingWentWrong: {
    en: "Something went wrong",
    tr: "Bir şeyler ters gitti",
  },
  sectionLoadFailed: {
    en: "Could not load this section",
    tr: "Bu bölüm yüklenemedi",
  },
  tryAgain: { en: "Try again", tr: "Tekrar dene" },
  empty: { en: "Nothing here yet", tr: "Burada henüz bir şey yok" },
  notFound: { en: "Not found", tr: "Bulunamadı" },
});
