// Copy for the Player screen (FR-043–FR-048, PB-1–PB-5).

import { defineMessages } from "../../i18n/message";

export const playerMessages = defineMessages({
  controlsHeading: { en: "Now playing", tr: "Şimdi çalıyor" },
  mixToggle: { en: "Soundscape", tr: "Ses manzarası" },
  programLink: { en: "Open program {name}", tr: "{name} programını aç" },
  skipBack: { en: "Back 10 seconds", tr: "10 saniye geri" },
  skipForward: { en: "Forward 10 seconds", tr: "10 saniye ileri" },
  seek: { en: "Playback position", tr: "Oynatma konumu" },
  seekValue: { en: "{elapsed} of {total}", tr: "{total} içinde {elapsed}" },
  elapsed: { en: "Elapsed", tr: "Geçen süre" },
  total: { en: "Total", tr: "Toplam süre" },
  speedButton: { en: "Speed {value}", tr: "Hız {value}" },
  speedValue: { en: "{value}×", tr: "{value}×" },
  restart: { en: "Restart", tr: "Başa dön" },
  moreActionsFor: {
    en: "More actions for {name}",
    tr: "{name} için diğer işlemler",
  },
  download: { en: "Download meditation", tr: "Meditasyonu indir" },
  playbackError: {
    en: "This meditation's audio could not be played. Notes, completion, and download still work.",
    tr: "Bu meditasyonun sesi oynatılamadı. Notlar, tamamlama ve indirme yine de çalışır.",
  },
  positionFailed: {
    en: "Your listening position could not be saved.",
    tr: "Dinleme konumunuz kaydedilemedi.",
  },
  completionFailed: {
    en: "The completion could not be recorded.",
    tr: "Tamamlama kaydedilemedi.",
  },
  loadFailed: {
    en: "Could not load this meditation. Try again.",
    tr: "Bu meditasyon yüklenemedi. Yeniden deneyin.",
  },
  notFoundTitle: {
    en: "Meditation not found",
    tr: "Meditasyon bulunamadı",
  },
  notFoundText: {
    en: "It may have been removed. Choose another one in the Library.",
    tr: "Kaldırılmış olabilir. Kütüphaneden başka bir meditasyon seçin.",
  },
  notesHeading: { en: "Notes", tr: "Notlar" },
  notesHint: {
    en: "Private to you, newest first.",
    tr: "Yalnızca size özel, en yenisi önce.",
  },
  noteLabel: { en: "New note", tr: "Yeni not" },
  notePlaceholder: { en: "How did it feel?", tr: "Nasıl hissettirdi?" },
  addNote: { en: "Add note", tr: "Not ekle" },
  noteSource: { en: "You", tr: "Siz" },
  noteAdded: { en: "Note added", tr: "Not eklendi" },
  noteAddFailed: {
    en: "The note could not be added. Try again.",
    tr: "Not eklenemedi. Yeniden deneyin.",
  },
  notesEmptyTitle: { en: "No notes yet", tr: "Henüz not yok" },
  notesEmptyText: {
    en: "Write down how this meditation felt.",
    tr: "Bu meditasyonun nasıl hissettirdiğini yazın.",
  },
  notesLoadFailed: {
    en: "Could not load your notes. Try again.",
    tr: "Notlarınız yüklenemedi. Yeniden deneyin.",
  },
  mixDialogTitle: { en: "Background mix", tr: "Arka plan karışımı" },
  mixDialogDescription: {
    en: "Choose the sounds that play behind the meditation.",
    tr: "Meditasyonun arkasında çalacak sesleri seçin.",
  },
  mixPlay: { en: "Play mix", tr: "Karışımı oynat" },
  mixStop: { en: "Stop mix", tr: "Karışımı durdur" },
  mixRandom: { en: "Random", tr: "Rastgele" },
  mixRandomLoaded: {
    en: "A random mix of {count} sounds is ready",
    tr: "{count} seslik rastgele karışım hazır",
  },
  uncategorized: { en: "Uncategorized", tr: "Kategorisiz" },
  mixBlocked: {
    en: "The browser blocked the sound. Press Play mix.",
    tr: "Tarayıcı sesi engelledi. Karışımı oynat'a basın.",
  },
  mixSavedHeading: { en: "Saved mixes", tr: "Kayıtlı karışımlar" },
  mixLoaded: { en: "Mix {name} loaded", tr: "{name} karışımı yüklendi" },
  mixAddHeading: { en: "Add a sound", tr: "Ses ekle" },
  mixLoading: { en: "Loading sounds", tr: "Sesler yükleniyor" },
  mixNoSounds: {
    en: "No sounds are available.",
    tr: "Kullanılabilir ses yok.",
  },
  volumeLabel: { en: "{name} volume", tr: "{name} ses düzeyi" },
  volumeValue: { en: "{value} percent", tr: "yüzde {value}" },
  removeSound: { en: "Remove {name}", tr: "{name} sesini kaldır" },
  addSound: { en: "Add {name}", tr: "{name} sesini ekle" },
  doneTitle: { en: "Well done", tr: "Tebrikler" },
  doneText: {
    en: "You finished this meditation.",
    tr: "Bu meditasyonu tamamladınız.",
  },
  doneNoteLabel: {
    en: "How did it feel? (optional)",
    tr: "Nasıl hissettirdi? (isteğe bağlı)",
  },
  doneContinue: { en: "Continue", tr: "Devam et" },
  doneContinueLibrary: {
    en: "Continue to Library",
    tr: "Kütüphaneye devam et",
  },
  doneSaved: { en: "Note saved", tr: "Not kaydedildi" },
  deleteNote: { en: "Delete note", tr: "Notu sil" },
  deleteNoteTitle: { en: "Delete this note?", tr: "Bu not silinsin mi?" },
  deleteNoteConsequence: {
    en: "The note is removed for good.",
    tr: "Not kalıcı olarak silinir.",
  },
  noteDeleted: { en: "Note deleted", tr: "Not silindi" },
});
