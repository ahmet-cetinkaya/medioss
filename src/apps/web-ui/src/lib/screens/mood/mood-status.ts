export type RegionStatus = "loading" | "error" | "empty" | "ready";

/** The status of a region fed by one request: failed, still loading, empty or ready. */
export function regionStatus(
  failed: boolean,
  loaded: boolean,
  empty: boolean,
): RegionStatus {
  if (failed) return "error";
  if (!loaded) return "loading";
  return empty ? "empty" : "ready";
}
