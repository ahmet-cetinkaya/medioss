// Copy for the About screen.

import { defineMessages } from "../../i18n/message";

export const aboutMessages = defineMessages({
  description: {
    en: "MediOSS is a self-hosted wellness app for calmer days: a meditation library, an ambient soundscape mixer and a mood tracker, all behind one account on your own server. It exists so your practice and your moods stay yours, with no third-party services and no tracking. One program serves the web interface and the API from a single SQLite database, and the administrator creates the accounts. It is built by Ahmet Çetinkaya and the MediOSS contributors as open source software under the AGPL.",
    tr: "MediOSS, kendi sunucunuzda çalışan bir esenlik uygulamasıdır: meditasyon kütüphanesi, ortam sesi karıştırıcı ve ruh hâli takibi tek bir hesapta bir arada. Meditasyon alışkanlığınız ve ruh hâliniz yalnızca size ait kalsın diye geliştirildi; üçüncü taraf hizmet kullanmaz, sizi izlemez. Web arayüzünü ve API'yi tek bir program sunar, veriler tek bir SQLite veritabanında durur; hesapları yönetici oluşturur. MediOSS'u Ahmet Çetinkaya ve katkıda bulunanlar, AGPL lisanslı açık kaynaklı bir yazılım olarak geliştiriyor.",
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
