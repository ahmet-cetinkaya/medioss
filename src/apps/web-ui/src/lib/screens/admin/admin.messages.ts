// Copy for the Admin screen (FR-071–FR-078, PB-13). Server refusals use
// server-reason.messages.ts; shared actions use common.messages.ts.

import { defineMessages } from "../../i18n/message";

export const adminMessages = defineMessages({
  sectionLegend: { en: "Admin section", tr: "Yönetim bölümü" },
  sectionUsers: { en: "Users", tr: "Kullanıcılar" },
  sectionMeditations: { en: "Meditations", tr: "Meditasyonlar" },
  sectionSoundscape: { en: "Soundscape", tr: "Ses Manzarası" },
  loadFailed: {
    en: "Could not load this section. Try again.",
    tr: "Bu bölüm yüklenemedi. Yeniden deneyin.",
  },
  chooseOption: { en: "Choose…", tr: "Seçin…" },
});

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

export const meditationAdminMessages = defineMessages({
  uploadHeading: { en: "Upload meditation audio", tr: "Meditasyon sesi yükle" },
  uploadHint: {
    en: "MP3, WAV, Ogg, or Opus up to 100 MB; stored as Opus.",
    tr: "En fazla 100 MB MP3, WAV, Ogg veya Opus; Opus olarak saklanır.",
  },
  program: { en: "Program", tr: "Program" },
  programHint: {
    en: "The file is stored in this program's folder.",
    tr: "Dosya bu programın klasöründe saklanır.",
  },
  noProgramYet: {
    en: "Create a program first; every upload belongs to one.",
    tr: "Önce bir program oluşturun; her yükleme bir programa aittir.",
  },
  audioFile: { en: "Audio file", tr: "Ses dosyası" },
  name: { en: "Name", tr: "Ad" },
  upload: { en: "Upload", tr: "Yükle" },
  converting: { en: "Converting…", tr: "Dönüştürülüyor…" },
  uploaded: {
    en: "“{name}” was uploaded and stored as Opus.",
    tr: "“{name}” yüklendi ve Opus olarak saklandı.",
  },
  programHeading: { en: "Create program", tr: "Program oluştur" },
  title: { en: "Title", tr: "Başlık" },
  description: { en: "Description (optional)", tr: "Açıklama (isteğe bağlı)" },
  cover: { en: "Cover image (optional)", tr: "Kapak görseli (isteğe bağlı)" },
  coverHint: {
    en: "PNG, JPEG, or WebP up to 5 MB.",
    tr: "En fazla 5 MB PNG, JPEG veya WebP.",
  },
  createProgram: { en: "Create program", tr: "Program oluştur" },
  programCreated: {
    en: "Program “{title}” was created.",
    tr: "“{title}” programı oluşturuldu.",
  },
  categoryHeading: {
    en: "Meditation categories",
    tr: "Meditasyon kategorileri",
  },
  categoriesPanelHeading: { en: "Categories", tr: "Categories" },
  categoryName: { en: "New category name", tr: "Yeni kategori adı" },
  addCategory: { en: "Add category", tr: "Kategori ekle" },
  categoryCreated: {
    en: "Category “{name}” was added.",
    tr: "“{name}” kategorisi eklendi.",
  },
  renameCategory: { en: "Rename", tr: "Yeniden adlandır" },
  renameCategoryTitle: {
    en: "Rename category",
    tr: "Kategoriyi yeniden adlandır",
  },
  categoryRenamed: {
    en: "Category “{name}” was saved.",
    tr: "“{name}” kategorisi kaydedildi.",
  },
  deleteCategoryItem: { en: "Delete", tr: "Sil" },
  deleteCategoryDialogTitle: {
    en: "Delete this category?",
    tr: "Bu kategori silinsin mi?",
  },
  deleteCategoryDialogText: {
    en: "Its programs become uncategorized. This cannot be undone.",
    tr: "Programları kategorisiz hâle gelir. Bu işlem geri alınamaz.",
  },
  deleteCategoryDialogConfirm: {
    en: "Delete category",
    tr: "Kategoriyi sil",
  },
  meditationCategoryDeleted: {
    en: "Category “{name}” was deleted.",
    tr: "“{name}” kategorisi silindi.",
  },
  noCategories: { en: "No categories yet", tr: "Henüz kategori yok" },
  categoryActions: {
    en: "More actions for {name}",
    tr: "{name} için diğer eylemler",
  },
  noProgramDescription: {
    en: "No description provided.",
    tr: "Açıklama eklenmemiş.",
  },
  noMeditationDescription: {
    en: "No description.",
    tr: "Açıklama yok.",
  },
  programCategory: {
    en: "Category: {category}",
    tr: "Kategori: {category}",
  },
  programsHeading: { en: "Programs", tr: "Programlar" },
  programsText: {
    en: "Choose each program's category.",
    tr: "Her programın kategorisini seçin.",
  },
  categoryFor: { en: "Category for {title}", tr: "{title} kategorisi" },
  uncategorized: { en: "Uncategorized", tr: "Kategorisiz" },
  categoryUpdated: {
    en: "Category of “{title}” was updated.",
    tr: "“{title}” kategorisi güncellendi.",
  },
  noPrograms: { en: "No programs yet", tr: "Henüz program yok" },
  libraryHeading: { en: "Library", tr: "Kütüphane" },
  sizeOpus: { en: "{size} · Opus", tr: "{size} · Opus" },
  noMeditations: { en: "No meditations yet", tr: "Henüz meditasyon yok" },
  noMeditationsText: {
    en: "Uploaded meditations appear here.",
    tr: "Yüklenen meditasyonlar burada görünür.",
  },
  addToProgram: {
    en: "Add {name} to a program",
    tr: "{name} meditasyonunu bir programa ekle",
  },
  addToPlaceholder: { en: "Add to program…", tr: "Programa ekle…" },
  addedToProgram: {
    en: "“{name}” was added to “{title}”.",
    tr: "“{name}”, “{title}” programına eklendi.",
  },
  playNamed: { en: "Play {name}", tr: "{name} oynat" },
  downloadNamed: { en: "Download {name}", tr: "{name} indir" },
  descriptionCount: { en: "{count} / {max}", tr: "{count} / {max}" },
  descriptionTooLong: {
    en: "The description must be at most {max} characters.",
    tr: "Açıklama en fazla {max} karakter olmalıdır.",
  },
  editDescriptionNamed: {
    en: "Edit description of {name}",
    tr: "{name} açıklamasını düzenle",
  },
  editDescriptionTitle: {
    en: "Edit description",
    tr: "Açıklamayı düzenle",
  },
  editDescriptionHint: {
    en: "Leave empty to remove the description.",
    tr: "Açıklamayı kaldırmak için boş bırakın.",
  },
  descriptionUpdated: {
    en: "Description of “{name}” was saved.",
    tr: "“{name}” açıklaması kaydedildi.",
  },
  allPrograms: { en: "All programs", tr: "Tüm programlar" },
  categoryFilterNamed: {
    en: "{name} ({count})",
    tr: "{name} ({count})",
  },
  addCategoryTitle: { en: "Add category", tr: "Kategori ekle" },
  addProgram: { en: "Add program", tr: "Program ekle" },
  addProgramTitle: { en: "Add program", tr: "Program ekle" },
  programsFilteredEmpty: {
    en: "No programs in this category",
    tr: "Bu kategoride program yok",
  },
  programActions: {
    en: "More actions for {title}",
    tr: "{title} için diğer eylemler",
  },
  selectProgram: { en: "Open {title}", tr: "{title} programını aç" },
  setCategory: { en: "Set category", tr: "Kategori seç" },
  setCategoryTitle: { en: "Set category", tr: "Kategori seç" },
  addMeditationItem: {
    en: "Add existing meditation",
    tr: "Mevcut meditasyonu ekle",
  },
  addMeditationTitle: {
    en: "Add meditation to program",
    tr: "Programa meditasyon ekle",
  },
  chooseMeditation: { en: "Meditation", tr: "Meditasyon" },
  noMeditationsToAdd: {
    en: "Every meditation is already in this program.",
    tr: "Tüm meditasyonlar zaten bu programda.",
  },
  addMeditationSubmit: { en: "Add", tr: "Ekle" },
  editProgram: { en: "Edit program", tr: "Programı düzenle" },
  editProgramTitle: { en: "Edit program", tr: "Programı düzenle" },
  editProgramHint: {
    en: "Leave the cover empty to keep the current one.",
    tr: "Mevcut kapağı korumak için kapak alanını boş bırakın.",
  },
  programSaved: {
    en: "Program “{title}” was saved.",
    tr: "“{title}” programı kaydedildi.",
  },
  uploadItem: { en: "Upload audio", tr: "Ses yükle" },
  uploadTitle: { en: "Upload meditation audio", tr: "Meditasyon sesi yükle" },
  backToPrograms: { en: "Back to programs", tr: "Programlara dön" },
  pickProgram: {
    en: "Select a program to manage its meditations.",
    tr: "Meditasyonlarını yönetmek için bir program seçin.",
  },
  meditationsOf: { en: "Meditations in {title}", tr: "{title} meditasyonları" },
  meditationActions: {
    en: "More actions for {name}",
    tr: "{name} için diğer eylemler",
  },
  moveUp: { en: "Move {name} up", tr: "{name} yukarı taşı" },
  moveDown: { en: "Move {name} down", tr: "{name} aşağı taşı" },
  moved: {
    en: "“{name}” moved to position {position}.",
    tr: "“{name}” {position}. sıraya taşındı.",
  },
  positionOf: { en: "{position} of {total}", tr: "{position} / {total}" },
  unassignedHeading: {
    en: "Meditations without a program",
    tr: "Programsız meditasyonlar",
  },
  unassignedEmpty: {
    en: "Every meditation belongs to a program.",
    tr: "Tüm meditasyonlar bir programa ait.",
  },
  assignToProgram: { en: "Add to a program", tr: "Programa ekle" },
  assignTitle: {
    en: "Add meditation to a program",
    tr: "Meditasyonu bir programa ekle",
  },
  removeFromProgram: { en: "Remove from program", tr: "Programdan çıkar" },
  removeTitle: {
    en: "Remove from this program?",
    tr: "Bu programdan çıkarılsın mı?",
  },
  removeConsequence: {
    en: "The meditation stays in the library; it is only removed from this program.",
    tr: "Meditasyon kütüphanede kalır; yalnızca bu programdan çıkarılır.",
  },
  removeConfirm: { en: "Remove", tr: "Çıkar" },
  removed: {
    en: "“{name}” was removed from the program.",
    tr: "“{name}” programdan çıkarıldı.",
  },
  membersLoadFailed: {
    en: "The meditations could not be loaded.",
    tr: "Meditasyonlar yüklenemedi.",
  },
  noMembers: {
    en: "No meditations in this program",
    tr: "Bu programda meditasyon yok",
  },
  noMembersText: {
    en: "Upload audio or add an existing meditation.",
    tr: "Ses yükleyin veya mevcut bir meditasyonu ekleyin.",
  },
});

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
