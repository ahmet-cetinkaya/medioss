// Copy for the app shell, the unauthenticated layout, sign-out, and the
// language switcher (T12). Nav labels and screen titles live in
// common.messages.ts.

import { defineMessages } from "../../i18n/message";

export const shellMessages = defineMessages({
  documentTitle: { en: "{screen} · {product}", tr: "{screen} · {product}" },
  skipToContent: { en: "Skip to main content", tr: "Ana içeriğe geç" },
  sidebarNav: { en: "Main navigation", tr: "Ana gezinme" },
  railNav: { en: "Compact navigation", tr: "Kompakt gezinme" },
  bottomNav: { en: "Bottom navigation", tr: "Alt gezinme" },
  support: { en: "Support MediOSS", tr: "MediOSS'a destek ol" },
  supportTitle: {
    en: "Free, ad-free, yours",
    tr: "Ücretsiz, reklamsız, sizin",
  },
  supportStory: {
    en: "MediOSS is free software built in spare time. It shows no ads and tracks nothing.",
    tr: "MediOSS boş zamanlarda geliştirilen özgür bir yazılımdır. Reklam göstermez, sizi izlemez.",
  },
  supportFunds: {
    en: "If it helps you, a coffee supports its ongoing development.",
    tr: "İşinize yarıyorsa bir kahve, geliştirmenin sürmesine destek olur.",
  },
  supportAction: { en: "Buy me a coffee", tr: "Bana bir kahve ısmarla" },
  moreNav: { en: "More pages", tr: "Diğer sayfalar" },
  aside: { en: "Side panel", tr: "Yan panel" },
  searchLabel: { en: "Search the library", tr: "Kütüphanede ara" },
  searchPlaceholder: { en: "Search…", tr: "Ara…" },
  adminsOnlyTitle: { en: "Admins only", tr: "Yalnızca yöneticiler" },
  adminsOnlyText: {
    en: "This screen is available to administrators only.",
    tr: "Bu ekran yalnızca yöneticilere açıktır.",
  },
});

export const signOutMessages = defineMessages({
  title: { en: "Sign out?", tr: "Çıkış yapılsın mı?" },
  consequence: {
    en: "You will need to sign in again on this device.",
    tr: "Bu cihazda yeniden giriş yapmanız gerekecek.",
  },
  failed: {
    en: "Sign-out did not complete. Try again.",
    tr: "Çıkış tamamlanamadı. Yeniden deneyin.",
  },
});

/** Language names are proper names: each is written in its own language. */
export const languageMessages = defineMessages({
  legend: { en: "Language", tr: "Dil" },
  english: { en: "English", tr: "English" },
  turkish: { en: "Türkçe", tr: "Türkçe" },
});
