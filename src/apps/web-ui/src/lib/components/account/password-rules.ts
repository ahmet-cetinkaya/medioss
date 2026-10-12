// New-password rules checked before any request (spec FR-104). Pure: no
// Svelte or browser access. The server enforces the same minimum.

/** Minimum length in Unicode code points ("ğ" counts as one). */
export const PASSWORD_MIN_LENGTH = 8;

export type PasswordRuleViolation = "tooShort" | "mismatch";

/** The first broken rule, length before mismatch; null when both hold. */
export function validateNewPassword(
  password: string,
  confirm: string,
): PasswordRuleViolation | null {
  if ([...password].length < PASSWORD_MIN_LENGTH) return "tooShort";
  if (password !== confirm) return "mismatch";
  return null;
}
