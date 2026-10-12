// Whether the library has any meditation, shared by the Home regions for
// their empty states (FR-028). The daily suggestion is null exactly when the
// library is empty, so the suggestion region sets it; `null` means unknown
// (still loading, or the suggestion failed), which shows the Library link.

export const homeLibrary = $state<{ empty: boolean | null }>({ empty: null });
