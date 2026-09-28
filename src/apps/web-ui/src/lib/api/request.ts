/**
 * Shared typed request helper for the module API clients. Every call is
 * same-origin with session cookies; JSON errors surface as thrown Error
 * values. FormData bodies (uploads) skip the JSON content type.
 */

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    credentials: "same-origin",
    ...init,
    headers: {
      ...(init?.body && !(init.body instanceof FormData)
        ? { "content-type": "application/json" }
        : {}),
      ...init?.headers,
    },
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as {
      error?: string;
    };
    throw new Error(body.error ?? `request failed (${response.status})`);
  }
  // 204 is not the only empty-body success: some handlers answer 201
  // with no payload, so parse only when a body exists.
  if (
    response.status === 204 ||
    response.headers.get("content-length") === "0"
  ) {
    return undefined as T;
  }
  const text = await response.text();
  if (!text) return undefined as T;
  return JSON.parse(text) as T;
}
