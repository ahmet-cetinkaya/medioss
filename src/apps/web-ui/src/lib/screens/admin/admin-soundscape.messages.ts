// Copy for the Admin soundscape section.

import { defineMessages } from "../../i18n/message";

export const soundAdminMessages = defineMessages({
  uncategorized: { en: "Uncategorized", tr: "Kategorisiz" },
  categoriesHeading: { en: "Sound categories", tr: "Ses kategorileri" },
  categoryName: { en: "New sound category", tr: "Yeni ses kategorisi" },
  addCategory: { en: "Add category", tr: "Kategori ekle" },
  categoryAdded: {
    en: "Sound category “{name}” was added.",
    tr: "“{name}” ses kategorisi eklendi.",
  },
  noCategories: {
    en: "No sound categories yet",
    tr: "Henüz ses kategorisi yok",
  },
  renameCategoryTitle: {
    en: "Rename sound category",
    tr: "Ses kategorisini yeniden adlandır",
  },
  renameNamed: { en: "Rename {name}", tr: "{name} yeniden adlandır" },
  deleteNamed: { en: "Delete {name}", tr: "{name} sil" },
  editNamed: { en: "Edit {name}", tr: "{name} düzenle" },
  downloadNamed: { en: "Download {name}", tr: "{name} indir" },
  deleteCategoryTitle: {
    en: "Delete this sound category?",
    tr: "Bu ses kategorisi silinsin mi?",
  },
  deleteCategoryConsequence: {
    en: "Only a category without sounds can be deleted.",
    tr: "Yalnızca içinde ses olmayan bir kategori silinebilir.",
  },
  deleteCategoryConfirm: { en: "Delete category", tr: "Kategoriyi sil" },
  categoryDeleted: {
    en: "Sound category “{name}” was deleted.",
    tr: "“{name}” ses kategorisi silindi.",
  },
  soundsHeading: { en: "Sounds", tr: "Sesler" },
  soundsIn: { en: "Sounds in {name}", tr: "{name} sesleri" },
  selectCategory: { en: "Open {name}", tr: "{name} kategorisini aç" },
  categoryCount: { en: "{name} ({count})", tr: "{name} ({count})" },
  categoryActions: {
    en: "More actions for {name}",
    tr: "{name} için diğer eylemler",
  },
  soundActions: {
    en: "More actions for {name}",
    tr: "{name} için diğer eylemler",
  },
  previewSound: { en: "Play preview", tr: "Önizlemeyi oynat" },
  stopPreview: { en: "Stop preview", tr: "Önizlemeyi durdur" },
  previewFailed: {
    en: "{name} could not be played.",
    tr: "{name} oynatılamadı.",
  },
  addCategoryTitle: { en: "Add sound category", tr: "Ses kategorisi ekle" },
  soundName: { en: "Sound name", tr: "Ses adı" },
  category: { en: "Category", tr: "Kategori" },
  audioFile: { en: "Audio file", tr: "Ses dosyası" },
  uploadSound: { en: "Upload sound", tr: "Ses yükle" },
  uploading: { en: "Uploading…", tr: "Yükleniyor…" },
  soundUploaded: {
    en: "Sound “{name}” was uploaded.",
    tr: "“{name}” sesi yüklendi.",
  },
  needCategory: {
    en: "Add a sound category first; every sound belongs to one.",
    tr: "Önce bir ses kategorisi ekleyin; her ses bir kategoriye aittir.",
  },
  builtin: { en: "Built-in", tr: "Yerleşik" },
  noSounds: { en: "No sounds yet", tr: "Henüz ses yok" },
  editTitle: { en: "Edit sound", tr: "Sesi düzenle" },
  soundUpdated: {
    en: "Sound “{name}” was updated.",
    tr: "“{name}” sesi güncellendi.",
  },
  deleteSoundTitle: { en: "Delete this sound?", tr: "Bu ses silinsin mi?" },
  deleteSoundConsequence: {
    en: "It will be removed from every saved mix. This cannot be undone.",
    tr: "Kayıtlı tüm karışımlardan kaldırılır. Bu işlem geri alınamaz.",
  },
  deleteSoundConfirm: { en: "Delete sound", tr: "Sesi sil" },
  soundDeleted: {
    en: "Sound “{name}” was deleted.",
    tr: "“{name}” sesi silindi.",
  },
});

export const soundIconMessages = defineMessages({
  legend: { en: "Icon", tr: "Simge" },
  none: { en: "No icon", tr: "Simge yok" },
  uploadImage: { en: "Or upload an image", tr: "Veya bir görsel yükle" },
  uploadTile: { en: "Upload", tr: "Yükle" },
  imageHint: {
    en: "PNG, JPEG or WebP, up to 1 MB.",
    tr: "PNG, JPEG veya WebP, en fazla 1 MB.",
  },
  clearImage: { en: "Remove image", tr: "Görseli kaldır" },
  currentImage: {
    en: "This sound has an uploaded image.",
    tr: "Bu sesin yüklenmiş bir görseli var.",
  },
  badType: {
    en: "Choose a PNG, JPEG or WebP image.",
    tr: "PNG, JPEG veya WebP görsel seç.",
  },
  tooLarge: {
    en: "The image must be 1 MB or smaller.",
    tr: "Görsel en fazla 1 MB olmalı.",
  },
  rain: { en: "Rain", tr: "Yağmur" },
  wind: { en: "Wind", tr: "Rüzgar" },
  fire: { en: "Fire", tr: "Ateş" },
  water: { en: "Water", tr: "Su" },
  night: { en: "Night", tr: "Gece" },
  forest: { en: "Forest", tr: "Orman" },
  birds: { en: "Birds", tr: "Kuşlar" },
  thunder: { en: "Thunder", tr: "Gök gürültüsü" },
  noise: { en: "Noise and cafe", tr: "Gürültü ve kafe" },
  heartbeat: { en: "Heartbeat", tr: "Kalp atışı" },
  city: { en: "City and trains", tr: "Şehir ve tren" },
  music: { en: "Music", tr: "Müzik" },
  drop: { en: "Drop", tr: "Damla" },
  leaf: { en: "Leaves", tr: "Yapraklar" },
  bell: { en: "Bell", tr: "Çan" },
  piano: { en: "Piano", tr: "Piyano" },
  guitar: { en: "Guitar", tr: "Gitar" },
  fan: { en: "Fan", tr: "Vantilatör" },
  airplane: { en: "Airplane", tr: "Uçak" },
  cricket: { en: "Crickets", tr: "Cırcır böceği" },
  cat: { en: "Cat purring", tr: "Kedi" },
  mountains: { en: "Mountains", tr: "Dağlar" },
  campfire: { en: "Campfire", tr: "Kamp ateşi" },
  snowflake: { en: "Snow", tr: "Kar" },
});
