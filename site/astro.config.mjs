// engine.origingame.ai: a static site (docs: README.md). Media lives outside
// the build (public/m/ locally, R2 behind PUBLIC_MEDIA_BASE on Cloudflare).
import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://engine.origingame.ai",
  output: "static",
  trailingSlash: "ignore",
  build: { inlineStylesheets: "auto", format: "directory" },
  vite: {
    server: { fs: { allow: [".."] } },
    json: { stringify: false },
  },
});
