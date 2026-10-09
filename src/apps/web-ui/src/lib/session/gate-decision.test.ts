import { describe, expect, test } from "bun:test";
import {
  decideGate,
  GATE_DECISIONS,
  type GateRequirement,
  isAdmin,
  mustChangePassword,
  type SessionState,
  type SessionUser,
  sessionStateFromProbe,
} from "./gate-decision";

const user = (role: string, flagged = false): SessionUser => ({
  id: 1,
  email: "a@example.com",
  role,
  must_change_password: flagged,
  display_name: null,
  has_avatar: false,
});

const authed = (u: SessionUser): SessionState => ({
  status: "authenticated",
  user: u,
});

const REQUIREMENTS: GateRequirement[] = ["user", "admin"];

describe("decideGate", () => {
  test("loading → skeleton", () => {
    for (const r of REQUIREMENTS)
      expect(decideGate({ status: "loading" }, r)).toBe("skeleton");
  });

  test("anonymous → redirectLogin", () => {
    for (const r of REQUIREMENTS)
      expect(decideGate({ status: "anonymous" }, r)).toBe("redirectLogin");
  });

  test("network or 5xx error → error, never a redirect (FR-011)", () => {
    for (const status of [0, 500, 503])
      for (const r of REQUIREMENTS)
        expect(
          decideGate(sessionStateFromProbe({ error: { status } }), r),
        ).toBe("error");
  });

  test("flagged member or admin → changePassword on every screen (FR-105)", () => {
    for (const role of ["member", "admin"])
      for (const r of REQUIREMENTS)
        expect(decideGate(authed(user(role, true)), r)).toBe("changePassword");
  });

  test("member on an admin screen → adminsOnly (FR-071)", () => {
    expect(decideGate(authed(user("member")), "admin")).toBe("adminsOnly");
  });

  test("admin on an admin screen → render", () => {
    expect(decideGate(authed(user("admin")), "admin")).toBe("render");
  });

  test("authenticated, unflagged user → render", () => {
    expect(decideGate(authed(user("member")), "user")).toBe("render");
    expect(decideGate(authed(user("admin")), "user")).toBe("render");
  });

  test("exhaustive matrix stays within the declared set; no change-password redirect", () => {
    const states: SessionState[] = [
      { status: "loading" },
      { status: "anonymous" },
      { status: "error" },
      { status: "error", error: { status: 0 } },
    ];
    for (const role of ["member", "admin", "", "Admin"])
      for (const flagged of [false, true])
        states.push(authed(user(role, flagged)));

    const allowed: readonly string[] = GATE_DECISIONS;
    expect(allowed).toEqual([
      "skeleton",
      "error",
      "redirectLogin",
      "changePassword",
      "adminsOnly",
      "render",
    ]);
    for (const s of states)
      for (const r of REQUIREMENTS) {
        const decision = decideGate(s, r);
        expect(allowed).toContain(decision);
        expect(decision).not.toMatch(/redirect.*password|password.*redirect/i);
      }
  });

  test("role match is exact", () => {
    expect(decideGate(authed(user("Admin")), "admin")).toBe("adminsOnly");
  });
});

describe("sessionStateFromProbe", () => {
  test("200 → authenticated with the user", () => {
    const u = user("member");
    expect(sessionStateFromProbe({ user: u })).toEqual(authed(u));
  });

  test("401 → anonymous", () => {
    expect(sessionStateFromProbe({ error: { status: 401 } })).toEqual({
      status: "anonymous",
    });
  });

  test("0, 500, 403 → error carrying the cause", () => {
    for (const status of [0, 500, 403]) {
      const error = { status };
      expect(sessionStateFromProbe({ error })).toEqual({
        status: "error",
        error,
      });
    }
  });

  test("non-API throws → error", () => {
    for (const error of [new TypeError("x"), null, undefined, "401"])
      expect(sessionStateFromProbe({ error }).status).toBe("error");
  });
});

describe("helpers", () => {
  test("isAdmin and mustChangePassword", () => {
    expect(isAdmin(user("admin"))).toBe(true);
    expect(isAdmin(user("member"))).toBe(false);
    expect(mustChangePassword(user("member", true))).toBe(true);
    expect(mustChangePassword(user("member"))).toBe(false);
  });
});
