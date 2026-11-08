// Colour-blind modes on the web (docs/ui.md §3.2): og-ui's colour vision
// frame (crates/og-ui/tests/support/scenes.rs `color_vision_scene`, the
// overlay's swatches over a scene colour) drawn by crates/og-tests-web in
// headless Chrome on WebGPU at 1280×720 and 390×844, once unfiltered and
// once for each colour vision and filter. Each filtered frame must be the
// CPU filter (`og_render::ColorFilter::apply_srgb8`) of the unfiltered one
// within 2/255 on every channel of every pixel, must change at least an
// eighth of the pixels, and taking the filter off must give back the
// unfiltered frame exactly: the same checks as the native test
// (crates/og-ui/tests/color_vision.rs), so the WGSL pass and its routing
// through the glass source behave alike on both hosts.
//
//   bun tests/web/color-vision.ts [--chrome <path>]
//
// Needs Chrome (cdp.ts), a GPU, wasm-bindgen at the Cargo.lock version and
// the wasm32 target. Writes dist/test-color-vision/color-vision.json; exits 1
// on a failure.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const work = join(pkg, "dist", "test-color-vision");
mkdirSync(work, { recursive: true });

function run(cmd: string[]): string {
  const r = Bun.spawnSync(cmd, { cwd: repo, stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`${cmd.join(" ")}: ${new TextDecoder().decode(r.stderr).slice(-4000)}`);
  return new TextDecoder().decode(r.stdout);
}

// og-tests-web in its own target dir, as reference-gfx.ts builds it.
const targetDir = join(process.env.CARGO_TARGET_DIR ? resolve(repo, process.env.CARGO_TARGET_DIR) : join(repo, "target"), "wasm-gfx");
const messages = run(["cargo", "build", "--profile", "wasm-test", "--target", "wasm32-unknown-unknown", "--target-dir", targetDir, "-p", "og-tests-web", "--lib", "--message-format=json-render-diagnostics"]);
const raw = messages
  .split("\n")
  .filter((l) => l.includes('"reason":"compiler-artifact"') && l.includes("og_tests_web"))
  .flatMap((l) => (JSON.parse(l) as { filenames: string[] }).filenames)
  .find((f) => f.endsWith(".wasm"));
if (!raw) throw new Error("cargo built no og_tests_web.wasm");
const lock = /name = "wasm-bindgen"\nversion = "([^"]+)"/.exec(readFileSync(join(repo, "Cargo.lock"), "utf8"))?.[1];
const have = run(["wasm-bindgen", "--version"]).trim().split(" ")[1];
if (lock !== have) throw new Error(`wasm-bindgen ${have} does not match Cargo.lock ${lock} (scripts/install-tools.sh)`);
const dist = join(work, "site");
run(["wasm-bindgen", "--target", "web", "--out-dir", join(dist, "pkg"), "--out-name", "og-tests", raw]);

const html = `<!doctype html><html><head><meta charset="utf-8"><title>colour vision</title></head><body>
<script type="module">
import init, * as gfx from "/game/pkg/og-tests.js";
try { await init(); globalThis.__gfx = gfx; } catch (e) { globalThis.__gfx_error = String(e && e.stack || e); }
</script></body></html>`;
const server = serve({ dist, isolate: false, extra: { "/cv.html": html } });
const browser = await Browser.launch({ chrome: chromePath(args), width: 1280, height: 720 });
const log: string[] = [];
type Row = { scene: string; vision: string; filter: string; worst: number; changed: number; pixels: number; ok: boolean };
const rows: Row[] = [];
let failures = 0;
try {
  const tab = await browser.open(`http://127.0.0.1:${server.port}/cv.html`, log);
  await tab.until("globalThis.__gfx || globalThis.__gfx_error", 60_000);
  const failed = await tab.eval<string | undefined>("globalThis.__gfx_error");
  if (failed) throw new Error(`the module did not load: ${failed}`);
  for (const scene of ["og-ui/color-vision-1280", "og-ui/color-vision-390"]) {
    const result = await tab.eval<{ adapter: string; rows: Row[]; restored: boolean }>(`(async () => {
      const gfx = globalThis.__gfx;
      const o = JSON.parse(await gfx.og_gfx_open(${JSON.stringify(scene)}));
      const frame = async () => { gfx.og_gfx_render(1); return await gfx.og_gfx_capture(); };
      const plain = await frame();
      const rows = [];
      for (const vision of ["protanopia", "deuteranopia", "tritanopia"]) {
        for (const filter of ["correct", "simulate"]) {
          gfx.og_gfx_color_filter(vision, filter);
          const got = await frame();
          const want = gfx.og_gfx_color_filter_cpu(vision, filter, plain);
          let worst = 0, changed = 0;
          for (let i = 0; i < got.length; i += 4) {
            for (let k = 0; k < 3; k++) worst = Math.max(worst, Math.abs(got[i + k] - want[i + k]));
            if (got[i] !== plain[i] || got[i + 1] !== plain[i + 1] || got[i + 2] !== plain[i + 2]) changed++;
          }
          const pixels = got.length / 4;
          rows.push({ scene: ${JSON.stringify(scene)}, vision, filter, worst, changed, pixels, ok: worst <= 2 && changed > pixels / 8 });
        }
      }
      gfx.og_gfx_color_filter("none", "correct");
      const back = await frame();
      let restored = back.length === plain.length;
      for (let i = 0; restored && i < back.length; i++) restored = back[i] === plain[i];
      gfx.og_gfx_close();
      return { adapter: o.adapter, rows, restored };
    })()`);
    for (const r of result.rows) {
      rows.push(r);
      const line = `${r.scene} ${r.vision} ${r.filter}: worst ${r.worst}/255 off the CPU filter, ${r.changed} of ${r.pixels} pixels changed (${result.adapter})`;
      if (r.ok) console.log(`ok   ${line}`);
      else {
        failures++;
        console.error(`FAIL ${line}`);
      }
    }
    if (result.restored) console.log(`ok   ${scene}: no filter gives back the unfiltered frame`);
    else {
      failures++;
      console.error(`FAIL ${scene}: the frame with the filter off differs from the unfiltered one`);
    }
  }
} catch (e) {
  failures++;
  console.error(`FAIL ${e instanceof Error ? e.message : e}\n${log.slice(-20).join("\n")}`);
} finally {
  await browser.close();
  server.stop();
}
writeFileSync(join(work, "color-vision.json"), JSON.stringify({ rows, failures }, null, 2));
process.exit(failures > 0 ? 1 : 0);
