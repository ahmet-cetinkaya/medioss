/** Instance facts for the About screen (`GET /api/meta`, public, no user data). */

import { request } from "./request";

export type AppMeta = {
  name: string;
  version: string;
  source: string;
  license: string;
};

export async function fetchAppMeta(): Promise<AppMeta> {
  return request<AppMeta>("/api/meta", undefined, { action: "loadMeta" });
}
