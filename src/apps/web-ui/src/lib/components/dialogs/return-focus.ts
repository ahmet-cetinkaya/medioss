// Where focus goes when a dialog closes (plan D6, FR-017/FR-018). Pure: the
// callers pass elements in; nothing here reads `document`.

export interface FocusCandidate {
  readonly isConnected: boolean;
}

/** An element, or a getter evaluated when the dialog closes. */
export type FocusSource<T> =
  T | null | undefined | (() => T | null | undefined);

/**
 * The opener while it is still in the document; otherwise the fallback when
 * it is connected; otherwise null (let the dialog library decide).
 */
export function resolveReturnFocus<T extends FocusCandidate>(
  trigger: T | null | undefined,
  fallback?: T | null,
): T | null {
  if (trigger?.isConnected) return trigger;
  if (fallback?.isConnected) return fallback;
  return null;
}

/**
 * `onCloseAutoFocus` handler body: focuses the resolved target and cancels
 * the library default; leaves the default alone when there is no target.
 */
export function restoreFocus<T extends FocusCandidate & { focus(): void }>(
  event: { preventDefault(): void },
  trigger: T | null | undefined,
  fallback?: FocusSource<T>,
): void {
  const target = resolveReturnFocus(
    trigger,
    typeof fallback === "function" ? fallback() : fallback,
  );
  if (!target) return;
  event.preventDefault();
  target.focus();
}
