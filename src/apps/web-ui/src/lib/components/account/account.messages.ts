// Copy for the password change form and the required change view (T12).

import { defineMessages } from "../../i18n/message";

export const passwordMessages = defineMessages({
  newPassword: { en: "New password", tr: "Yeni parola" },
  confirmPassword: {
    en: "Confirm new password",
    tr: "Yeni parolayı onaylayın",
  },
  hint: {
    en: "At least {min} characters.",
    tr: "En az {min} karakter.",
  },
  tooShort: {
    en: "The password must be at least {min} characters long.",
    tr: "Parola en az {min} karakter uzunluğunda olmalıdır.",
  },
  mismatch: {
    en: "The two passwords do not match.",
    tr: "Girilen iki parola eşleşmiyor.",
  },
  submit: { en: "Change password", tr: "Parolayı değiştir" },
  changed: {
    en: "Your password has been changed.",
    tr: "Parolanız değiştirildi.",
  },
});

export const requiredPasswordMessages = defineMessages({
  explanation: {
    en: "Your account was created with a temporary password. Choose a new password to continue.",
    tr: "Hesabınız geçici bir parolayla oluşturuldu. Devam etmek için yeni bir parola belirleyin.",
  },
});
