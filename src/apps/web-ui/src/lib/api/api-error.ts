// The API error type and body parsing (plan D8). Error bodies carry no
// machine-readable code: identity and mood send `{"error": msg}`, meditation
// sends JSON or nothing, soundscape sends nothing, portability sends text.

/** Every API client call names its action, so refusals map to a reason. */
export type ApiAction =
  | "generic"
  // meta (About)
  | "loadMeta"
  // identity
  | "login"
  | "logout"
  | "setup"
  | "setup.status"
  | "me.load"
  | "users.list"
  | "users.create"
  | "users.status"
  | "users.delete"
  | "account.delete"
  | "profile.name"
  | "profile.avatar"
  | "profile.avatar.remove"
  | "password.change"
  // meditation
  | "statistics.load"
  | "meditation.load"
  | "library.load"
  | "programs.load"
  | "program.load"
  | "program.members.load"
  | "program.member.add"
  | "program.member.remove"
  | "program.members.reorder"
  | "program.category"
  | "program.create"
  | "program.update"
  | "program.cover"
  | "continue.load"
  | "suggestion.load"
  | "recentPrograms.load"
  | "categories.load"
  | "category.create"
  | "category.rename"
  | "category.delete"
  | "playback.load"
  | "playback.position"
  | "playback.completion"
  | "meditation.completed"
  | "notes.load"
  | "note.create"
  | "note.update"
  | "note.delete"
  | "favorites.load"
  | "favorite.add"
  | "favorite.remove"
  | "audio.upload"
  | "meditation.description"
  // mood
  | "mood.list"
  | "mood.create"
  | "mood.update"
  | "mood.delete"
  | "mood.statistics"
  | "timeline.load"
  // portability
  | "import"
  // soundscape
  | "soundscape.categories.load"
  | "soundscape.category.create"
  | "soundscape.category.rename"
  | "soundscape.category.delete"
  | "soundscape.sounds.load"
  | "sound.upload"
  | "sound.update"
  | "sound.icon.upload"
  | "sound.icon.clear"
  | "sound.delete"
  | "mixes.load"
  | "mix.load"
  | "mix.create"
  | "mix.update"
  | "mix.delete";

/**
 * A failed API call. `status` 0 means the server was not reached (network).
 * `message` is the server's text or `request failed (status)`, so callers
 * that display `err.message` keep working.
 */
export class ApiError extends Error {
  readonly status: number;
  readonly action: ApiAction;
  readonly serverMessage: string;

  constructor(status: number, action: ApiAction, serverMessage: string) {
    super(
      serverMessage ||
        (status === 0 ? "network error" : `request failed (${status})`),
    );
    this.name = "ApiError";
    this.status = status;
    this.action = action;
    this.serverMessage = serverMessage;
  }
}

/** The raw body text and, when it is JSON, the parsed value. */
export async function readErrorBody(
  response: Response,
): Promise<{ message: string; data: unknown }> {
  const text = await response.text().catch(() => "");
  if (!text) return { message: "", data: undefined };
  try {
    const data: unknown = JSON.parse(text);
    const error =
      data && typeof data === "object" && "error" in data
        ? (data as { error: unknown }).error
        : undefined;
    return { message: typeof error === "string" ? error : text, data };
  } catch {
    return { message: text, data: undefined };
  }
}
