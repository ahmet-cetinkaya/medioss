// Copy for the Setup screen (FR-083, FR-084, PB-15).

import { defineMessages } from "../../i18n/message";

export const setupMessages = defineMessages({
  intro: {
    en: "Create the first administrator account for this instance.",
    tr: "Bu kurulum için ilk yönetici hesabını oluşturun.",
  },
  secret: { en: "Setup secret", tr: "Kurulum anahtarı" },
  secretHint: {
    en: "The SESSION_SECRET value this instance runs with. It is sent once and never stored by the browser.",
    tr: "Bu kurulumun çalıştığı SESSION_SECRET değeri. Yalnızca bir kez gönderilir ve tarayıcıda hiçbir zaman saklanmaz.",
  },
  email: { en: "Administrator email", tr: "Yönetici e-postası" },
  password: { en: "Administrator password", tr: "Yönetici parolası" },
  passwordHint: {
    en: "At least {min} characters.",
    tr: "En az {min} karakter.",
  },
  passwordTooShort: {
    en: "The password must be at least {min} characters long.",
    tr: "Parola en az {min} karakter uzunluğunda olmalıdır.",
  },
  submit: { en: "Create administrator", tr: "Yönetici oluştur" },
  failed: {
    en: "Setup did not complete. Try again.",
    tr: "Kurulum tamamlanamadı. Yeniden deneyin.",
  },
  statusFailed: {
    en: "Could not check whether this instance is set up.",
    tr: "Bu kurulumun yapılandırılıp yapılandırılmadığı denetlenemedi.",
  },
  alreadySetUp: {
    en: "This instance is already set up.",
    tr: "Bu kurulum zaten yapılandırılmış.",
  },
  done: {
    en: "The administrator account was created. You can sign in now.",
    tr: "Yönetici hesabı oluşturuldu. Artık giriş yapabilirsiniz.",
  },
  goToLogin: { en: "Go to sign in", tr: "Girişe git" },
});
