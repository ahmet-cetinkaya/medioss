// Copy for the About screen.

import { defineMessages } from "../../i18n/message";

export const aboutMessages = defineMessages({
  whatHeading: { en: "What is it?", tr: "Nedir?" },
  whatBody: {
    en: "A self-hosted wellness app for calmer days: a meditation library, an ambient soundscape mixer and a mood tracker, all behind one account on your own server.",
    tr: "Daha sakin günler için kendi sunucunuzda çalışan bir iyi oluş uygulaması: meditasyon kütüphanesi, ortam sesi karıştırıcı ve ruh hâli takibi; hepsi tek bir hesabın arkasında.",
  },
  whyHeading: { en: "Why?", tr: "Niçin?" },
  whyBody: {
    en: "So your practice and your moods stay yours: no third-party services, no tracking, no accounts elsewhere. Your data lives in one file you control.",
    tr: "Pratiğiniz ve ruh hâliniz size ait kalsın diye: üçüncü taraf hizmet yok, izleme yok, başka yerde hesap yok. Verileriniz sizin denetlediğiniz tek bir dosyada durur.",
  },
  howHeading: { en: "How does it work?", tr: "Nasıl çalışır?" },
  howBody: {
    en: "One program serves the web interface and the API from a single SQLite database. Run it with Docker or from source; accounts are created by the administrator.",
    tr: "Tek bir program, web arayüzünü ve API'yi tek bir SQLite veritabanından sunar. Docker ile ya da kaynaktan çalıştırılır; hesapları yönetici oluşturur.",
  },
  whoHeading: { en: "Who builds it?", tr: "Kim geliştiriyor?" },
  whoBody: {
    en: "Ahmet Çetinkaya and the MediOSS contributors, as open source software under the AGPL.",
    tr: "Ahmet Çetinkaya ve MediOSS katkıcıları; AGPL lisanslı, açık kaynaklı bir yazılım olarak.",
  },
  version: { en: "Version", tr: "Sürüm" },
  license: { en: "License", tr: "Lisans" },
  openSource: { en: "Open source", tr: "Açık kaynak" },
  licenseFullText: {
    en: "Read the full license",
    tr: "Lisansın tam metnini oku",
  },
  thirdPartyOpen: {
    en: "Third-party licenses",
    tr: "Üçüncü taraf lisansları",
  },
  loadFailed: {
    en: "Version information could not be loaded.",
    tr: "Sürüm bilgisi yüklenemedi.",
  },
});
