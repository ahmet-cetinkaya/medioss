/**
 * The background mix plays on through the end-of-meditation dialog as a soft
 * exit, and stops the moment that dialog closes by any path (Continue, X,
 * Escape, backdrop), even when the user stays on the player.
 *
 * Returns a function to call with the dialog's open state on every change;
 * it runs `onClosed` once per open -> closed transition and never for the
 * initial closed state.
 */
export function createDialogCloseWatcher(
  onClosed: () => void,
): (open: boolean) => void {
  let wasOpen = false;
  return (open) => {
    const justClosed = wasOpen && !open;
    wasOpen = open;
    if (justClosed) onClosed();
  };
}
