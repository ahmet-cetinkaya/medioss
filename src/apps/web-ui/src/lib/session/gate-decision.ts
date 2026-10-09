// Session states and the AppShell gate (plan D2; spec FR-009, FR-011,
// FR-071, FR-105). Pure: no Svelte, no browser access. The gate never yields
// a redirect to change-password; that view renders in place, so no loop.

/** The `/me` profile; structurally compatible with `MeProfile`. */
export interface SessionUser {
  id: number;
  email: string;
  role: string;
  must_change_password: boolean;
  display_name: string | null;
  has_avatar: boolean;
}

export type SessionState =
  | { status: "loading" }
  | { status: "authenticated"; user: SessionUser }
  | { status: "anonymous" }
  | { status: "error"; error?: unknown };

export type GateRequirement = "user" | "admin";

export const GATE_DECISIONS = [
  "skeleton",
  "error",
  "redirectLogin",
  "changePassword",
  "adminsOnly",
  "render",
] as const;

export type GateDecision = (typeof GATE_DECISIONS)[number];

export const isAdmin = (user: SessionUser): boolean => user.role === "admin";

export const mustChangePassword = (user: SessionUser): boolean =>
  user.must_change_password === true;

/**
 * What AppShell shows. A failed probe is an error, never Login (FR-011). The
 * password flag wins on every screen, Admin included (FR-105), before the
 * admin check (FR-071).
 */
export function decideGate(
  state: SessionState,
  requires: GateRequirement,
): GateDecision {
  switch (state.status) {
    case "loading":
      return "skeleton";
    case "error":
      return "error";
    case "anonymous":
      return "redirectLogin";
    case "authenticated":
      if (mustChangePassword(state.user)) return "changePassword";
      if (requires === "admin" && !isAdmin(state.user)) return "adminsOnly";
      return "render";
    default:
      return "error";
  }
}

/** A settled `/me` probe: the profile, or what it threw. */
export type ProbeResult = { user: SessionUser } | { error: unknown };

/**
 * Only a 401 means signed out; a network failure (status 0), a 5xx, any
 * other status, or a non-API throw is an error (FR-011).
 */
export function sessionStateFromProbe(result: ProbeResult): SessionState {
  if ("user" in result) return { status: "authenticated", user: result.user };
  const { error } = result;
  const status =
    error && typeof error === "object" && "status" in error
      ? (error as { status: unknown }).status
      : undefined;
  return status === 401 ? { status: "anonymous" } : { status: "error", error };
}
