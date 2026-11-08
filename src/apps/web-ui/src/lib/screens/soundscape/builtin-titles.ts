import { t } from "$lib/i18n/translate";
import type { SoundscapeSound } from "$lib/api/soundscape";
import type { Message } from "$lib/i18n/message";
import {
  builtinCategoryTitles,
  builtinSoundTitles,
} from "./builtin-titles.messages";

function titleOf(titles: Record<string, Message>, name: string): string {
  return Object.hasOwn(titles, name) ? t(titles[name]) : name;
}

/**
 * Built-in sound name in the active language. An uploaded sound shows as
 * stored, even when its name equals a built-in one.
 */
export function soundTitle(
  sound: Pick<SoundscapeSound, "name" | "builtin">,
): string {
  return sound.builtin ? titleOf(builtinSoundTitles, sound.name) : sound.name;
}

/** Built-in category name in the active language; any other name shows as stored. */
export function categoryTitle(name: string): string {
  return titleOf(builtinCategoryTitles, name);
}
