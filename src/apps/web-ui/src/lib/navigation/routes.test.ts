import { describe, expect, test } from "bun:test";
import { ICON_NAMES } from "../components/icons/icon-name";
import { navMessages } from "../i18n/common.messages";
import {
  isMoreCurrent,
  MOBILE_PRIMARY,
  MORE_ICON,
  MORE_ITEMS,
  NAV_GROUPS,
  navCurrentFor,
  ROUTES,
  SCREEN_ALLOWLIST,
  isScreenAddress,
  type ScreenKey,
} from "./routes";

const ALL_SCREENS = Object.keys(ROUTES) as ScreenKey[];

describe("routes", () => {
  test("groups and order equal FR-002", () => {
    expect(
      NAV_GROUPS.map((group) => [group.key, group.items.map((i) => i.key)]),
    ).toEqual([
      ["practice", ["home", "library", "soundscape", "mood"]],
      ["progress", ["history", "stats"]],
      ["account", ["settings", "admin", "signOut"]],
      ["project", ["about"]],
    ]);
  });

  test("sign out is an action, not a screen", () => {
    const signOut = NAV_GROUPS[2].items[2];
    expect(signOut.kind).toBe("action");
    expect(signOut.label).toBe(navMessages.signOut);
    expect(signOut.icon).toBe("sign-out");
  });

  test("only Admin requires admin", () => {
    expect(ALL_SCREENS.filter((key) => ROUTES[key].requiresAdmin)).toEqual([
      "admin",
    ]);
    const adminItem = NAV_GROUPS[2].items.find((i) => i.key === "admin");
    expect(adminItem?.requiresAdmin).toBe(true);
  });

  test("Program and Player mark Library current; others mark themselves", () => {
    expect(navCurrentFor("program")).toBe("library");
    expect(navCurrentFor("player")).toBe("library");
    for (const key of ALL_SCREENS.filter(
      (k) => k !== "program" && k !== "player",
    )) {
      expect(navCurrentFor(key)).toBe(key);
    }
  });

  test("mobile primary and More equal FR-087", () => {
    expect(MOBILE_PRIMARY.map((i) => i.key)).toEqual([
      "home",
      "library",
      "soundscape",
      "mood",
    ]);
    expect(MORE_ITEMS.map((i) => i.key)).toEqual([
      "history",
      "stats",
      "settings",
      "admin",
      "signOut",
      "about",
    ]);
  });

  test("More is current exactly for history, stats, settings, about, admin", () => {
    expect(ALL_SCREENS.filter(isMoreCurrent)).toEqual([
      "history",
      "stats",
      "settings",
      "about",
      "admin",
    ]);
  });

  test("every icon name is valid", () => {
    const icons = [
      ...NAV_GROUPS.flatMap((g) => g.items.map((i) => i.icon)),
      MORE_ICON,
    ];
    expect(icons).toHaveLength(11);
    for (const icon of icons) expect(ICON_NAMES).toContain(icon);
  });

  test("allowlist is the eleven directory-form paths", () => {
    expect(SCREEN_ALLOWLIST).toEqual([
      "/",
      "/library/",
      "/program/",
      "/player/",
      "/soundscape/",
      "/mood/",
      "/history/",
      "/stats/",
      "/settings/",
      "/about/",
      "/admin/",
    ]);
  });

  test("screen addresses include the canonical deep paths", () => {
    for (const path of [
      "/player/5/",
      "/admin/meditations/",
      "/admin/soundscape/",
      "/admin/meditations/12/",
    ]) {
      expect(isScreenAddress(path)).toBe(true);
    }
    for (const path of [
      "/player/abc/",
      "/player/5/extra/",
      "/admin/meditations/12/extra/",
      "/unknown/",
      "/admin/meditations/abc/",
    ]) {
      expect(isScreenAddress(path)).toBe(false);
    }
  });
});
