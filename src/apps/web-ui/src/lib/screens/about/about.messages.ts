// Copy for the About screen.

import { defineMessages } from "../../i18n/message";

export const aboutMessages = defineMessages({
  description: {
    en: "MediOSS is a self-hosted wellness app for calmer days: a meditation library, an ambient soundscape mixer and a mood tracker, all behind one account on your own server. It exists so your practice and your moods stay yours, with no third-party services and no tracking. One program serves the web interface and the API from a single SQLite database, and the administrator creates the accounts. It is built by Ahmet Çetinkaya and the MediOSS contributors as open source software under the AGPL.",
    tr: "MediOSS, daha sakin günler için kendi sunucunuzda çalışan bir iyi oluş uygulamasıdır: meditasyon kütüphanesi, ortam sesi karıştırıcı ve ruh hâli takibi; hepsi tek bir hesabın arkasında. Pratiğiniz ve ruh hâliniz size ait kalsın diye var: üçüncü taraf hizmet yok, izleme yok. Tek bir program, web arayüzünü ve API'yi tek bir SQLite veritabanından sunar; hesapları yönetici oluşturur. Ahmet Çetinkaya ve MediOSS katkıcıları tarafından, AGPL lisanslı açık kaynaklı yazılım olarak geliştirilir.",
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
