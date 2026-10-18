# Demo data and screenshots

`make demo` starts a throwaway local instance, fills it with deterministic demo content, and writes PNG screenshots of the main screens to [`docs/screenshots/`](screenshots/). It is local only: the server binds to `127.0.0.1` (`MEDIOSS_BIND`), so it is not reachable from other machines, nothing is deployed, and the data directory is deleted when the script ends.

```bash
nix develop -c make demo    # NixOS: the dev shell provides Chromium's libraries
make demo                   # elsewhere: needs bun, ffmpeg, openssl and a Playwright Chromium
```

## What it does

1. `scripts/demo.sh` runs `make build`, then starts `target/release/medioss` on `DEMO_PORT` (default 3917) with a temporary data directory and a random `SESSION_SECRET`.
2. `make demo-seed` (`src/apps/web-ui/demo/seed-demo.ts`) registers the test-only admin and member from `e2e/support/auth.ts` (`@example.test` addresses). It then creates three programs with covers and sessions, two uploaded sounds, a mix of built-in sounds, 45 days of mood entries and ten completed sessions. The content is fixed, so every run looks the same apart from dates. It refuses an instance that already has programs.
3. `make demo-screenshots` (`demo/screenshots.ts`) signs in with those users and captures 11 screens at desktop (1280×800) and mobile (390×844).

The two targets can also be run by hand against your own empty instance on `DEMO_PORT`: set `SESSION_SECRET` to the value that instance was started with, because first-run setup needs it.

## Notes

- The credentials are test-only and exist only on the throwaway instance. Do not seed an instance you care about.
- Screenshots are taken in English. The UI has no dark theme yet, so only the light theme is captured.
- Generated audio and cover files are cached in `.agents/qa/demo/` (gitignored). Demo covers are Unsplash photos fetched from `images.unsplash.com` at seed time, so `make demo` needs network access. They are used under the [Unsplash License](https://unsplash.com/license), which does not require attribution; each photo's page (with its photographer credit) is reachable through its image id on the CDN — the ids are listed in `COVER_PHOTO_IDS` in [`demo-content.ts`](../src/apps/web-ui/demo/demo-content.ts).
