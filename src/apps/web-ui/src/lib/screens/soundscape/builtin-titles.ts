import { t } from "$lib/i18n/translate";
import type { Message } from "$lib/i18n/message";
import {
  builtinCategoryTitles,
  builtinSoundTitles,
} from "./builtin-titles.messages";

function titleOf(titles: Record<string, Message>, name: string): string {
  return Object.hasOwn(titles, name) ? t(titles[name]) : name;
}

/** Built-in sound name in the active language; any other name shows as stored. */
export function soundTitle(name: string): string {
  return titleOf(builtinSoundTitles, name);
}

/** Built-in category name in the active language; any other name shows as stored. */
export function categoryTitle(name: string): string {
  return titleOf(builtinCategoryTitles, name);
}
