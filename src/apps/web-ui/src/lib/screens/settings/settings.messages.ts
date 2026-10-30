// Copy for the Settings screen (FR-066–FR-070, PB-12).

import { defineMessages } from "../../i18n/message";

export const profileMessages = defineMessages({
  heading: { en: "Profile", tr: "Profil" },
  intro: {
    en: "Only you and the administrators see your display name and picture.",
    tr: "Görünen adınızı ve resminizi yalnızca siz ve yöneticiler görür.",
  },
  pictureAlt: { en: "Your profile picture", tr: "Profil resminiz" },
  uploadPicture: { en: "Upload picture", tr: "Resim yükle" },
  removePicture: { en: "Remove picture", tr: "Resmi kaldır" },
  pictureRules: {
    en: "JPEG, PNG, or WebP, at most 2 MB.",
    tr: "JPEG, PNG veya WebP; en fazla 2 MB.",
  },
  wrongType: {
    en: "This file is not a JPEG, PNG, or WebP image. Choose an image of one of these types, at most 2 MB.",
    tr: "Bu dosya JPEG, PNG veya WebP görseli değil. Bu türlerden birinde, en fazla 2 MB'lık bir görsel seçin.",
  },
  tooLarge: {
    en: "This image is larger than 2 MB. Choose a JPEG, PNG, or WebP image of at most 2 MB.",
    tr: "Bu görsel 2 MB'tan büyük. En fazla 2 MB'lık bir JPEG, PNG veya WebP görseli seçin.",
  },
  uploadFailed: {
    en: "The picture could not be uploaded. Try again.",
    tr: "Resim yüklenemedi. Yeniden deneyin.",
  },
  uploaded: {
    en: "Your profile picture has been updated.",
    tr: "Profil resminiz güncellendi.",
  },
  removeTitle: {
    en: "Remove your profile picture?",
    tr: "Profil resminiz kaldırılsın mı?",
  },
  removeConsequence: {
    en: "The account button will show your initial instead.",
    tr: "Hesap düğmesinde bunun yerine adınızın baş harfi görünür.",
  },
  removeFailed: {
    en: "The picture could not be removed. Try again.",
    tr: "Resim kaldırılamadı. Yeniden deneyin.",
  },
  removed: {
    en: "Your profile picture has been removed.",
    tr: "Profil resminiz kaldırıldı.",
  },
  displayName: { en: "Display name", tr: "Görünen ad" },
  displayNameHint: {
    en: "Up to 64 characters. Leave it empty to remove your display name.",
    tr: "En fazla 64 karakter. Görünen adınızı kaldırmak için boş bırakın.",
  },
  saveName: { en: "Save display name", tr: "Görünen adı kaydet" },
  nameSaved: {
    en: "Your display name has been saved.",
    tr: "Görünen adınız kaydedildi.",
  },
  nameFailed: {
    en: "The display name could not be saved. Try again.",
    tr: "Görünen ad kaydedilemedi. Yeniden deneyin.",
  },
});

export const languagePanelMessages = defineMessages({
  heading: { en: "Language", tr: "Dil" },
  intro: {
    en: "Your choice is remembered on this device.",
    tr: "Seçiminiz bu cihazda hatırlanır.",
  },
});

export const passwordPanelMessages = defineMessages({
  heading: { en: "Password", tr: "Parola" },
  change: { en: "Change password", tr: "Parolayı değiştir" },
  intro: {
    en: "You stay signed in here; your other sessions are signed out.",
    tr: "Burada oturumunuz açık kalır; diğer oturumlarınız kapatılır.",
  },
});

export const dataMessages = defineMessages({
  heading: { en: "Your data", tr: "Verileriniz" },
  intro: {
    en: "Download a copy of your data, or bring back a file you exported earlier.",
    tr: "Verilerinizin bir kopyasını indirin ya da daha önce dışa aktardığınız bir dosyayı geri yükleyin.",
  },
  exportJson: {
    en: "Download all data (JSON)",
    tr: "Tüm verileri indir (JSON)",
  },
  exportCsv: {
    en: "Download mood entries only (CSV)",
    tr: "Yalnızca ruh hâli kayıtlarını indir (CSV)",
  },
  importLabel: {
    en: "Import a previous export (JSON or CSV)",
    tr: "Önceki bir dışa aktarımı içe aktar (JSON veya CSV)",
  },
  importAction: { en: "Import", tr: "İçe aktar" },
  importFailed: {
    en: "The file could not be imported. Try again.",
    tr: "Dosya içe aktarılamadı. Yeniden deneyin.",
  },
  reportHeading: { en: "Import report", tr: "İçe aktarma raporu" },
  imported: { en: "The import is complete.", tr: "İçe aktarma tamamlandı." },
  moodCreated: { en: "Mood entries added", tr: "Eklenen ruh hâli kayıtları" },
  moodDuplicates: {
    en: "Mood entries skipped as duplicates",
    tr: "Yinelendiği için atlanan ruh hâli kayıtları",
  },
  practiceLinked: {
    en: "Practice records linked",
    tr: "Eşleştirilen pratik kayıtları",
  },
  practiceSkipped: {
    en: "Practice records skipped (no matching meditation)",
    tr: "Atlanan pratik kayıtları (eşleşen meditasyon yok)",
  },
  practiceDuplicates: {
    en: "Practice records skipped as duplicates",
    tr: "Yinelendiği için atlanan pratik kayıtları",
  },
});

export const deleteAccountMessages = defineMessages({
  heading: { en: "Delete account", tr: "Hesabı sil" },
  consequence: {
    en: "Your account, mood entries, meditation history, notes, mixes, and picture are deleted permanently. This cannot be undone.",
    tr: "Hesabınız, ruh hâli kayıtlarınız, meditasyon geçmişiniz, notlarınız, mix'leriniz ve resminiz kalıcı olarak silinir. Bu işlem geri alınamaz.",
  },
  typeEmail: {
    en: "To confirm, type your email address: {email}",
    tr: "Onaylamak için e-posta adresinizi yazın: {email}",
  },
  action: { en: "Delete account", tr: "Hesabı sil" },
  dialogTitle: {
    en: "Delete your account?",
    tr: "Hesabınız silinsin mi?",
  },
  failed: {
    en: "Your account could not be deleted. Try again.",
    tr: "Hesabınız silinemedi. Yeniden deneyin.",
  },
});
