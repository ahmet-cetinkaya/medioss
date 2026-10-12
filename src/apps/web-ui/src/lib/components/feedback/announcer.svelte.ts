// One polite live region per page (FR-021). Call `announce(text)` with
// already-translated text; focus never moves. `LiveAnnouncer` renders
// `announcement.message`, and the shell mounts it once per page.

export const announcement = $state({ message: "" });

let pending: ReturnType<typeof setTimeout> | undefined;

/**
 * Announces `message` politely. The region is cleared first and refilled on
 * the next tick, so repeating the same text is announced again.
 */
export function announce(message: string): void {
  clearTimeout(pending);
  announcement.message = "";
  pending = setTimeout(() => {
    announcement.message = message;
  }, 50);
}
