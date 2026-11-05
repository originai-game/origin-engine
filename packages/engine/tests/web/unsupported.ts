// A browser without WebGPU (docs/design.md §8: the web is WebGPU only, with
// no WebGL2 tier): Chrome started with WebGPU turned off, so the page has no
// `navigator.gpu`, and Chrome with no GPU at all (no adapter comes back),
// each opening the runner's publish build framed by a stand-in Player at the
// same origin, as play.<domain>/g/:slug frames a release.
//
// Asserts, for each: the shell's card in the brand's ink is on screen with
// the reason and what to do (never a blank canvas), without the Reload
// button a reload cannot help; the page threw nothing uncaught; the Worker
// never started; and the Player heard the report: a `sample` with phase
// `unsupported`, the reason, the browser and the device (field telemetry,
// docs/web-export.md §Telemetry), then `fail` (`engine_start`, not
// retryable), and never `ready`.
//
//   bun tests/web/unsupported.ts [--chrome <path>] [--evidence <dir>]
//
// --evidence writes each card's screenshot and the Player's messages there.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "archetypes", "runner");
const work = join(pkg, "dist", "test-unsupported");
const evidence = opt("--evidence") ? resolve(opt("--evidence")!) : undefined;
const chrome = chromePath(args);
const b = await webBuild(project, join(project, "src", "main.ts"), join(work, "publish"), true, settingsOf(project));
if (!b.ok) throw new Error(b.reason);

const player = `<!doctype html><html><head><meta charset="utf-8"></head><body style="margin:0">
<iframe id="g" src="/game/index.html" style="width:1280px;height:720px;border:0"></iframe>
<script>
const g = document.getElementById("g");
window.got = [];
addEventListener("message", (e) => {
  if (e.source !== g.contentWindow || e.origin !== location.origin) return;
  const m = e.data;
  if (!m || m.og !== "player" || m.protocol !== 2) return;
  got.push(m);
  if (m.kind === "hello") g.contentWindow.postMessage({ og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", capabilities: [], started: false, paused: false }, location.origin);
});
</script></body></html>`;

let failures = 0;
const check = (ok: boolean, what: string, detail?: unknown) => {
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${ok ? "" : `: ${String(JSON.stringify(detail)).slice(0, 600)}`}`);
};

type Message = { kind: string; phase?: string; reason?: string; renderer?: string; browser?: { name?: string; major?: number; os?: string }; device?: Record<string, unknown>; code?: string; message?: string; retryable?: boolean };
const cases = [
  // A browser without WebGPU: Chrome without its WebGPU switches, and the page's Navigator without
  // `gpu` before its first script (Chrome 156 keeps navigator.gpu under --disable-features=WebGPU
  // and --disable-blink-features=WebGPU alike).
  { name: "no-navigator-gpu", webgpu: false, flags: [], strip: true, reason: "navigator.gpu is missing", shown: "(navigator.gpu is missing)", title: "This browser does not support WebGPU", advice: "Open it in a current Chrome or Edge" },
  // No GPU and no software fallback: navigator.gpu is there, requestAdapter answers null.
  { name: "no-adapter", webgpu: true, flags: ["--disable-gpu", "--disable-software-rasterizer"], strip: false, reason: "the browser found no WebGPU adapter", shown: "(navigator.gpu.requestAdapter() returned null)", title: "No graphics adapter for WebGPU", advice: "turn on hardware acceleration" },
] as const;
const STRIP_GPU = "delete Navigator.prototype.gpu; if (typeof WorkerNavigator !== 'undefined') delete WorkerNavigator.prototype.gpu;";

const server = serve({ dist: join(work, "publish"), isolate: true, extra: { "/g/runner": player } });
try {
  for (const c of cases) {
    const browser = await Browser.launch({ chrome, width: 1280, height: 720, webgpu: c.webgpu, flags: [...c.flags] });
    try {
      const log: string[] = [];
      const page = await browser.open("about:blank", log);
      // Every frame of the page, the game's iframe included (same origin, in process).
      if (c.strip) await page.send("Page.addScriptToEvaluateOnNewDocument", { source: STRIP_GPU });
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/g/runner` });
      await page.until("got.some((m) => m.kind === 'fail')", 30_000).catch(() => null);
      const gpu = await page.eval<string>("typeof g.contentWindow.navigator.gpu");
      check(c.name === "no-navigator-gpu" ? gpu === "undefined" : gpu === "object", `${c.name}: navigator.gpu is ${c.name === "no-navigator-gpu" ? "missing" : "there"}`, gpu);
      const got = await page.eval<Message[]>("got");
      const card = await page.eval<{ hidden: boolean; title: string; text: string; reload: boolean; ink: string; canvas: boolean }>(`(() => {
        const d = g.contentWindow.document, e = d.getElementById("og-error");
        const r = e.querySelector("[data-action=reload]");
        return { hidden: e.hidden, title: e.querySelector("h1").textContent, text: e.querySelector("p").textContent,
          reload: !!r && !r.hidden && r.getClientRects().length > 0, ink: g.contentWindow.getComputedStyle(e).backgroundColor,
          canvas: !!g.contentWindow.__runtime_probe__ };
      })()`);
      check(!card.hidden && card.title === c.title, `${c.name}: the card is on screen with its title`, card);
      check(card.text.includes(c.advice) && card.text.includes(c.shown) && !card.text.includes("(the browser found no WebGPU adapter)"), `${c.name}: the card says why and what to do, the reason once`, card.text);
      check(!card.reload, `${c.name}: no Reload button (a reload cannot help)`, card);
      check(card.ink !== "rgba(0, 0, 0, 0)" && card.ink !== "", `${c.name}: the card is drawn in the brand's ink, not over a bare canvas`, card.ink);
      check(!card.canvas, `${c.name}: the engine never started (no probe)`, card);
      const uncaught = log.filter((l) => l.startsWith("exception:"));
      check(uncaught.length === 0, `${c.name}: nothing uncaught on the page`, uncaught);
      const sample = got.find((m) => m.kind === "sample");
      check(sample?.phase === "unsupported" && sample.renderer === "none" && sample.reason === c.reason, `${c.name}: the Player hears the unsupported sample with its reason`, sample);
      check(sample?.browser?.name === "chrome" && typeof sample.browser.major === "number" && sample.browser.os === (process.platform === "win32" ? "windows" : process.platform === "darwin" ? "macos" : "linux"), `${c.name}: the sample names the browser and its OS`, sample?.browser);
      check(!!sample?.device && sample.device["class"] === "desktop", `${c.name}: the sample names the device`, sample?.device);
      const fail = got.find((m) => m.kind === "fail");
      check(fail?.code === "engine_start" && fail.retryable === false && !!fail.message?.includes(c.shown), `${c.name}: then fail, engine_start, not retryable, with the reason`, fail);
      const kinds = got.map((m) => m.kind);
      check(kinds.includes("sample") && kinds.indexOf("sample") < kinds.indexOf("fail") && !kinds.includes("ready") && !kinds.includes("crash"), `${c.name}: sample before fail, never ready or crash`, kinds);
      if (evidence) {
        mkdirSync(evidence, { recursive: true });
        writeFileSync(join(evidence, `unsupported-${c.name}.png`), await page.screenshot());
        writeFileSync(join(evidence, `unsupported-${c.name}.json`), JSON.stringify({ chrome, flags: c.flags, card, messages: got, log }, null, 1) + "\n");
      }
    } finally {
      await browser.close();
    }
  }
} finally {
  server.stop();
}
console.log(`${failures === 0 ? "PASS" : "FAIL"} unsupported: ${failures} failed`);
process.exit(failures === 0 ? 0 : 1);
