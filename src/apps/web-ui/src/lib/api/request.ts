/**
 * Shared typed request helper for the module API clients (plan D2, D8).
 * Every call is same-origin with session cookies. FormData bodies (uploads)
 * skip the JSON content type. Failures throw `ApiError`:
 * - a fetch rejection becomes `ApiError{status: 0}` (network), except an
 *   `AbortError`, which is rethrown unchanged so aborted calls stay silent;
 * - a 401 calls `redirectToLogin()` and returns a promise that never settles,
 *   unless the call opts out with `onUnauthorized: "throw"` or the redirect
 *   is skipped (on Login or Setup), which throws `ApiError{status: 401}`;
 * - any other non-2xx status throws `ApiError` with the parsed body text.
 * The action defaults to "generic"; every client function passes its own.
 */

import { redirectToLogin } from "../navigation/login-redirect";
import { ApiError, readErrorBody, type ApiAction } from "./api-error";

export interface RequestOptions {
  action?: ApiAction;
  onUnauthorized?: "redirect" | "throw";
}

type FetchFn = (input: string, init?: RequestInit) => Promise<Response>;

let testFetch: FetchFn | null = null;
let testRedirect: (() => boolean) | null = null;

/** Test-only: replaces `fetch` and the login redirect; call with {} to reset. */
export function setRequestDependenciesForTests(deps: {
  fetch?: FetchFn;
  redirectToLogin?: () => boolean;
}): void {
  testFetch = deps.fetch ?? null;
  testRedirect = deps.redirectToLogin ?? null;
}

/**
 * Sends the request and returns the raw response of any status. Only a
 * network failure throws (`ApiError{status: 0}`). For clients that read
 * headers of a refusal, such as login's `Retry-After`.
 */
export async function send(
  path: string,
  init: RequestInit | undefined,
  action: ApiAction,
): Promise<Response> {
  const doFetch = testFetch ?? fetch;
  const headers = new Headers(init?.headers);
  if (
    init?.body &&
    !(init.body instanceof FormData) &&
    !headers.has("content-type")
  )
    headers.set("content-type", "application/json");
  try {
    return await doFetch(path, {
      credentials: "same-origin",
      ...init,
      headers,
    });
  } catch (error) {
    if ((error as { name?: unknown } | null)?.name === "AbortError")
      throw error;
    throw new ApiError(0, action, "");
  }
}

/** Builds the `ApiError` for a non-2xx response. */
export async function apiErrorFrom(
  response: Response,
  action: ApiAction,
): Promise<ApiError> {
  const { message } = await readErrorBody(response);
  return new ApiError(response.status, action, message);
}

export async function request<T>(
  path: string,
  init?: RequestInit,
  options: RequestOptions = {},
): Promise<T> {
  const action = options.action ?? "generic";
  const response = await send(path, init, action);
  if (response.status === 401 && options.onUnauthorized !== "throw") {
    if ((testRedirect ?? redirectToLogin)())
      return new Promise<never>(() => {});
    throw await apiErrorFrom(response, action);
  }
  if (!response.ok) throw await apiErrorFrom(response, action);
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
  // A 2xx body that is not valid JSON (reverse-proxy interstitial, truncated
  // response) must not break the ApiError contract with a raw SyntaxError.
  try {
    return JSON.parse(text) as T;
  } catch {
    throw new ApiError(response.status, action, "");
  }
}
