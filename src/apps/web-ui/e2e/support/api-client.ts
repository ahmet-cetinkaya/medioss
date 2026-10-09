// Thin API helpers over Playwright's request context. Every context sends the
// configured Origin, because the backend rejects mutations without it.
import {
  request,
  type APIRequestContext,
  type APIResponse,
} from "@playwright/test";
import { baseURL } from "./server-env";

export type Api = APIRequestContext;

export async function newApi(storageState?: string): Promise<Api> {
  return request.newContext({
    baseURL: baseURL(),
    extraHTTPHeaders: { Origin: new URL(baseURL()).origin },
    storageState,
  });
}

/** Throws with status and body (server messages carry no secrets) unless 2xx. */
export async function expectOk(
  response: APIResponse,
  action: string,
): Promise<APIResponse> {
  if (!response.ok()) {
    const body = await response.text().catch(() => "");
    throw new Error(
      `${action} failed: HTTP ${response.status()} ${body.slice(0, 300)}`,
    );
  }
  return response;
}

export async function json<T>(
  response: Promise<APIResponse>,
  action: string,
): Promise<T> {
  return (await expectOk(await response, action)).json() as Promise<T>;
}

export async function getJson<T>(api: Api, url: string): Promise<T> {
  return json<T>(api.get(url), `GET ${url}`);
}

export async function postJson<T>(
  api: Api,
  url: string,
  data: unknown,
): Promise<T> {
  return json<T>(api.post(url, { data }), `POST ${url}`);
}

/** Multipart upload; files are `{ name, mimeType, buffer }`. */
export async function postMultipart<T>(
  api: Api,
  url: string,
  multipart: Record<
    string,
    string | { name: string; mimeType: string; buffer: Buffer }
  >,
): Promise<T> {
  return json<T>(api.post(url, { multipart }), `POST ${url}`);
}

export async function healthStatus(api: Api): Promise<{ status: string }> {
  return getJson(api, "/api/health");
}
