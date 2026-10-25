// Single source for screens and navigation (plan T6; spec FR-002, FR-004, FR-087).
// Pure TS: no Svelte or browser access, so `bun test` can load it.

import type { IconName } from "../components/icons/icon-name";
import {
  navGroupMessages,
  navMessages,
  screenTitleMessages,
} from "../i18n/common.messages";
import type { StringMessage } from "../i18n/message";

export type ScreenKey =
  | "home"
  | "library"
  | "program"
  | "player"
  | "soundscape"
  | "mood"
  | "history"
  | "stats"
  | "settings"
  | "about"
  | "admin";

export type NavGroupKey = "practice" | "progress" | "account";

export type Screen = {
  readonly key: ScreenKey;
  /** Directory form, e.g. `/library/`. */
  readonly path: string;
  readonly title: StringMessage;
  /** `null` for screens that are not nav items (Program, Player). */
  readonly group: NavGroupKey | null;
  readonly icon: IconName | null;
  readonly requiresAdmin: boolean;
  /** The nav item marked current while this screen is shown. */
  readonly navCurrentFor: ScreenKey;
  /** Shown in the mobile bottom bar; otherwise reached through More. */
  readonly mobilePrimary: boolean;
};

function screen(
  key: ScreenKey,
  path: string,
  title: StringMessage,
  group: NavGroupKey | null,
  icon: IconName | null,
  options: Partial<
    Pick<Screen, "requiresAdmin" | "navCurrentFor" | "mobilePrimary">
  > = {},
): Screen {
  return {
    key,
    path,
    title,
    group,
    icon,
    requiresAdmin: options.requiresAdmin ?? false,
    navCurrentFor: options.navCurrentFor ?? key,
    mobilePrimary: options.mobilePrimary ?? false,
  };
}

const primary = { mobilePrimary: true };

export const ROUTES: Readonly<Record<ScreenKey, Screen>> = {
  home: screen(
    "home",
    "/",
    screenTitleMessages.home,
    "practice",
    "house",
    primary,
  ),
  library: screen(
    "library",
    "/library/",
    screenTitleMessages.library,
    "practice",
    "books",
    primary,
  ),
  program: screen(
    "program",
    "/program/",
    screenTitleMessages.program,
    null,
    null,
    { navCurrentFor: "library" },
  ),
  player: screen("player", "/player/", screenTitleMessages.player, null, null, {
    navCurrentFor: "library",
  }),
  soundscape: screen(
    "soundscape",
    "/soundscape/",
    screenTitleMessages.soundscape,
    "practice",
    "waveform",
    primary,
  ),
  mood: screen(
    "mood",
    "/mood/",
    screenTitleMessages.mood,
    "practice",
    "smiley",
    primary,
  ),
  history: screen(
    "history",
    "/history/",
    screenTitleMessages.history,
    "progress",
    "clock-counter-clockwise",
  ),
  stats: screen(
    "stats",
    "/stats/",
    screenTitleMessages.statistics,
    "progress",
    "chart-bar",
  ),
  settings: screen(
    "settings",
    "/settings/",
    screenTitleMessages.settings,
    "account",
    "gear-six",
  ),
  about: screen(
    "about",
    "/about/",
    screenTitleMessages.about,
    "account",
    "info",
  ),
  admin: screen(
    "admin",
    "/admin/",
    screenTitleMessages.admin,
    "account",
    "shield-check",
    { requiresAdmin: true },
  ),
};

/** The screen paths, directory form. */
export const SCREEN_ALLOWLIST: readonly string[] = Object.values(ROUTES).map(
  (route) => route.path,
);

const DEEP_SCREEN_PATH = /^\/(player\/\d+|admin\/[\w-]+(?:\/\d+)?)\/?$/;

/**
 * Whether a directory-form pathname is a screen address the app can return a
 * user to: a screen root, or one of the canonical deep paths the app itself
 * produces (`playerHref`, admin sections and their program details).
 */
export function isScreenAddress(pathname: string): boolean {
  return SCREEN_ALLOWLIST.includes(pathname) || DEEP_SCREEN_PATH.test(pathname);
}

/** A meditation's Player address, e.g. `/player/5/`. */
export function playerHref(id: number): string {
  return `/player/${id}/`;
}

/** `/player` → `/player/`; `/` and `/player/` are unchanged. */
export function directoryPath(pathname: string): string {
  return pathname.endsWith("/") ? pathname : `${pathname}/`;
}

export function navCurrentFor(screenKey: ScreenKey): ScreenKey {
  return ROUTES[screenKey].navCurrentFor;
}

export type NavItem =
  | {
      readonly kind: "screen";
      readonly key: ScreenKey;
      readonly href: string;
      readonly label: StringMessage;
      readonly icon: IconName;
      readonly requiresAdmin: boolean;
    }
  | {
      readonly kind: "action";
      readonly key: "signOut";
      readonly label: StringMessage;
      readonly icon: IconName;
      readonly requiresAdmin: false;
    };

const navLabels: Record<ScreenKey, StringMessage> = {
  home: navMessages.home,
  library: navMessages.library,
  program: screenTitleMessages.program,
  player: screenTitleMessages.player,
  soundscape: navMessages.soundscape,
  mood: navMessages.mood,
  history: navMessages.history,
  stats: navMessages.statistics,
  settings: navMessages.settings,
  about: navMessages.about,
  admin: navMessages.admin,
};

function screenItem(key: ScreenKey): NavItem {
  const route = ROUTES[key];
  return {
    kind: "screen",
    key,
    href: route.path,
    label: navLabels[key],
    icon: route.icon!,
    requiresAdmin: route.requiresAdmin,
  };
}

export const SIGN_OUT_ITEM: NavItem = {
  kind: "action",
  key: "signOut",
  label: navMessages.signOut,
  icon: "sign-out",
  requiresAdmin: false,
};

export type NavGroup = {
  readonly key: NavGroupKey;
  readonly label: StringMessage;
  readonly items: readonly NavItem[];
};

/** FR-002: Practice, Progress, Account, in this order. */
export const NAV_GROUPS: readonly NavGroup[] = [
  {
    key: "practice",
    label: navGroupMessages.practice,
    items: (["home", "library", "soundscape", "mood"] as const).map(screenItem),
  },
  {
    key: "progress",
    label: navGroupMessages.progress,
    items: (["history", "stats"] as const).map(screenItem),
  },
  {
    key: "account",
    label: navGroupMessages.account,
    items: [
      screenItem("settings"),
      screenItem("about"),
      screenItem("admin"),
      SIGN_OUT_ITEM,
    ],
  },
];

const navItems = NAV_GROUPS.flatMap((group) => group.items);

/** FR-087: the bottom bar's screens, before More. */
export const MOBILE_PRIMARY: readonly NavItem[] = navItems.filter(
  (item) => item.kind === "screen" && ROUTES[item.key].mobilePrimary,
);

/** FR-087: everything else, in nav order (Admin is filtered by the shell). */
export const MORE_ITEMS: readonly NavItem[] = navItems.filter(
  (item) => !MOBILE_PRIMARY.includes(item),
);

export const MORE_LABEL: StringMessage = navMessages.more;
export const MORE_ICON: IconName = "dots-three";

/** FR-087: More is current when the current nav item lives inside it. */
export function isMoreCurrent(screenKey: ScreenKey): boolean {
  const current = navCurrentFor(screenKey);
  return MORE_ITEMS.some(
    (item) => item.kind === "screen" && item.key === current,
  );
}
