// Copy for the Login screen (FR-080–FR-082, PB-14).

import { defineMessages } from "../../i18n/message";

/** The operator CLI command named in the help text (`medioss reset-password <email>`). */
export const RESET_PASSWORD_COMMAND = "medioss reset-password";

export const loginMessages = defineMessages({
  email: { en: "Email", tr: "E-posta" },
  password: { en: "Password", tr: "Parola" },
  submit: { en: "Sign in", tr: "Giriş yap" },
  failed: {
    en: "Sign-in did not complete. Try again.",
    tr: "Giriş tamamlanamadı. Yeniden deneyin.",
  },
  throttled: {
    en: "Too many sign-in attempts. Try again in {time}.",
    tr: "Çok fazla giriş denemesi yapıldı. {time} sonra yeniden deneyin.",
  },
  throttledAnnouncement: {
    en: "Too many sign-in attempts. You can try again in {time}.",
    tr: "Çok fazla giriş denemesi yapıldı. {time} sonra yeniden deneyebilirsiniz.",
  },
  throttleEnded: {
    en: "You can sign in again.",
    tr: "Yeniden giriş yapabilirsiniz.",
  },
  // `{command}` is replaced by the command in code style.
  resetHelp: {
    en: "Forgot your password? Ask the host operator to run {command} with your email.",
    tr: "Parolanızı mı unuttunuz? Sunucu yöneticisinden e-posta adresinizle {command} komutunu çalıştırmasını isteyin.",
  },
  setupNeeded: {
    en: "This instance is not set up yet.",
    tr: "Bu kurulum henüz yapılandırılmadı.",
  },
  setupLink: { en: "Go to Setup", tr: "Kuruluma git" },
});
