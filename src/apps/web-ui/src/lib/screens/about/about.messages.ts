// Copy for the About screen.

import { defineMessages } from "../../i18n/message";

export const aboutMessages = defineMessages({
  heading: { en: "MediOSS", tr: "MediOSS" },
  version: { en: "Version", tr: "Sürüm" },
  license: { en: "License", tr: "Lisans" },
  source: { en: "Source code", tr: "Kaynak kodu" },
  licenseFullText: {
    en: "Read the full license",
    tr: "Lisansın tam metnini oku",
  },
  thirdPartyOpen: {
    en: "Third-party licenses",
    tr: "Üçüncü taraf lisansları",
  },
  thirdPartyIntro: {
    en: "These works ship with MediOSS but are not covered by its license.",
    tr: "Bu çalışmalar MediOSS ile birlikte gelir ancak onun lisansı kapsamında değildir.",
  },
  soundsHeading: { en: "Built-in sounds", tr: "Yerleşik sesler" },
  soundsLicense: {
    en: "From the Moodist project, which states they are under the Pixabay Content License or CC0 1.0, without a per-file list.",
    tr: "Moodist projesinden gelir; proje bunların dosya bazında ayrım yapmadan Pixabay İçerik Lisansı veya CC0 1.0 altında olduğunu belirtir.",
  },
  soundSource: { en: "Source", tr: "Kaynak" },
  linkMoodist: { en: "Moodist", tr: "Moodist" },
  linkPixabay: {
    en: "Pixabay Content License",
    tr: "Pixabay İçerik Lisansı",
  },
  linkCc0: { en: "CC0 1.0", tr: "CC0 1.0" },
  linkOfl: { en: "OFL 1.1", tr: "OFL 1.1" },
  fontsHeading: { en: "Fonts", tr: "Yazı tipleri" },
  fontLicense: {
    en: "Poppins, licensed under the SIL Open Font License 1.1.",
    tr: "Poppins, SIL Open Font License 1.1 ile lisanslıdır.",
  },
  loadFailed: {
    en: "Version information could not be loaded.",
    tr: "Sürüm bilgisi yüklenemedi.",
  },
});
