// Evidence files live under .agents/evidence/ui-ux-design/<task>/ (gitignored).
import fs from "node:fs";
import path from "node:path";
import type { Page } from "@playwright/test";
import { evidenceRoot } from "./server-env";

/** Resolves (and creates) the evidence dir for a plan task such as "T13". */
export function evidenceDir(task: string): string {
  if (!/^T\d+$/.test(task))
    throw new Error(`evidence task must look like "T13" (got "${task}")`);
  const dir = path.join(evidenceRoot, task);
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

const safeName = (name: string): string =>
  name.replace(/[^a-zA-Z0-9._-]+/g, "-");

export async function saveScreenshot(
  page: Page,
  task: string,
  name: string,
  fullPage = true,
): Promise<string> {
  const file = path.join(evidenceDir(task), `${safeName(name)}.png`);
  await page.screenshot({ path: file, fullPage });
  return file;
}

export function saveJson(task: string, name: string, data: unknown): string {
  const file = path.join(evidenceDir(task), `${safeName(name)}.json`);
  fs.writeFileSync(file, `${JSON.stringify(data, null, 2)}\n`);
  return file;
}
