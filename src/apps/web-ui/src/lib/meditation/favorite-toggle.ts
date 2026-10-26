/**
 * Flips a favorite heart optimistically: `apply` shows the new state at once
 * and, when the server call fails, restores the old one before rethrowing so
 * the caller can surface the failure.
 */
export async function toggleFavoriteOptimistically(
  current: boolean,
  apply: (favorite: boolean) => void,
  send: (next: boolean) => Promise<void>,
): Promise<void> {
  const next = !current;
  apply(next);
  try {
    await send(next);
  } catch (error) {
    apply(current);
    throw error;
  }
}
