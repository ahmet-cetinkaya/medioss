// Copy for the Admin users section.

import { defineMessages } from "../../i18n/message";

export const userMessages = defineMessages({
  createHeading: { en: "Create user", tr: "Kullanıcı oluştur" },
  email: { en: "Email", tr: "E-posta" },
  temporaryPassword: { en: "Temporary password", tr: "Geçici şifre" },
  temporaryPasswordHint: {
    en: "At least 8 characters. The user changes it at first sign-in.",
    tr: "En az 8 karakter. Kullanıcı ilk girişte değiştirir.",
  },
  create: { en: "Create user", tr: "Kullanıcı oluştur" },
  created: {
    en: "{email} was created.",
    tr: "{email} oluşturuldu.",
  },
  shownOnce: {
    en: "This temporary password is shown only once. Share it privately; {email} must change it at first sign-in.",
    tr: "Bu geçici şifre yalnızca bir kez gösterilir. Gizlice paylaşın; {email} ilk girişte şifresini değiştirmelidir.",
  },
  copied: { en: "Password copied.", tr: "Şifre kopyalandı." },
  copySelected: {
    en: "Password selected. Press Ctrl+C or ⌘C to copy it.",
    tr: "Şifre seçildi. Kopyalamak için Ctrl+C veya ⌘C tuşlarına basın.",
  },
  listHeading: { en: "Users", tr: "Kullanıcılar" },
  roleAdmin: { en: "Administrator", tr: "Yönetici" },
  roleMember: { en: "Member", tr: "Üye" },
  statusDisabled: { en: "Disabled", tr: "Devre dışı" },
  statusMustChange: {
    en: "Must change password",
    tr: "Şifresini değiştirmeli",
  },
  statusActive: { en: "Active", tr: "Etkin" },
  roleStatus: { en: "{role} · {status}", tr: "{role} · {status}" },
  disable: { en: "Disable", tr: "Devre dışı bırak" },
  disableNamed: { en: "Disable {email}", tr: "{email}: devre dışı bırak" },
  enable: { en: "Enable", tr: "Etkinleştir" },
  enableNamed: { en: "Enable {email}", tr: "{email}: etkinleştir" },
  deleteNamed: { en: "Delete {email}", tr: "{email}: sil" },
  enabled: { en: "{email} was enabled.", tr: "{email} etkinleştirildi." },
  disableTitle: {
    en: "Disable this user?",
    tr: "Bu kullanıcı devre dışı bırakılsın mı?",
  },
  disableConsequence: {
    en: "They will be signed out immediately and cannot sign in until you enable them again.",
    tr: "Hemen oturumları kapatılır ve siz yeniden etkinleştirene kadar giriş yapamazlar.",
  },
  disableConfirm: { en: "Disable user", tr: "Kullanıcıyı devre dışı bırak" },
  disabled: {
    en: "{email} was disabled.",
    tr: "{email} devre dışı bırakıldı.",
  },
  deleteTitle: { en: "Delete this user?", tr: "Bu kullanıcı silinsin mi?" },
  deleteConsequence: {
    en: "This permanently erases all of this user's data: profile, mood entries, notes, history, and mixes. It cannot be undone.",
    tr: "Bu işlem kullanıcının tüm verilerini kalıcı olarak siler: profil, ruh hâli kayıtları, notlar, geçmiş ve karışımlar. Geri alınamaz.",
  },
  deleteConfirm: { en: "Delete user", tr: "Kullanıcıyı sil" },
  deleted: { en: "{email} was deleted.", tr: "{email} silindi." },
  noUsers: { en: "No users yet", tr: "Henüz kullanıcı yok" },
});
