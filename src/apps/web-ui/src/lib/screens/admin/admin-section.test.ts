import { describe, expect, test } from "bun:test";
import { adminSectionPath, parseAdminSection } from "./admin-section";

describe("parseAdminSection", () => {
  test("admin paths map to their sections", () => {
    expect(parseAdminSection("/admin/")).toBe("users");
    expect(parseAdminSection("/admin")).toBe("users");
    expect(parseAdminSection("/admin/meditations/")).toBe("meditations");
    expect(parseAdminSection("/admin/soundscape/")).toBe("soundscape");
    expect(parseAdminSection("/admin/soundscape")).toBe("soundscape");
  });

  test("deeper addresses belong to their section", () => {
    expect(parseAdminSection("/admin/meditations/12/")).toBe("meditations");
    expect(parseAdminSection("/admin/soundscape/x/")).toBe("soundscape");
  });

  test("unknown paths mean users", () => {
    expect(parseAdminSection("/admin/other/")).toBe("users");
    expect(parseAdminSection("/admin/Soundscape/")).toBe("users");
  });
});

describe("adminSectionPath", () => {
  test("each section has its own address", () => {
    expect(adminSectionPath("users")).toBe("/admin/");
    expect(adminSectionPath("meditations")).toBe("/admin/meditations/");
    expect(adminSectionPath("soundscape")).toBe("/admin/soundscape/");
  });
});
