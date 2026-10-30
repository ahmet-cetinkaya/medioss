// Copy for the Admin meditations section.

import { defineMessages } from "../../i18n/message";

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
  renameMeditationItem: { en: "Rename", tr: "Yeniden adlandır" },
  renameMeditationTitle: {
    en: "Rename meditation",
    tr: "Meditasyonu yeniden adlandır",
  },
  meditationName: { en: "Name", tr: "Ad" },
  meditationRenamed: {
    en: "Meditation renamed to “{name}”.",
    tr: "Meditasyon “{name}” olarak yeniden adlandırıldı.",
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
  deleteMeditationItem: { en: "Delete meditation", tr: "Meditasyonu sil" },
  deleteMeditationTitle: {
    en: "Delete this meditation?",
    tr: "Bu meditasyon silinsin mi?",
  },
  deleteMeditationConsequence: {
    en: "It leaves every program, and its audio, notes and favorites are removed. This cannot be undone.",
    tr: "Tüm programlardan çıkar; sesi, notları ve favorileri silinir. Bu işlem geri alınamaz.",
  },
  meditationDeleted: {
    en: "Meditation “{name}” was deleted.",
    tr: "“{name}” meditasyonu silindi.",
  },
  removeCover: { en: "Remove the cover", tr: "Kapağı kaldır" },
  coverRemoved: {
    en: "Cover of “{title}” was removed.",
    tr: "“{title}” kapağı kaldırıldı.",
  },
  deleteProgramItem: { en: "Delete program", tr: "Programı sil" },
  deleteProgramDialogTitle: {
    en: "Delete this program?",
    tr: "Bu program silinsin mi?",
  },
  deleteProgramDialogText: {
    en: "Its meditations stay in the library. The program and its cover are removed. This cannot be undone.",
    tr: "Meditasyonları kütüphanede kalır. Program ve kapağı silinir. Bu işlem geri alınamaz.",
  },
  programDeleted: {
    en: "Program “{title}” was deleted.",
    tr: "“{title}” programı silindi.",
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
  addToAnotherProgram: {
    en: "Add to another program",
    tr: "Başka bir programa ekle",
  },
  searchMeditationsLabel: {
    en: "Search by meditation or program name",
    tr: "Meditasyon veya program adına göre ara",
  },
  searchProgramsLabel: {
    en: "Search programs",
    tr: "Programlarda ara",
  },
  pickerNoMatches: {
    en: "No meditations match your search.",
    tr: "Aramanızla eşleşen meditasyon yok.",
  },
  assignNoMatches: {
    en: "No programs match your search.",
    tr: "Aramanızla eşleşen program yok.",
  },
  assignInEveryProgram: {
    en: "This meditation is already in every program.",
    tr: "Bu meditasyon zaten tüm programlarda.",
  },
  pickerNoProgram: { en: "No program", tr: "Programsız" },
  alreadyInProgram: {
    en: "Already in this program",
    tr: "Zaten bu programda",
  },
  addFromLibraryHint: {
    en: "Click a meditation to add it to the program.",
    tr: "Eklemek için bir meditasyona tıklayın.",
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
