// A font-pack face explicitly loaded in GameDesc.start, outside the host's
// locale chain: the real Worker must start at tick 0 with a ready, stable
// font and draw its text. Another pack stays lazy until a locale switch.
// Run after building dist/web (or dist/web-publish with --publish):
//   bun tests/web/font-start.ts [--publish] [--out <evidence directory>]
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { DEFAULT_PUBLISH, settingsOf, webBuild } from "../../src/node/web-build.ts";
import type { EngineModules } from "../../src/web/split.ts";
import { cookProject } from "../support/og-bin.ts";
import { decodePng } from "../support/png.ts";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { serve } from "./serve.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
};

const args = process.argv.slice(2);
const pkg = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const repo = resolve(pkg, "../..");
const out = resolve(args.includes("--out") ? args[args.indexOf("--out") + 1]! : join(repo, "target/evidence/web-font-start"));
const project = join(repo, "target/web-font-start-project");
rmSync(project, { recursive: true, force: true });
mkdirSync(join(project, "assets/fonts"), { recursive: true });
mkdirSync(join(project, "assets/strings"), { recursive: true });
for (const locale of ["en", "ja", "ar"]) writeFileSync(join(project, `assets/strings/${locale}.ogt`), `type strings\nversion 1\n---\nlocale "${locale}"\nstring "ready" "Ready"\n`);
mkdirSync(out, { recursive: true });
for (const pack of ["latin", "arabic"]) cpSync(join(repo, "templates/fonts", pack), join(project, "assets/fonts", pack), { recursive: true });
writeFileSync(join(project, "og-project.json"), '{"version":1,"name":"Font start","engine":"0.9.0"}\n');
writeFileSync(join(project, "project.ts"), 'import {defineProject} from "@origingame/engine/author"; export default defineProject({id:"font-start",entry:"main.ts",publish:{keep:["fonts/arabic/extension.json"]}});\n');
writeFileSync(join(project, "main.ts"), `import { og, defineSystem } from "@origingame/engine";
let font = 0, startTick = -1, same = false, draws = 0;
let hiddenResult = null;
const must = (r) => { if (!r.ok) throw new Error(r.reason); return r.value; };
const Draw = defineSystem("Draw", {}, (_q, _time, world) => {
  draws++;
  must(og.render.text(world, { font, text: "Skyfarer ready", screen: true, layer: 1100, position: { x: 40, y: 120, z: 0 }, size: 42 }));
});
export default og.game({
  systems: [Draw],
  start(world) {
    font = must(og.load.font(world, "fonts/latin/NotoSans-Latin.ttf"));
    same = font === must(og.load.font(world, "fonts/latin/NotoSans-Latin.ttf"));
    startTick = world.tick;
  },
  commands: { arabic: (world) => { og.i18n.setLocale(world, "ar"); }, hidden: (world, [code, path]) => { hiddenResult = new Function("og", "world", "path", code)(og, world, path); } },
  facts(world) { return { startTick, same, draws, tick: world.tick, hiddenResult, family: must(og.load.fontInfo(world, font)).family, fonts: og.i18n.locale(world).fonts, completions: og.load.completions(world) }; }
});
`);
cookProject(project, repo);
const dist = join(project, "dist");
const publish = args.includes("--publish");
const built = await webBuild(project, join(project, "main.ts"), dist, publish, { ...settingsOf(project), publish: { ...DEFAULT_PUBLISH, gate: "none" } });
if (!built.ok) throw new Error(built.reason);
const manifest = built.manifest!;
const config = JSON.parse(/<script type="application\/json" id="og-config">([^<]*)<\/script>/.exec(readFileSync(join(dist, "index.html"), "utf8"))![1]!) as { modules?: { single: EngineModules } };
const shellEntry = join(project, "page.ts");
writeFileSync(shellEntry, `import { Shell } from ${JSON.stringify(join(pkg, "src/web/shell.ts"))};
globalThis.__error = null;
globalThis.__logs = [];
try {
  const modules = ${JSON.stringify(config.modules)};
  if (modules) modules.single.base = new URL(modules.single.base || "/game/", location.href).href;
  globalThis.__shell = await Shell.start({ canvas: document.querySelector("canvas"),
    worker: new Worker(${JSON.stringify(`/game/${manifest.worker}`)}, { type: "module" }),
    glue: ${JSON.stringify(`/game/${manifest.names["origin-engine.js"] ?? "origin-engine.js"}`)},
    wasm: ${JSON.stringify(`/game/${manifest.wasm}`)}, modules, root: "/game", locale: "ja", autoplay: false, prime: true,
    onFirstFrame: () => { globalThis.__firstFrame = true; },
    onLog: (lines) => globalThis.__logs.push(...lines),
    onCrash: (message) => { globalThis.__error = message; } });
} catch (e) { globalThis.__error = String(e); }
`);
const pageBuild = await Bun.build({ entrypoints: [shellEntry], target: "browser", format: "esm" });
if (!pageBuild.success) throw new Error(pageBuild.logs.map(String).join("\n"));
writeFileSync(join(dist, "font-page.js"), await pageBuild.outputs[0]!.text());
const page = '<!doctype html><html><head><meta charset="utf-8"><title>Font start</title><style>html,body{margin:0;background:#14161b}canvas{width:640px;height:240px;display:block}</style></head><body><canvas></canvas><script type="module" src="/game/font-page.js"></script></body></html>';
const requests: string[] = [];
const server = serve({ dist, isolate: false, extra: { "/font.html": page }, onRequest: (p) => requests.push(p) });
const browser = await Browser.launch({ chrome: chromePath(args), width: 640, height: 240,
  flags: process.platform === "linux" ? ["--enable-features=WebGPU,Vulkan", "--use-angle=vulkan", "--disable-vulkan-surface"] : [],
});
const log: string[] = [];
const checks: string[] = [];
const check = (ok: boolean, what: string) => { if (!ok) throw new Error(what); checks.push(what); console.log(`ok ${what}`); };
let tab: Page | undefined;
try {
  tab = await browser.open(`http://127.0.0.1:${server.port}/font.html`, log);
  await tab.send("Emulation.setDeviceMetricsOverride", { width: 640, height: 240, deviceScaleFactor: 2, mobile: false });
  await tab.until("globalThis.__shell || globalThis.__error", 90_000);
  await tab.until("globalThis.__firstFrame || globalThis.__error", 90_000);
  await tab.until("globalThis.__error || globalThis.__shell.probe('state').then(s => s.facts?.startTick === 0)", 60_000);
  const error = await tab.eval<string | null>("globalThis.__error");
  if (error) throw new Error(error);
  const state = await tab.eval<{ facts: { startTick: number; same: boolean; family: string; completions: unknown[] } }>("__shell.probe('state')");
  check(state.facts.startTick === 0 && state.facts.same && state.facts.family.length > 0, "start loaded a ready, stable font at tick 0 outside the locale chain");
  check(state.facts.completions.length === 0, "boot font loading introduced no host-timed completion");
  const latin = "/game/cooked/fonts/latin/NotoSans-Latin.ttf.face.ogb";
  const arabic = "/game/cooked/fonts/arabic/NotoSansArabic-Regular.ttf.face.ogb";
  check(requests.filter((p) => p === latin).length === 1 && !requests.includes(arabic), "the explicit face fetched once; the unrelated pack stayed lazy");
  const latinFile = latin.slice("/game/".length);
  check(manifest.fontPreloads?.length === 1 && manifest.fontPreloads[0]!.path === latinFile && manifest.fontPreloads[0]!.bytes === readFileSync(join(dist, latinFile)).length, "the build manifest lists the explicit face and its actual raw bytes");
  if (publish) {
    const sw = JSON.parse(/^const OG_SW=(.*);$/m.exec(readFileSync(join(dist, "sw.js"), "utf8"))![1]!) as { files: string[]; lazy: string[] };
    check(sw.files.includes(latin.slice("/game/".length)) && sw.lazy.includes(arabic.slice("/game/".length)), "the service worker precaches the explicit face, not the optional one");
    check(manifest.pruning?.omitted.some((o) => o.id === "fonts/latin/extension.json") === true, "the start font works even when publish prunes its pack index");
  }
  // Capture the Worker's rendered target, as reference.ts does, rather
  // than the headless compositor's OffscreenCanvas surface. No tick passes.
  const shot = await tab.eval<{ png: string }>(`(async () => {
    for (let i = 0; i < 240; i++) {
      const shot = await __shell.probe('capture', { width: 1280, height: 480 });
      if (shot) return shot;
      await new Promise(r => setTimeout(r, 16));
    }
    throw new Error('the rendered frame never became available');
  })()`);
  const png = Uint8Array.from(atob(shot.png), (c) => c.charCodeAt(0));
  const image = decodePng(png);
  let ink = 0;
  for (let at = 0; at < image.rgba.length; at += 4) {
    if (image.rgba[at]! > 220 && image.rgba[at + 1]! > 220 && image.rgba[at + 2]! > 220) ink++;
  }
  check(ink > 500, "the rendered frame contains the start font's text ink");
  writeFileSync(join(out, "ready.png"), png);
  // Both loader code and path arrive as input, outside the bundler's graph.
  await tab.eval("__shell.call('command', 'hidden', ['return og.load.font(world, path)', 'fonts/arabic/NotoSansArabic-Regular.ttf']).then(r => { if (!r.ok) throw new Error(r.reason); })");
  const refused = await tab.eval<{ ok: boolean; reason: string }>("__shell.probe('state').then(s => s.facts.hiddenResult)");
  check(!refused.ok && refused.reason.includes("the web cannot prefetch") && refused.reason.includes("publish.keep"), "a runtime-provided loader refuses an undeclared font with a declaration remedy");
  await tab.eval("__shell.call('command', 'arabic', [])");
  await tab.until("__shell.call('frame', 1000 / 60).then(() => __shell.probe('state')).then(s => s.facts.fonts.some(f => f.includes('arabic')))", 30_000);
  check(requests.filter((p) => p === arabic).length === 1, "a later locale switch fetched its optional font pack on demand");
  console.log(`PASS web font start (${publish ? "publish" : "dev"}): ${checks.length} checks`);
  writeFileSync(join(out, "result.json"), JSON.stringify({ ok: true, checks, state, ink, refused, workerLog: await tab.eval("globalThis.__logs"), requests }, null, 2));
} catch (e) {
  if (tab) writeFileSync(join(out, "failed.png"), await tab.screenshot());
  writeFileSync(join(out, "result.json"), JSON.stringify({ ok: false, checks, error: String(e), log, workerLog: tab ? await tab.eval("globalThis.__logs") : [], requests }, null, 2));
  throw e;
} finally {
  await browser.close();
  server.stop(true);
}
