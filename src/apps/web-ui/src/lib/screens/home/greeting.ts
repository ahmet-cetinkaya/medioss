// Home greeting (FR-022/023, SC-011). Pure: the caller passes the local time
// and the session user. The name is the display name exactly as entered; the
// email is never read, so it can never appear in the greeting.

export type GreetingBand = "morning" | "afternoon" | "evening";

export type Greeting = { band: GreetingBand; name: string | null };

/** 05:00–11:59 morning, 12:00–17:59 afternoon, 18:00–04:59 evening. */
export function greetingBand(hour: number): GreetingBand {
  if (hour >= 5 && hour < 12) return "morning";
  if (hour >= 12 && hour < 18) return "afternoon";
  return "evening";
}

/** The band for `now` (device local time) and the user's display name, if any. */
export function greeting(
  now: Date,
  user: { display_name: string | null } | null,
): Greeting {
  const name = user?.display_name;
  return {
    band: greetingBand(now.getHours()),
    name: name && name.trim() ? name : null,
  };
}
