// Resolves where the E2E server runs and how Playwright starts it (plan D7).
//
// Default mode: `make run` on port 3000 (real-surface QA). Isolated mode
// (`E2E_PORT` set): the UI is built into its own static dir and the already
// built release binary serves it, so parallel runs never share a port, a data
// dir, or the `src/apps/web-api/static` copy.
//
// Playwright evaluates the config in the runner and again in every worker, so
// the run id is computed once and handed down through `process.env`.
import fs from "node:fs";
import path from "node:path";

export const webUiDir = path.resolve(__dirname, "..", "..");
export const repoRoot = path.resolve(webUiDir, "..", "..", "..");
export const qaDir = path.join(repoRoot, ".agents", "qa");
export const fixturesDir = path.join(qaDir, "fixtures");
export const evidenceRoot = path.join(
  repoRoot,
  ".agents",
  "evidence",
  "ui-ux-design",
);
export const builtinSoundsDir = path.join(
  repoRoot,
  "assets",
  "sounds",
  "soundscape-builtin",
);

const DEFAULT_PORT = 3000;

function runId(): string {
  process.env.E2E_RUN_ID ??= new Date()
    .toISOString()
    .replace(/[-:]/g, "")
    .replace(/\..*$/, "");
  return process.env.E2E_RUN_ID;
}

export function e2ePort(): number {
  const raw = process.env.E2E_PORT;
  if (!raw) return DEFAULT_PORT;
  const port = Number(raw);
  if (!Number.isInteger(port) || port < 1024 || port > 65535) {
    throw new Error(
      `E2E_PORT must be an integer port between 1024 and 65535 (got "${raw}")`,
    );
  }
  return port;
}

export const isIsolated = (): boolean => Boolean(process.env.E2E_PORT);

export const baseURL = (): string => `http://localhost:${e2ePort()}`;

/** The data dir of the server this run started; fixed once per run. */
export function e2eDataDir(): string {
  const port = e2ePort();
  process.env.E2E_DATA_DIR ??= path.join(
    qaDir,
    isIsolated() ? `data-${port}-${runId()}` : `data-${runId()}`,
  );
  return process.env.E2E_DATA_DIR;
}

/** Per-port so concurrent isolated runs never wipe each other's results. */
export const outputDir = (): string =>
  path.join(qaDir, `test-results-${e2ePort()}`);

/** Seed ids/names for later suites; per port, overwritten by every seed run. */
export const seedFile = (): string =>
  path.join(qaDir, `seed-${e2ePort()}.json`);

export const authDir = (): string => path.join(qaDir, `auth-${e2ePort()}`);

/** The setup secret; read from the caller's environment and never logged. */
export function sessionSecret(): string {
  const secret = process.env.SESSION_SECRET?.trim();
  if (!secret) {
    throw new Error(
      'SESSION_SECRET is not set. Export a test-only secret first, e.g. export SESSION_SECRET="$(openssl rand -base64 32)"',
    );
  }
  return secret;
}

type WebServer = {
  command: string;
  cwd: string;
  url: string;
  reuseExistingServer: boolean;
  timeout: number;
  env: Record<string, string>;
  gracefulShutdown: { signal: "SIGINT" | "SIGTERM"; timeout: number };
};

const quote = (value: string): string => `'${value.replace(/'/g, `'\\''`)}'`;

export function webServer(): WebServer {
  sessionSecret();
  const port = e2ePort();
  const dataDir = e2eDataDir();
  const env = {
    MEDIOSS_ORIGIN: baseURL(),
    SESSION_COOKIE_SECURE: "false",
    MEDIOSS_BUILTIN_SOUNDS_DIR: builtinSoundsDir,
    // SESSION_SECRET is inherited from the caller's environment unchanged.
  };
  const common = {
    cwd: repoRoot,
    url: `${baseURL()}/api/health`,
    reuseExistingServer: false,
    gracefulShutdown: { signal: "SIGTERM" as const, timeout: 10_000 },
  };

  if (!isIsolated()) {
    return {
      ...common,
      command: `make run DATA_DIR=${quote(dataDir)} PORT=${port}`,
      // First run compiles the release and debug binaries: allow 30 minutes.
      timeout: 30 * 60_000,
      env: {
        ...env,
        MEDIOSS_STATIC_DIR: path.join(
          repoRoot,
          "src",
          "apps",
          "web-api",
          "static",
        ),
      },
    };
  }

  // The web-api package builds a binary named "medioss".
  const binary = path.join(repoRoot, "target", "release", "medioss");
  if (!fs.existsSync(binary)) {
    throw new Error(
      `Isolated mode needs the release binary at ${binary}; run "cargo build --release -p web-api" (or make build) first.`,
    );
  }
  const staticDir = path.join(qaDir, `static-${port}`);
  return {
    ...common,
    command: `(cd ${quote(webUiDir)} && bunx astro build --outDir ${quote(staticDir)}) && exec ${quote(binary)}`,
    timeout: 10 * 60_000,
    env: {
      ...env,
      MEDIOSS_STATIC_DIR: staticDir,
      MEDIOSS_DATA_DIR: dataDir,
      PORT: String(port),
    },
  };
}
