// Account button initial (plan D10, FR-006).

/**
 * First code point of the trimmed display name, uppercased with Turkish
 * casing regardless of UI language ("i" → "İ", "ı" → "I"). Null when there is
 * no name, so the caller shows the generic User icon.
 */
export function avatarInitial(name: string | null | undefined): string | null {
  const first = name?.trim().codePointAt(0);
  return first === undefined
    ? null
    : String.fromCodePoint(first).toLocaleUpperCase("tr");
}
