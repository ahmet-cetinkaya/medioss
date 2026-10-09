// Client-side avatar check (FR-068, PB-12): runs before any upload request.

/** The types the server accepts; also the file input's `accept`. */
export const AVATAR_TYPES = ["image/jpeg", "image/png", "image/webp"] as const;

/** 2 MB, the server's `MAX_AVATAR_BYTES`. */
export const AVATAR_MAX_BYTES = 2 * 1024 * 1024;

export type AvatarFileProblem = "type" | "size";

/** Why `file` cannot be uploaded as an avatar, or null when it can. */
export function avatarFileProblem(file: {
  type: string;
  size: number;
}): AvatarFileProblem | null {
  if (!(AVATAR_TYPES as readonly string[]).includes(file.type)) return "type";
  return file.size > AVATAR_MAX_BYTES ? "size" : null;
}
