// User-facing text for every server refusal reason (plan D8). Keys match
// `ServerReason` in `src/lib/api/server-reason.ts`; `generic` is a fallback,
// screens usually show their own action-specific failure text instead.

import { defineMessages } from "./message";

export const serverReasonMessages = defineMessages({
  emailExists: {
    en: "An account with this email already exists.",
    tr: "Bu e-posta adresiyle bir hesap zaten var.",
  },
  setupAlreadyCompleted: {
    en: "This instance is already set up. Sign in instead.",
    tr: "Bu kurulum zaten tamamlanmış. Bunun yerine giriş yapın.",
  },
  setupForbidden: {
    en: "The setup secret is not correct, or this address is not the instance's configured origin.",
    tr: "Kurulum anahtarı doğru değil ya da bu adres, bu kurulum için yapılandırılmış adres değil.",
  },
  lastActiveAdmin: {
    en: "You are the last active administrator, so this would leave the instance without one.",
    tr: "Son etkin yönetici sizsiniz; bu işlem kurulumu yöneticisiz bırakır.",
  },
  selfDisable: {
    en: "You cannot disable your own account.",
    tr: "Kendi hesabınızı devre dışı bırakamazsınız.",
  },
  passwordTooShort: {
    en: "The password must be at least 8 characters long.",
    tr: "Parola en az 8 karakter uzunluğunda olmalıdır.",
  },
  emailInvalid: {
    en: "Enter a valid email address.",
    tr: "Geçerli bir e-posta adresi girin.",
  },
  avatarTooLarge: {
    en: "The image is larger than 2 MB.",
    tr: "Görsel 2 MB'tan büyük.",
  },
  descriptionTooLong: {
    en: "The description must be at most 2000 characters.",
    tr: "Açıklama en fazla 2000 karakter olmalıdır.",
  },
  avatarInvalid: {
    en: "This image could not be used. Try a different picture.",
    tr: "Bu görsel kullanılamadı. Başka bir resim deneyin.",
  },
  categoryInUse: {
    en: "This category still contains sounds. Move or delete them first.",
    tr: "Bu kategoride hâlâ sesler var. Önce onları taşıyın veya silin.",
  },
  reorderMismatch: {
    en: "This program's meditations changed. Reload and try again.",
    tr: "Bu programın meditasyonları değişti. Sayfayı yenileyip tekrar deneyin.",
  },
  duplicateName: {
    en: "That name is already in use.",
    tr: "Bu ad zaten kullanılıyor.",
  },
  unsupportedMedia: {
    en: "This file type is not supported.",
    tr: "Bu dosya türü desteklenmiyor.",
  },
  fileTooLarge: {
    en: "The file is larger than {limitMb} MB.",
    tr: "Dosya {limitMb} MB'tan büyük.",
  },
  importMissingFile: {
    en: "Choose a file to import.",
    tr: "İçe aktarılacak bir dosya seçin.",
  },
  importNotExportFile: {
    en: "This file is not a MediOSS export.",
    tr: "Bu dosya bir MediOSS dışa aktarımı değil.",
  },
  importUnknownVersion: {
    en: "This export comes from a version this instance does not understand.",
    tr: "Bu dışa aktarım, bu kurulumun tanımadığı bir sürümden geliyor.",
  },
  importNotCsv: {
    en: "This file is not a valid CSV export.",
    tr: "Bu dosya geçerli bir CSV dışa aktarımı değil.",
  },
  importBadColumns: {
    en: "Each CSV row must have exactly 3 columns.",
    tr: "Her CSV satırında tam olarak 3 sütun olmalıdır.",
  },
  importInvalidRecordedAt: {
    en: "The file contains an invalid date.",
    tr: "Dosyada geçersiz bir tarih var.",
  },
  importInvalidScore: {
    en: "The file contains a mood score outside 1 to 5.",
    tr: "Dosyada 1 ile 5 arasında olmayan bir ruh hâli puanı var.",
  },
  importFutureEntry: {
    en: "The file contains a mood entry dated in the future.",
    tr: "Dosyada gelecek tarihli bir ruh hâli kaydı var.",
  },
  importNoteTooLong: {
    en: "The file contains a mood note longer than 10,000 characters.",
    tr: "Dosyada 10.000 karakterden uzun bir ruh hâli notu var.",
  },
  importEmptyNote: {
    en: "The file contains an empty note.",
    tr: "Dosyada boş bir not var.",
  },
  importTooLarge: {
    en: "The import file is larger than 10 MB.",
    tr: "İçe aktarma dosyası 10 MB'tan büyük.",
  },
  importInvalid: {
    en: "This file could not be imported.",
    tr: "Bu dosya içe aktarılamadı.",
  },
  invalidCredentials: {
    en: "The email or password is not correct.",
    tr: "E-posta veya parola doğru değil.",
  },
  throttled: {
    en: "Too many sign-in attempts. Wait a moment and try again.",
    tr: "Çok fazla giriş denemesi yapıldı. Biraz bekleyip yeniden deneyin.",
  },
  notFound: {
    en: "This item no longer exists.",
    tr: "Bu öğe artık mevcut değil.",
  },
  network: {
    en: "Could not reach the server. Check your connection and try again.",
    tr: "Sunucuya ulaşılamadı. Bağlantınızı kontrol edip yeniden deneyin.",
  },
  server: {
    en: "Something went wrong on the server. Try again later.",
    tr: "Sunucuda bir sorun oluştu. Daha sonra yeniden deneyin.",
  },
  generic: {
    en: "That did not work. Try again.",
    tr: "İşlem tamamlanamadı. Yeniden deneyin.",
  },
});
