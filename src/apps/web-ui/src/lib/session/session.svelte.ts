// The page's session (plan D2). One `/me` probe per page load, shared by every
// caller; it never redirects (AppShell acts on `decideGate`). No browser
// access at module top level.

import { fetchMeProfile } from "../api/identity";
import {
  isAdmin,
  mustChangePassword,
  type SessionState,
  type SessionUser,
  sessionStateFromProbe,
} from "./gate-decision";

class Session {
  #state = $state.raw<SessionState>({ status: "loading" });
  // Starts per page load: the avatar is cached for a day, so a constant
  // start would show a stale image after a change on an earlier page.
  #avatarVersion = $state(Date.now());
  #pending: Promise<SessionState> | null = null;
  #settled = false;
  #generation = 0;

  get state(): SessionState {
    return this.#state;
  }

  get user(): SessionUser | null {
    return this.#state.status === "authenticated" ? this.#state.user : null;
  }

  get isAdmin(): boolean {
    const user = this.user;
    return user !== null && isAdmin(user);
  }

  get mustChangePassword(): boolean {
    const user = this.user;
    return user !== null && mustChangePassword(user);
  }

  /** Cache-buster for `avatarUrl(id, version)`; bumped by `setUser`. */
  get avatarVersion(): number {
    return this.#avatarVersion;
  }

  /** The first call probes `/me`; later calls share it, or its settled state. */
  load = (): Promise<SessionState> => {
    if (this.#pending) return this.#pending;
    if (this.#settled) return Promise.resolve(this.#state);
    return this.#probe();
  };

  /** Forces a new probe: retry, after a password change, on bfcache restore. */
  refresh = (): Promise<SessionState> => this.#probe();

  /**
   * Settings: the fresh profile after a name or avatar change. Always bumps
   * the avatar version, but only replaces the signed-in user it matches; an
   * in-flight probe, or a different or absent user, keeps the `/me` result.
   */
  setUser = (user: SessionUser): void => {
    this.#avatarVersion++;
    const current = this.#state;
    if (
      this.#pending ||
      current.status !== "authenticated" ||
      current.user.id !== user.id
    )
      return;
    this.#state = { status: "authenticated", user };
  };

  /** `pageshow` handler: a page restored from bfcache re-probes. */
  revalidateOnPageShow = (event: PageTransitionEvent): void => {
    if (event.persisted) void this.refresh();
  };

  #probe(): Promise<SessionState> {
    const generation = ++this.#generation;
    this.#settled = false;
    this.#state = { status: "loading" };
    const pending = fetchMeProfile().then(
      (user) => this.#apply(generation, sessionStateFromProbe({ user })),
      (error: unknown) =>
        this.#apply(generation, sessionStateFromProbe({ error })),
    );
    this.#pending = pending;
    return pending;
  }

  // A superseded probe yields to the newer one instead of overwriting it.
  #apply(generation: number, next: SessionState): Promise<SessionState> {
    if (generation !== this.#generation) return this.load();
    this.#pending = null;
    this.#settled = true;
    this.#state = next;
    return Promise.resolve(next);
  }
}

export const session = new Session();
