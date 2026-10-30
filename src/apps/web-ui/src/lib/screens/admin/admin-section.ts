// Admin section addresses (FR-072): each section is its own path under
// /admin/ — /admin/ (meditations), /admin/soundscape/, /admin/users/.

export const ADMIN_SECTIONS = ["meditations", "soundscape", "users"] as const;

export type AdminSection = (typeof ADMIN_SECTIONS)[number];

/** The section carried by an admin path; unknown paths → meditations. A deeper
 *  address (e.g. /admin/meditations/12/) belongs to its first segment. */
export function parseAdminSection(pathname: string): AdminSection {
  const rest = pathname.replace(/\/+$/, "").replace(/^\/admin\/+?/, "");
  const head = rest.split("/")[0];
  return ADMIN_SECTIONS.find((section) => section === head) ?? "meditations";
}

/** The address (path) of an admin section. */
export function adminSectionPath(section: AdminSection): string {
  return section === "meditations" ? "/admin/" : `/admin/${section}/`;
}
