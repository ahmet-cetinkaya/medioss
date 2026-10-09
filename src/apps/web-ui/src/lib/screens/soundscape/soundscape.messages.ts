// Copy for the Soundscape screen (FR-049–FR-054, PB-6/PB-7/PB-8/PB-16).

import { defineMessages } from "../../i18n/message";

export const soundscapeMessages = defineMessages({
  soundsHeading: { en: "Sounds", tr: "Sesler" },
  categoryLegend: { en: "Sound category", tr: "Ses kategorisi" },
  allCategories: { en: "All", tr: "Tümü" },
  addToMix: { en: "Add {name} to mix", tr: "{name} sesini karışıma ekle" },
  removeFromMix: {
    en: "Remove {name} from mix",
    tr: "{name} sesini karışımdan kaldır",
  },
  inMix: { en: "In mix", tr: "Karışımda" },
  moreActionsFor: {
    en: "More actions for {name}",
    tr: "{name} için diğer işlemler",
  },
  downloadSound: { en: "Download {name}", tr: "{name} sesini indir" },
  noSoundsTitle: { en: "No sounds yet", tr: "Henüz ses yok" },
  noSoundsText: {
    en: "The host has not added any sounds yet.",
    tr: "Sunucu yöneticisi henüz ses eklemedi.",
  },
  openAdmin: { en: "Add sounds in Admin", tr: "Yönetim'de ses ekle" },
  soundsLoadFailed: {
    en: "Could not load sounds. Try again.",
    tr: "Sesler yüklenemedi. Yeniden deneyin.",
  },

  mixHeading: { en: "Current mix", tr: "Geçerli karışım" },
  mixEmptyTitle: { en: "Your mix is empty", tr: "Karışımınız boş" },
  mixEmptyText: {
    en: "Add sounds from the list to start a mix.",
    tr: "Bir karışım başlatmak için listeden ses ekleyin.",
  },
  volumeLabel: { en: "{name} volume", tr: "{name} ses düzeyi" },
  masterVolume: { en: "Overall volume", tr: "Genel ses düzeyi" },
  volumeValue: { en: "{value} percent", tr: "yüzde {value}" },
  playMix: { en: "Play mix", tr: "Karışımı oynat" },
  stopMix: { en: "Stop", tr: "Durdur" },
  randomMixButton: { en: "Random mix", tr: "Rastgele karışım" },
  randomMixFilled: {
    en: (p: { count: number }) =>
      `Filled the mix with ${p.count} random ${new Intl.PluralRules("en").select(p.count) === "one" ? "sound" : "sounds"}.`,
    tr: (p: { count: number }) =>
      `Karışım rastgele ${p.count} sesle dolduruldu.`,
  },
  playBlocked: {
    en: "The browser blocked playback. Press Play mix to start.",
    tr: "Tarayıcı oynatmayı engelledi. Başlatmak için Karışımı oynat'a basın.",
  },
  missingSounds: {
    en: "Some sounds from this mix are no longer available.",
    tr: "Bu karışımdaki bazı sesler artık mevcut değil.",
  },

  timerButton: { en: "Sleep timer", tr: "Uyku zamanlayıcısı" },
  timerTitle: { en: "Sleep timer", tr: "Uyku zamanlayıcısı" },
  timerLabel: {
    en: "Minutes until the mix stops",
    tr: "Karışımın durmasına kalan dakika",
  },
  timerHint: {
    en: "Whole minutes from 5 to 120. The sound fades out over the last minute.",
    tr: "5 ile 120 arasında tam dakika. Ses son dakikada giderek kısılır.",
  },
  timerInvalid: {
    en: "Enter whole minutes from 5 to 120.",
    tr: "5 ile 120 arasında tam dakika girin.",
  },
  timerUnavailable: {
    en: "The sleep timer is available while the mix plays.",
    tr: "Uyku zamanlayıcısı karışım çalarken kullanılabilir.",
  },
  startTimer: { en: "Start timer", tr: "Zamanlayıcıyı başlat" },
  cancelTimer: { en: "Cancel timer", tr: "Zamanlayıcıyı iptal et" },
  timeLeft: { en: "Time left", tr: "Kalan süre" },

  mixNameLabel: { en: "Mix name", tr: "Karışım adı" },
  saveMix: { en: "Save mix", tr: "Karışımı kaydet" },
  saveTitle: { en: "Save mix", tr: "Karışımı kaydet" },
  saveUnavailable: {
    en: "Add at least one sound to save the mix.",
    tr: "Karışımı kaydetmek için en az bir ses ekleyin.",
  },
  soundsInMix: {
    en: (p: { count: number }) =>
      `${p.count} ${new Intl.PluralRules("en").select(p.count) === "one" ? "sound" : "sounds"} playing together`,
    tr: (p: { count: number }) => `Birlikte çalan ${p.count} ses`,
  },
  saveFailed: {
    en: "Could not save the mix. Try again.",
    tr: "Karışım kaydedilemedi. Yeniden deneyin.",
  },
  mixSaved: { en: "Saved {name}.", tr: "{name} kaydedildi." },

  savedHeading: { en: "Saved mixes", tr: "Kayıtlı karışımlar" },
  noMixesTitle: { en: "No saved mixes", tr: "Kayıtlı karışım yok" },
  noMixesText: {
    en: "Add sounds, set their volumes and press Save mix to name it.",
    tr: "Ses ekleyin, ses düzeylerini ayarlayın ve adlandırmak için Karışımı kaydet'e basın.",
  },
  mixesLoadFailed: {
    en: "Could not load your saved mixes. Try again.",
    tr: "Kayıtlı karışımlarınız yüklenemedi. Yeniden deneyin.",
  },
  soundCount: {
    en: (p: { count: number }) =>
      `${p.count} ${new Intl.PluralRules("en").select(p.count) === "one" ? "sound" : "sounds"}`,
    tr: (p: { count: number }) => `${p.count} ses`,
  },
  loadMix: { en: "Load {name}", tr: "{name} karışımını yükle" },
  load: { en: "Load", tr: "Yükle" },
  mixLoaded: { en: "Loaded {name}.", tr: "{name} yüklendi." },
  renameMix: { en: "Rename {name}", tr: "{name} karışımını yeniden adlandır" },
  renameTitle: { en: "Rename mix", tr: "Karışımı yeniden adlandır" },
  renameFailed: {
    en: "Could not rename the mix. Try again.",
    tr: "Karışım yeniden adlandırılamadı. Yeniden deneyin.",
  },
  deleteMix: { en: "Delete {name}", tr: "{name} karışımını sil" },
  deleteTitle: { en: "Delete this mix?", tr: "Bu karışım silinsin mi?" },
  deleteConsequence: {
    en: "The saved mix is removed. Its sounds stay in the library.",
    tr: "Kayıtlı karışım kaldırılır. Sesleri kütüphanede kalır.",
  },
  deleteConfirm: { en: "Delete mix", tr: "Karışımı sil" },
  deleteFailed: {
    en: "Could not delete the mix. Try again.",
    tr: "Karışım silinemedi. Yeniden deneyin.",
  },
  mixDeleted: { en: "Deleted {name}.", tr: "{name} silindi." },
});
