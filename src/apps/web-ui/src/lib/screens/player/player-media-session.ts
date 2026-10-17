export type MediaSessionControls = {
  play: () => void;
  pause: () => void;
  skip: (delta: number) => void;
  seekTo: (seconds: number) => void;
  stop: () => void;
};

export const mediaSession = () =>
  typeof navigator !== "undefined" && "mediaSession" in navigator
    ? navigator.mediaSession
    : null;

/** Mirror the audio element's paused state into the Media Session. */
export function syncSessionState(audio: HTMLAudioElement | null) {
  const session = mediaSession();
  if (session && audio)
    session.playbackState = audio.paused ? "paused" : "playing";
}

function setActionHandlerSafely(
  session: MediaSession,
  action: MediaSessionAction,
  handler: MediaSessionActionHandler | null,
) {
  try {
    session.setActionHandler(action, handler);
  } catch {
    // Unsupported action in this browser.
  }
}

/** Register the Media Session (PB-5); returns the cleanup, or undefined when unsupported. */
export function bindMediaSession(
  title: string,
  skipSeconds: number,
  controls: MediaSessionControls,
): (() => void) | undefined {
  const session = mediaSession();
  if (!session) return;
  session.metadata = new MediaMetadata({ title });
  const handlers: [MediaSessionAction, MediaSessionActionHandler][] = [
    ["play", controls.play],
    ["pause", controls.pause],
    ["seekbackward", () => controls.skip(-skipSeconds)],
    ["seekforward", () => controls.skip(skipSeconds)],
    [
      "seekto",
      (details) => {
        if (details.seekTime !== undefined) controls.seekTo(details.seekTime);
      },
    ],
    ["stop", controls.stop],
  ];
  for (const [action, handler] of handlers)
    setActionHandlerSafely(session, action, handler);
  return () => {
    for (const [action] of handlers)
      setActionHandlerSafely(session, action, null);
    session.metadata = null;
  };
}
