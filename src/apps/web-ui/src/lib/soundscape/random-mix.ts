// A random starter mix: up to 3 distinct sounds, each at volume 50.

export const RANDOM_MIX_SIZE = 3;
export const RANDOM_MIX_VOLUME = 50;

/** `rand` returns [0, 1) like Math.random; pass a stub for determinism. */
export function randomMix<T>(
  sounds: readonly T[],
  rand: () => number = Math.random,
): { sound: T; volume: number }[] {
  const pool = [...sounds];
  const picked: { sound: T; volume: number }[] = [];
  while (pool.length > 0 && picked.length < RANDOM_MIX_SIZE) {
    const index = Math.min(pool.length - 1, Math.floor(rand() * pool.length));
    picked.push({ sound: pool.splice(index, 1)[0], volume: RANDOM_MIX_VOLUME });
  }
  return picked;
}
