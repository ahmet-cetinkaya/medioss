import { fileURLToPath } from "node:url";
import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  integrations: [svelte()],
  // Dev-only: proxy API calls to the Axum server so `make dev` serves the
  // live Astro UI on :4321 against the real backend on :3000. The same
  // origin stays http://localhost:4321, so the browser's Origin header
  // matches MEDIOSS_DEV_ORIGIN (see scripts/dev.sh); production is unaffected
  // (no proxy in the built output, the binary serves the static dist).
  devToolbar: { enabled: false },
  vite: {
    resolve: {
      alias: {
        $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
      },
    },
    server: {
      proxy: {
        "/api": {
          target: process.env.ASTRO_API_TARGET ?? "http://localhost:3000",
          changeOrigin: false,
        },
      },
    },
    plugins: [
      tailwindcss(),
      // Dev-only SPA fallback mirroring the Axum routes in
      // web-api/router.rs: path-addressed pages whose <id> no static file can
      // cover (/player/1/, /admin/meditations/2/) are rewritten to their
      // page so `astro dev` serves the live source, never a stale build.
      // Runs before Vite's internal middlewares (direct .use, not returned),
      // and the URL patterns never match /api or /_astro.
      {
        name: "dev-spa-fallback",
        configureServer(server) {
          server.middlewares.use((req, _res, next) => {
            const url = req.url ?? "";
            if (/^\/player\/[^/]+\/?$/.test(url)) req.url = "/player/";
            else if (/^\/admin\/meditations\/[^/]+\/?$/.test(url))
              req.url = "/admin/meditations/";
            else if (/^\/admin\/soundscape\/[^/]+\/?$/.test(url))
              req.url = "/admin/soundscape/";
            next();
          });
        },
      },
    ],
  },
  outDir: "dist",
});
