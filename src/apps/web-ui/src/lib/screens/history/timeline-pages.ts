// Timeline paging (PB-10): "Load more" appends the next cursor page without
// duplicates or omissions. An event is identified by its kind and id, since
// mood entries and meditation events have separate id sequences.

type Keyed = { readonly kind: string; readonly id: number };

export const eventKey = (event: Keyed): string => `${event.kind}:${event.id}`;

/** `current` followed by every item of `page` whose kind+id is not already present. */
export function appendPage<T extends Keyed>(
  current: readonly T[],
  page: readonly T[],
): T[] {
  const seen = new Set(current.map(eventKey));
  const merged = [...current];
  for (const event of page) {
    const key = eventKey(event);
    if (seen.has(key)) continue;
    seen.add(key);
    merged.push(event);
  }
  return merged;
}
