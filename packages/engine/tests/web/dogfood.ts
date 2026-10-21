// The web dogfood round (wave 12): a game published as its author would
// (`og build --publish`, the og binary, the project's own profile), served
// the way the platform opens a release (the top-level page on its play
// origin with `window.__OG_GAME` in its entry and a stand-in play origin at
// /__player/v2/…, the Player protocol 2 HTTP transport, docs/web-export.md
// §The top-level page; the build's own `_headers`, so the page is
// cross-origin isolated and the service worker registers) and played in
// headless Chrome on one tier (WebGPU, or the WebGL2 fallback with
// --webgl, the shell's `?webgl=1`), through real input on the canvas, the
// accessibility mirror (to find what to press) and `__runtime_probe__`.
//
// Per game and tier it records: the published files (raw and brotli -q 11),
// the time from navigation to the first frame (`og:first-frame`) and to the
// title (the first time the title's node is in the accessibility mirror),
// the lanes (`__runtime_probe__.lanes`, the Workers the page runs), the
// played frames over 600 ticks of play (`__og_timings__`: display interval
// and engine work per frame; `state().timings`: step and render, sampled),
// memory at the end (`__runtime_probe__.memory()`, and the
// browser's processes' resident memory), console warnings and errors (the
// page, its Workers, the browser's log), the platform calls the stand-in
// play origin answered (store, leaderboard), a save of the settings with
// the language switched and a reload that must come back in that language
// from the stub's store, the a11y tree on the title, and quit (the game's
// own quit to the title, then Back, which must report `end`).
//
// The game's own playtest frames are a second run (--frames): the frames
// playtest's script (its commands without screenshots, `quit` and dumps)
// through tests/web/reference.ts on the dev export, one tick and one render
// per frame as `og run` does natively, each screenshot compared with the
// game's metal reference under og-tests' rule, and 600 more frames timed
// (`--time 600`). A publish build has no command channel (og-build §6), so
// a frames playtest made of game commands cannot run on the published page
// itself; tests/web/published.ts runs a script of input and capture steps
// there, on its own clock (examples/WEB-FINDINGS.md #6). The served page
// and the mirror-driven input are tests/web/play-origin.ts, shared with it.
//
//   bun tests/web/dogfood.ts --project examples/hollowbell [--webgl] [--frames] [--no-play]
//       [--no-build] [--no-isolation] [--throttle <Mbit/s>] [--boot] [--label <name>]
//       [--out <dir>] [--chrome <path>]
//
// --no-build reuses the last publish build; --no-isolation serves it without
// COOP/COEP (the single-threaded engine); --throttle sends every file
// through one shared link of that throughput (see below); --boot loads the
// page to its gate only (the boot: first frame, title, where the Worker's
// boot went, what was fetched before the first frame, the engine's
// memory) and stops, for before/after measurements; --label names the
// run's directory's suffix (a before and an after side by side). Results go
// to <out>/<id>/<tier>[-single][-<n>mbit][-boot][-<label>]/.
//
// Every file the play origin sends is logged with its time from navigation
// and its brotli size (`requests`), and `boot` sums what the first load
// fetched before its first frame (`og:first-frame`): the boot download a
// player waits for, by kind (engine, shell, basis, cooked, sound chunks),
// with the Worker's own phases (`__runtime_probe__.boot`). The engine's
// memory (`__runtime_probe__.memory()`: the wasm linear memory, the
// counted heap) is read at the boot, after the 600 ticks and at the end.
//
// Writes <out>/<id>/<tier>/dogfood.json and screenshots (default
// dist/dogfood). Never fails on a finding: it records them.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { findInMirror as FIND, keyEvent, MIRROR_TREE as TREE, OBSERVE_MIRROR as OBSERVE, playOrigin, pressAt, publish, publishedFiles, VK } from "./play-origin.ts";
import { ogBin } from "../support/og-bin.ts";
import { compareFrames, decodePng, type Image } from "../support/png.ts";
import { bundleGame } from "../../src/node/build.ts";
import type { PlaytestDesc } from "../../src/playtest.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined>; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const projectRel = opt("--project") ?? "examples/hollowbell";
const project = join(repo, projectRel);
const webgl = args.includes("--webgl");
const tier = webgl ? "webgl2" : "webgpu";
const chrome = chromePath(args);
const id = projectRel.split("/").pop()!;
const work = resolve(opt("--out") ?? join(pkg, "dist", "dogfood"));
// --throttle <Mbit/s>: the server sends every file of the build through one shared link of that
// throughput, 40 ms before each response's first byte (DevTools' network emulation does not reach
// the engine Worker's fetches, which are most of the bytes).
const throttle = Number(opt("--throttle") ?? 0);
const bootOnly = args.includes("--boot");
const label = opt("--label");
const out = join(work, id, `${tier}${args.includes("--no-isolation") ? "-single" : ""}${throttle ? `-${throttle}mbit` : ""}${bootOnly ? "-boot" : ""}${label ? `-${label}` : ""}`);
mkdirSync(out, { recursive: true });
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const td = new TextDecoder();

/** What the run found, in the order found (WEB-FINDINGS reads these). */
const notes: string[] = [];
const note = (s: string) => {
  notes.push(s);
  console.log(`note: ${s}`);
};
const result: Record<string, unknown> = { project: projectRel, tier, notes };
const save = () => writeFileSync(join(out, "dogfood.json"), JSON.stringify(result, null, 1) + "\n");

// ── the game's page plan: what to press, found by the names in its accessibility tree ──
/** A step on the published page: a press found in the mirror, a key, a wait, or a wait for a node. */
type Step =
  | { click: string; nth?: number }
  | { key: string; hold?: number }
  | { wait: number }
  | { until: string; timeout?: number };
interface Plan {
  /** A node (path suffix or name) that says the title is up. */
  readonly title: string;
  /** From the title into play. */
  readonly start: Step[];
  /** Keys held in turn during the 600 timed ticks. */
  readonly play: string[];
  /** From play back to the title (the game's own quit). */
  readonly quit: Step[];
  /** From the title: open the settings, switch the language, save. */
  readonly language: Step[];
  /** A node (path suffix or name) that must be on the title after the reload, in the other language. */
  readonly translated: string;
  /** From the title: a screen that reads the leaderboard, and back. */
  readonly board?: Step[];
}
/** Each game's plan, from its README's controls and its ui.ts ids (the mirror's paths end in them). */
const PLANS: Record<string, Plan> = {
  "lantern-vale": {
    title: "play",
    start: [{ click: "play" }, { wait: 1500 }],
    play: ["KeyW", "KeyD", "KeyS", "KeyA"],
    quit: [{ key: "Escape" }, { wait: 600 }, { click: "quit" }],
    language: [{ click: "settings" }, { wait: 700 }, { click: "简体中文" }, { wait: 400 }, { click: "Done" }],
    translated: "出发",
  },
  hollowbell: {
    title: "new",
    start: [{ click: "new" }, { wait: 700 }, { click: "Begin" }, { wait: 1500 }],
    play: ["ArrowRight", "Space", "ArrowRight", "KeyX"],
    quit: [{ key: "Escape" }, { wait: 600 }, { click: "quit" }],
    language: [{ click: "settings" }, { wait: 700 }, { click: "简体中文" }, { wait: 400 }, { click: "Done" }],
    translated: "新游戏",
  },
  "kestrel-stair": {
    title: "new",
    start: [{ click: "new" }, { wait: 1200 }, { key: "Enter" }, { wait: 400 }, { click: "Skip" }, { wait: 500 }, { click: "Skip" }, { wait: 1500 }],
    play: ["KeyW", "KeyD", "KeyW", "KeyA"],
    quit: [{ key: "Escape" }, { wait: 600 }, { click: "quit" }],
    board: [{ click: "records" }, { wait: 1500 }, { key: "Escape" }, { wait: 800 }],
    language: [{ click: "settings" }, { wait: 900 }, { click: "Language" }, { wait: 500 }, { click: "日本語" }, { wait: 400 }, { click: "Done" }],
    translated: "新しい旅",
  },
  "meridian-loop": {
    title: "start",
    // Time trial, then the three-second countdown.
    start: [{ click: "start" }, { wait: 3600 }],
    play: ["KeyW", "KeyW", "KeyW", "KeyA", "KeyW", "KeyW", "KeyD", "KeyW"],
    // Escape over the racing HUD pauses; Quit to title.
    quit: [{ key: "Escape" }, { wait: 700 }, { click: "quit" }],
    board: [{ click: "board" }, { wait: 1500 }, { key: "Escape" }, { wait: 800 }],
    language: [{ click: "settings" }, { wait: 900 }, { click: "Language" }, { wait: 500 }, { click: "Deutsch" }, { wait: 400 }, { click: "Done" }],
    translated: "Zeitfahren",
  },
  "ui-kit": {
    title: "UI kit",
    start: [{ click: "platformer" }, { wait: 1000 }],
    play: ["ArrowRight", "Space", "ArrowLeft", "Space"],
    // Escape over the platformer HUD opens its pause menu; its Quit returns to the gallery.
    quit: [{ key: "Escape" }, { wait: 800 }, { click: "quit" }],
    // The kit's leaderboard is demo rows (no og.platform call), so no board step.
    language: [{ click: "settings" }, { wait: 900 }, { click: "Language" }, { wait: 500 }, { click: "简体中文" }, { wait: 400 }, { click: "Done" }],
    translated: "简体中文",
  },
};

// ── 1. publish, as the author does ────────────────────────────────────
const og = ogBin();
const dist = join(work, id, "publish");
if (!args.includes("--no-build") || !existsSync(join(dist, "og-build.json"))) {
  const r = publish({ og, project, dist, repo, pkg });
  const said = r.said;
  writeFileSync(join(work, id, "og-build.txt"), said);
  result["build"] = { exit: r.exit, ms: r.ms, warnings: said.split("\n").filter((l) => /warn|warning|missing|single-threaded/i.test(l)) };
  console.log(said);
  if (r.exit !== 0) {
    note(`og build --publish failed (exit ${r.exit})`);
    save();
    process.exit(1);
  }
}
const manifest = JSON.parse(readFileSync(join(dist, "og-build.json"), "utf8")) as Record<string, unknown>;
const listed = publishedFiles(dist);
const sum = (f: (x: { path: string; bytes: number; brotli: number }) => boolean) => listed.filter(f).reduce((a, x) => ({ bytes: a.bytes + x.bytes, brotli: a.brotli + x.brotli }), { bytes: 0, brotli: 0 });
result["package"] = {
  variant: manifest["variant"],
  lanes: manifest["lanes"] ?? null,
  probe: manifest["probe"],
  crossOriginIsolation: manifest["crossOriginIsolation"],
  stream: manifest["stream"] ? (manifest["stream"] as { groups: unknown[] }).groups.length : 0,
  total: sum(() => true),
  engine: sum((x) => /^origin-engine(-lanes)?(_bg\.wasm|\.js)$/.test(x.path)),
  single: sum((x) => /^origin-engine(_bg\.wasm|\.js)$/.test(x.path)),
  lanesModule: sum((x) => /^origin-engine-lanes(_bg\.wasm|\.js)$/.test(x.path) || x.path.startsWith("snippets/")),
  cooked: sum((x) => x.path.startsWith("cooked/")),
  shell: sum((x) => /^(index\.html|play\.js|game\.js|audio-worklet\.js|sw\.js)$/.test(x.path)),
  basis: sum((x) => x.path.startsWith("basis/")),
  modules: listed.filter((x) => !x.path.startsWith("cooked/") || x.bytes > 200_000),
  cookedFiles: listed.filter((x) => x.path.startsWith("cooked/")).length,
};
save();

// ── 2. the stand-in play origin ───────────────────────────────────────
// --no-isolation: a host that sends the files without the build's COOP/COEP (the single-threaded engine).
const origin = playOrigin({ dist, id, isolated: !args.includes("--no-isolation"), throttle, files: listed });
const { calls, reports, missing, store, requests } = origin;
const server = origin;

/**
 * What a load fetched before `firstFrame` ms (from its navigation), by kind, brotli bytes: each
 * file once (a host that lets the browser cache would send the pool Workers' glue once), the
 * service worker's precache apart (`sw`: it starts once the engine is ready, beside the boot).
 */
function bootOf(load: number, firstFrame: number | null) {
  const kind = (p: string) =>
    /^origin-engine(-lanes)?(_bg\.wasm|\.js)$|^snippets\//.test(p) ? "engine"
    : p.startsWith("basis/") ? "basis"
    : /\.chunk\d+\.ogb$/.test(p) ? "chunks"
    : p.startsWith("cooked/stream/") ? "packs"
    : p.startsWith("cooked/") ? "cooked"
    : "shell";
  const once = (rs: typeof requests) => [...new Map(rs.map((r) => [r.path, r])).values()];
  const page = requests.filter((r) => r.load === load && !r.sw);
  const before = once(page.filter((r) => firstFrame === null || r.at <= firstFrame));
  const sw = once(requests.filter((r) => r.load === load && r.sw));
  const by: Record<string, { files: number; brotli: number }> = {};
  for (const r of before) {
    const k = kind(r.path);
    by[k] ??= { files: 0, brotli: 0 };
    by[k].files++;
    by[k].brotli += r.brotli;
  }
  const after = once(page.filter((r) => firstFrame !== null && r.at > firstFrame && !before.some((b) => b.path === r.path)));
  return {
    firstFrame,
    requests: page.filter((r) => firstFrame === null || r.at <= firstFrame).length,
    sw: { files: sw.length, brotli: sw.reduce((n, r) => n + r.brotli, 0), beforeFirstFrame: sw.filter((r) => firstFrame !== null && r.at <= firstFrame).length },
    files: before.length,
    brotli: before.reduce((n, r) => n + r.brotli, 0),
    by,
    after: { files: after.length, brotli: after.reduce((n, r) => n + r.brotli, 0), chunks: after.filter((r) => kind(r.path) === "chunks").length, packs: after.filter((r) => kind(r.path) === "packs").length },
  };
}

// ── 3. play it ────────────────────────────────────────────────────────
const plan = PLANS[id];
async function run(page: Page, steps: readonly Step[], label: string): Promise<boolean> {
  for (const s of steps) {
    if ("wait" in s) await wait(s.wait);
    else if ("key" in s) {
      await keyEvent(page, "keyDown", s.key);
      await wait(s.hold ?? 60);
      await keyEvent(page, "keyUp", s.key);
      await wait(120);
    } else if ("until" in s) {
      const ok = await page.until(`${FIND(s.until)}`, s.timeout ?? 8000).then(() => true, () => false);
      if (!ok) {
        note(`${label}: '${s.until}' never appeared in the accessibility mirror`);
        return false;
      }
    } else {
      const at = await page.until<{ x: number; y: number; path: string } | null>(FIND(s.click, s.nth ?? 0), 8000).catch(() => null);
      if (!at) {
        note(`${label}: nothing named '${s.click}' to press (the mirror held: ${(await page.eval<Array<{ path: string; name: string }>>(TREE)).map((n) => `${n.path}=${n.name}`).join(", ").slice(0, 600)})`);
        return false;
      }
      await pressAt(page, at);
      await wait(350);
    }
  }
  return true;
}

const pct = (xs: number[], q: number) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? +s[Math.min(s.length - 1, Math.floor(q * s.length))]!.toFixed(3) : null;
};
const stats = (xs: number[]) => ({ n: xs.length, mean: xs.length ? +(xs.reduce((a, b) => a + b, 0) / xs.length).toFixed(3) : null, p50: pct(xs, 0.5), p95: pct(xs, 0.95), max: pct(xs, 1) });
/** og-tests' rule (tests/support/png.ts compareFrames): the outliers in percent, and the RMSE. */
function compare(a: Image, b: Image): { outliers: number; rmse: number } {
  const d = compareFrames(a, b);
  return { outliers: +(d.outliers * 100).toFixed(3), rmse: +d.rmse.toFixed(5) };
}

if (!args.includes("--no-play")) {
  const browser = await Browser.launch({ chrome, width: 1280, height: 720, flags: ["--enable-blink-features=ForceEagerMeasureMemory"] });
  const log: string[] = [];
  const workerLog: string[] = [];
  const browserLog: string[] = [];
  const workers = new Map<string, { type: string; url: string }>();
  // The Workers' consoles (the engine Worker, the lanes pool, the service worker).
  browser.on((method, params, session) => {
    if (method === "Target.attachedToTarget") {
      const info = params["targetInfo"] as { type: string; url: string; targetId: string };
      const child = params["sessionId"] as string;
      if (info.type === "worker" || info.type === "service_worker" || info.type === "shared_worker") {
        workers.set(child, { type: info.type, url: info.url });
        void browser.send("Runtime.enable", {}, child).catch(() => {});
        void browser.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: false, flatten: true }, child).catch(() => {});
        void browser.send("Runtime.runIfWaitingForDebugger", {}, child).catch(() => {});
      }
    } else if (session && workers.has(session) && (method === "Runtime.consoleAPICalled" || method === "Runtime.exceptionThrown")) {
      const w = workers.get(session)!;
      if (method === "Runtime.exceptionThrown") workerLog.push(`${w.type} exception: ${(params["exceptionDetails"] as { exception?: { description?: string }; text?: string }).exception?.description ?? (params["exceptionDetails"] as { text?: string }).text}`);
      else {
        const a = (params["args"] as Array<{ value?: unknown; description?: string }>) ?? [];
        workerLog.push(`${w.type} ${params["type"]}: ${a.map((x) => (x.value !== undefined ? String(x.value) : (x.description ?? ""))).join(" ")}`);
      }
    } else if (method === "Log.entryAdded") {
      const e = params["entry"] as { level: string; source: string; text: string; url?: string };
      browserLog.push(`${e.level} ${e.source}: ${e.text}${e.url ? ` (${e.url})` : ""}`);
    }
  });
  const shot = async (page: Page, name: string) => writeFileSync(join(out, `${name}.png`), await page.screenshot());
  /** The engine's own capture at 1280×720 against the game's metal reference of the same screen. */
  const versus = async (page: Page, name: string, reference: string | null) => {
    let c = await page.eval<{ png: string; tick: number; width: number; height: number } | null>("__runtime_probe__.capture(1280, 720)").catch((e) => ({ error: String(e) }) as never);
    for (let i = 0; !c?.png && i < 60; i++) {
      await wait(50);
      c = await page.eval<{ png: string; tick: number; width: number; height: number } | null>("__runtime_probe__.capture(1280, 720)").catch(() => null);
    }
    if (!c?.png) return { error: `capture answered ${JSON.stringify(c)?.slice(0, 200)}` };
    const bytes = Uint8Array.from(atob(c.png), (ch) => ch.charCodeAt(0));
    writeFileSync(join(out, `capture-${name}.png`), bytes);
    const row: Record<string, unknown> = { tick: c.tick, size: `${c.width}x${c.height}` };
    if (reference && existsSync(reference)) Object.assign(row, { against: relative(repo, reference) }, compare(decodePng(bytes), decodePng(new Uint8Array(readFileSync(reference)))));
    return row;
  };
  const metal = (f: string) => {
    for (const d of [join(project, "references", "metal"), join(repo, "crates", "og-ui", "references", "metal")]) if (existsSync(join(d, f))) return join(d, f);
    return null;
  };
  const play: Record<string, unknown> = {};
  result["play"] = play;
  try {
    const page = await browser.open("about:blank", log);
    await page.send("Log.enable");
    await page.send("Page.addScriptToEvaluateOnNewDocument", { source: OBSERVE });
    await page.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
    const url = origin.url(webgl ? "?webgl=1" : "");
    origin.navigated();
    await page.send("Page.navigate", { url });
    await page.until("typeof __runtime_probe__ === 'object' || !document.getElementById('og-error')?.hidden", 120_000);
    const failed = await page.eval<string | null>("document.getElementById('og-error') && !document.getElementById('og-error').hidden ? document.getElementById('og-error').innerText.slice(0, 3000) : null");
    if (failed) {
      note(`the page did not start: ${failed.split("\n").slice(0, 3).join(" / ")}`);
      play["error"] = failed;
      await shot(page, "error");
      throw new Error("no start");
    }
    await page.until("performance.getEntriesByName('og:first-frame').length > 0", 60_000).catch(() => note("no og:first-frame mark within 60 s"));
    await page.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000).catch(() => note("the play gate never showed"));
    const titleNode = plan?.title ?? "";
    if (plan) await page.until(`(() => { const s = __dog.seen; return Object.keys(s).some((k) => k === ${JSON.stringify(titleNode)} || k.endsWith("/" + ${JSON.stringify(titleNode)}) || k === "name:" + ${JSON.stringify(titleNode)}); })()`, 30_000).catch(() => note(`the title's '${titleNode}' never reached the mirror`));
    const marks = await page.eval<Record<string, number>>(`(() => { const m = {}; for (const e of performance.getEntriesByType("mark")) if (e.name.startsWith("og:")) m[e.name] = Math.round(e.startTime); const nav = performance.getEntriesByType("navigation")[0]; m.domContentLoaded = Math.round(nav?.domContentLoadedEventEnd ?? 0); m.load = Math.round(nav?.loadEventEnd ?? 0); return m; })()`);
    const seen = await page.eval<Record<string, number>>("__dog.seen");
    const titleAt = Object.entries(seen).filter(([k]) => k === titleNode || k.endsWith(`/${titleNode}`) || k === `name:${titleNode}`).map(([, v]) => v).sort((a, b) => a - b)[0] ?? null;
    play["timings"] = { ...marks, title: titleAt, firstMirror: Math.min(...Object.values(seen)) };
    play["hello"] = await page.eval("__runtime_probe__.hello()");
    play["lanes"] = await page.eval("__runtime_probe__.lanes");
    play["isolated"] = await page.eval("crossOriginIsolated");
    play["cores"] = await page.eval("navigator.hardwareConcurrency");
    const targets = (await browser.send<{ targetInfos: Array<{ type: string; url: string }> }>("Target.getTargets")).targetInfos;
    const ours = targets.filter((t) => !t.url.startsWith("chrome-extension:"));
    play["workers"] = { dedicated: ours.filter((t) => t.type === "worker").length, service: ours.filter((t) => t.type === "service_worker").length, urls: targets.filter((t) => t.type.endsWith("worker") && !t.url.startsWith("chrome-extension:")).map((t) => t.url.replace(/^.*\/play\/[^/]+\/[^/]+\//, "./").replace(/^blob:.*/, "blob:")) };
    await shot(page, "gate");
    play["boot"] = { ...bootOf(1, marks["og:first-frame"] ?? null), phases: await page.eval("__runtime_probe__.boot ?? null").catch(() => null) };
    play["memoryProbe"] = await page.eval("__runtime_probe__.memory?.() ?? null").catch((e) => ({ error: String(e) }));
    play["titleCapture"] = await versus(page, "title-behind-gate", metal(id === "ui-kit" ? "kit-desktop-gallery.png" : "title.png"));
    if (bootOnly) {
      // The renderer's warm-up behind the gate reaches the Worker's console meanwhile.
      await wait(1500);
      throw new Error("boot only");
    }
    const tree = await page.eval<Array<{ path: string; role: string; name: string; focusable: boolean }>>(TREE);
    play["a11yTitle"] = { nodes: tree.length, roles: Object.fromEntries([...new Set(tree.map((n) => n.role))].map((r) => [r, tree.filter((n) => n.role === r).length])), unnamed: tree.filter((n) => !n.name && n.role !== "group" && n.role !== "generic").map((n) => `${n.role} ${n.path}`), names: tree.map((n) => `${n.role}:${n.name}`).slice(0, 60) };
    const ax = await page.send<{ nodes: Array<{ role?: { value: string }; name?: { value: string }; ignored: boolean }> }>("Accessibility.getFullAXTree").catch(() => ({ nodes: [] }));
    play["chromeAx"] = { nodes: ax.nodes.filter((n) => !n.ignored).length, buttons: ax.nodes.filter((n) => !n.ignored && n.role?.value === "button").map((n) => n.name?.value ?? "") };
    writeFileSync(join(out, "a11y-title.json"), JSON.stringify(tree, null, 1));
    // The gate: a real click.
    await page.click(640, 360);
    await page.until("document.getElementById('og-back')?.hasAttribute('data-playing') || document.getElementById('og-gate')?.hidden", 10_000).catch(() => note("the gate's click did not start play"));
    await wait(1500);
    await shot(page, "title");
    play["titleCaptureStarted"] = await versus(page, "title", metal(id === "ui-kit" ? "kit-desktop-gallery.png" : "title.png"));
    if (!plan) {
      note(`no plan for ${id}: dumped the title's tree only`);
    } else {
      // Start a run.
      const started = await run(page, plan.start, "start");
      await wait(1500);
      await shot(page, "play");
      play["started"] = started;
      play["playCapture"] = await versus(page, "play", null);
      // 600 ticks of play, timed: the frames' work and display interval, and step/render sampled.
      await page.eval("__og_timings__(true)");
      const tick0 = await page.eval<number>("__runtime_probe__.tick()");
      const samples: Array<{ step: number; render: number }> = [];
      const w0 = performance.now();
      let k = 0;
      for (;;) {
        const tick = await page.eval<number>("__runtime_probe__.tick()");
        if (tick - tick0 >= 600 || performance.now() - w0 > 60_000) break;
        const key = plan.play[Math.floor(k / 8) % Math.max(1, plan.play.length)];
        if (key && k % 8 === 0) void page.send("Input.dispatchKeyEvent", { type: "keyDown", code: key, key, windowsVirtualKeyCode: VK[key] ?? 0 });
        if (key && k % 8 === 7) void page.send("Input.dispatchKeyEvent", { type: "keyUp", code: key, key, windowsVirtualKeyCode: VK[key] ?? 0 });
        const s = await page.eval<{ timings: { step: number; render: number } }>("__runtime_probe__.state()");
        samples.push(s.timings);
        k++;
        await wait(100);
      }
      for (const key of plan.play) await page.send("Input.dispatchKeyEvent", { type: "keyUp", code: key, key, windowsVirtualKeyCode: VK[key] ?? 0 });
      const wall = performance.now() - w0;
      const tick1 = await page.eval<number>("__runtime_probe__.tick()");
      const t = await page.eval<{ frames: number; intervals: number[]; work: number[] }>("__og_timings__()");
      play["memoryPlayed"] = await page.eval("__runtime_probe__.memory?.() ?? null").catch((e) => ({ error: String(e) }));
      play["steady"] = {
        ticks: tick1 - tick0,
        wallMs: Math.round(wall),
        ticksPerSecond: +(((tick1 - tick0) / wall) * 1000).toFixed(1),
        frames: t.frames,
        interval: stats(t.intervals),
        work: stats(t.work),
        step: stats(samples.map((s) => s.step)),
        render: stats(samples.map((s) => s.render)),
      };
      play["lanesReport"] = await page.eval("__runtime_probe__.lanes").catch(() => null);
      await shot(page, "played");
      play["facts"] = await page.eval<{ facts: unknown }>("__runtime_probe__.state()").then((s) => s.facts).catch(() => null);
      // The game's own quit, back to the title.
      play["quitToTitle"] = await run(page, plan.quit, "quit");
      await wait(1200);
      await shot(page, "after-quit");
      // The leaderboard, where the game reads one.
      if (plan.board) {
        const before = calls.length;
        play["board"] = await run(page, plan.board, "board");
        await wait(800);
        play["boardCalls"] = calls.slice(before).filter((c) => c.method.startsWith("leaderboard."));
        if ((play["boardCalls"] as unknown[]).length === 0) note("the game's leaderboard screen made no leaderboard call through the Player");
      }
      // A save: the settings with the language switched, kept by the stand-in's store.
      const before = calls.length;
      play["language"] = await run(page, plan.language, "language");
      await wait(1500);
      await shot(page, "language-saved");
      const sets = calls.slice(before).filter((c) => c.method === "store.set");
      play["saveCalls"] = sets;
      if (sets.length === 0) note("switching the language and saving the settings sent no store.set to the Player");
      play["storeAfterSave"] = Object.fromEntries([...store].map(([k2, v]) => [k2, { revision: v.revision, bytes: JSON.stringify(v.value).length }]));
      // Reload: the release's service worker now controls the page; the settings come back from the store.
      const reloadAt = calls.length;
      origin.navigated();
      await page.send("Page.reload", { ignoreCache: false });
      await page.until("typeof __runtime_probe__ === 'object'", 60_000);
      await page.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000).catch(() => {});
      // Behind the gate (the first screen a returning player sees), then after the gate: the
      // Worker applies the store's answers behind the gate (src/web/settle.ts; WEB-FINDINGS #1).
      await wait(1500);
      const behind = await page.eval<Array<{ path: string; role: string; name: string; states: string[] }>>(TREE);
      const translatedBehindGate = (await page.eval(FIND(plan.translated))) !== null;
      await shot(page, "reloaded-gate");
      await page.click(640, 360);
      const translated = await page.until(FIND(plan.translated), 5_000).then(() => true, () => false);
      await wait(1000);
      await shot(page, "reloaded-played");
      play["reload"] = {
        behindGate: behind.map((n) => `${n.role}:${n.name}${n.states.length ? ` [${n.states.join(",")}]` : ""}`).slice(0, 30),
        translatedBehindGate,
        controlled: await page.eval<boolean>("!!navigator.serviceWorker.controller"),
        firstFrame: await page.eval<number | null>("performance.getEntriesByName('og:first-frame')[0]?.startTime ?? null").then((v) => (v === null ? null : Math.round(v))),
        storeReads: calls.slice(reloadAt).filter((c) => c.method === "store.get").map((c) => c.args["key"]),
        translated,
      };
      if (!translatedBehindGate) note(`after the reload the title behind the gate is not in the saved language ('${plan.translated}' missing)${translated ? ", it is once the gate is passed" : ""}`);
      if (!translated) note(`after the reload and the gate the title is not in the saved language ('${plan.translated}' missing)`);
      const tree2 = await page.eval<Array<{ path: string; role: string; name: string }>>(TREE);
      play["a11yReloaded"] = tree2.map((n) => `${n.role}:${n.name}`).slice(0, 40);
      // Memory at the end of the run: the engine's own (__runtime_probe__.memory(): the wasm linear
      // memory, the counted heap, WEB-FINDINGS #9), and the browser processes' resident memory.
      play["memoryEnd"] = await page.eval("__runtime_probe__.memory?.() ?? null").catch((e) => ({ error: String(e) }));
      play["memoryApi"] = await page.eval<string>("typeof performance.measureUserAgentSpecificMemory");
      let heap = 0;
      for (const [session] of workers) {
        const h = await browser.send<{ usedSize: number }>("Runtime.getHeapUsage", {}, session).catch(() => null);
        heap += h?.usedSize ?? 0;
      }
      const pageHeap = await page.send<{ usedSize: number }>("Runtime.getHeapUsage").catch(() => null);
      play["jsHeap"] = { page: pageHeap?.usedSize ?? null, workers: heap };
      // measureUserAgentSpecificMemory never resolves headless (45 s); asked only without the engine's figure.
      play["memory"] = play["memoryEnd"] && !(play["memoryEnd"] as { error?: string }).error ? null : await page
        .eval<{ bytes: number; breakdown: Array<{ bytes: number; types: string[]; attribution: Array<{ scope: string; url: string }> }> }>(
          "Promise.race([performance.measureUserAgentSpecificMemory?.() ?? Promise.resolve(null), new Promise((r) => setTimeout(() => r(null), 45000))])",
        )
        .then((m) => (m ? { bytes: m.bytes, byScope: m.breakdown.reduce((acc: Record<string, number>, b) => { const s = b.attribution[0]?.scope ?? "shared"; acc[s] = (acc[s] ?? 0) + b.bytes; return acc; }, {}) } : null))
        .catch((e) => ({ error: String(e) }));
      const info = await browser.send<{ processInfo: Array<{ type: string; id: number }> }>("SystemInfo.getProcessInfo").catch(() => ({ processInfo: [] }));
      const rss: Record<string, number> = {};
      for (const p of info.processInfo) {
        const r = Bun.spawnSync(["ps", "-o", "rss=", "-p", String(p.id)], { stdout: "pipe", stderr: "pipe" });
        const kb = Number(td.decode(r.stdout).trim());
        if (kb > 0) rss[p.type] = (rss[p.type] ?? 0) + kb * 1024;
      }
      play["processRss"] = rss;
      // Quit: Back to the game's page, which reports `end` (keepalive).
      const reportsBefore = reports.length;
      const back = await page.eval<boolean>("(() => { const b = document.getElementById('og-back'); if (!b || b.hidden) return false; b.click(); return true; })()");
      await page.until(`location.pathname === '/portal/games/${id}'`, 10_000).catch(() => {});
      await wait(800);
      play["quit"] = { back, at: await page.eval<string>("location.pathname"), reports: reports.slice(reportsBefore).map((r) => r.kind) };
      if (!reports.slice(reportsBefore).some((r) => r.kind === "end")) note("Back reached the game's page but no `end` report reached the play origin");
    }
  } catch (e) {
    if (!(bootOnly && e instanceof Error && e.message === "boot only")) play["error"] = play["error"] ?? String(e instanceof Error ? e.stack : e);
    if (play["error"]) console.log(`error: ${play["error"]}`);
  } finally {
    play["calls"] = calls;
    play["requests"] = requests;
    play["reports"] = [...new Set(reports.map((r) => r.kind))];
    play["notFound"] = [...new Set(missing)];
    play["console"] = {
      page: log.filter((l) => /^(warn|warning|error|exception|assert)/.test(l)),
      workers: workerLog.filter((l) => / (warn|warning|error|assert)|exception/.test(l)),
      browser: browserLog.filter((l) => !/^verbose|^info/.test(l)),
      pageAll: log.length,
      workersAll: workerLog.length,
    };
    writeFileSync(join(out, "console.txt"), [...log.map((l) => `page ${l}`), ...workerLog, ...browserLog.map((l) => `browser ${l}`)].join("\n") + "\n");
    await browser.close();
    server.stop();
    save();
  }
}

// ── 4. the playtest's frames on the dev export (tests/web/reference.ts) ──
if (args.includes("--frames")) {
  const playtest = existsSync(join(project, "tests", "frames.og.ts")) ? join(project, "tests", "frames.og.ts") : join(project, "tests", "frames-desktop.og.ts");
  const bundled = join(work, id, ".frames.playtest.js");
  const b = await bundleGame(playtest, bundled, "native");
  if (!b.ok) throw new Error(`${playtest} does not bundle: ${b.reason}`);
  const desc = ((await import(pathToFileURL(bundled).href)) as { default: PlaytestDesc }).default;
  const script: string[] = [];
  const shots: string[] = [];
  const prefix = id === "ui-kit" ? "kit-desktop-" : "";
  for (const step of desc.commands.split(";")) {
    const [at, action, ...rest] = step.split(":");
    if (action === "screenshot") shots.push(`${prefix}${rest[0]!.replace(/\.png$/, "")}@${Math.round(Number(at) * 60)}`);
    else if (action !== "quit" && !action!.startsWith("dump_")) script.push(step);
  }
  // The metal references only (ui-kit's live in crates/og-ui/references, beside its browser tiers').
  const refs = id === "ui-kit" ? join(work, "ui-kit-metal-refs") : join(project, "references");
  if (id === "ui-kit" && !existsSync(join(refs, "metal"))) {
    mkdirSync(refs, { recursive: true });
    Bun.spawnSync(["ln", "-s", join(repo, "crates", "og-ui", "references", "metal"), join(refs, "metal")]);
  }
  const framesOut = join(out, "frames");
  const cmd = ["bun", join(here, "reference.ts"), ...(webgl ? ["--webgl"] : []), "--chrome", chrome, "--project", projectRel, "--references", relative(repo, refs), "--size", desc.size ?? "1280x720", "--script", script.join(";"), "--shots", shots.join(","), "--out", framesOut, "--time", "600"];
  console.log(`---- frames: ${shots.length} shots, ${script.length} commands`);
  const t1 = performance.now();
  const r = Bun.spawnSync(cmd, { cwd: pkg, stdout: "pipe", stderr: "pipe" });
  writeFileSync(join(out, "frames.txt"), td.decode(r.stdout) + td.decode(r.stderr));
  console.log(td.decode(r.stdout).split("\n").filter((l) => /^(ok|FAIL|PASS|frame time)/.test(l)).join("\n"));
  const ref = existsSync(join(framesOut, `reference-${tier}.json`)) ? JSON.parse(readFileSync(join(framesOut, `reference-${tier}.json`), "utf8")) : null;
  result["frames"] = { exit: r.exitCode, ms: Math.round(performance.now() - t1), playtest: relative(repo, playtest), shots: ref?.stations ?? null, frameMs: ref?.frameMs ?? null, renderer: ref?.renderer ?? null, textureFormat: ref?.textureFormat ?? null, error: ref?.error?.split("\n").slice(0, 8).join("\n") ?? null, engineLog: (ref?.engine ?? []).filter((l: string) => /warn|error/i.test(l)).slice(-20) };
  save();
}
{
  // One line per measure, for a before/after table.
  const p = (result["play"] ?? {}) as Record<string, any>;
  const b = p["boot"] as (ReturnType<typeof bootOf> & { phases?: unknown }) | undefined;
  if (b) console.log(`phases: ${JSON.stringify(b.phases ?? null)}`);
  if (b) console.log(`boot: first frame ${b.firstFrame} ms, ready ${p["timings"]?.["og:ready"]} ms, title ${p["timings"]?.["title"]} ms; fetched before it ${b.files} files (${b.requests} requests), ${b.brotli} B brotli (${Object.entries(b.by).map(([k, v]) => `${k} ${v.files}/${v.brotli}`).join(", ")}); after it ${b.after.files} files ${b.after.brotli} B (${b.after.chunks} chunks, ${b.after.packs} packs); the service worker's precache ${b.sw.files} files ${b.sw.brotli} B (${b.sw.beforeFirstFrame} before the first frame)`);
  const mb = (n: unknown) => (typeof n === "number" ? `${(n / 1048576).toFixed(1)} MiB` : "n/a");
  for (const [when, m] of [["boot", p["memoryProbe"]], ["after 600 ticks", p["memoryPlayed"]], ["end", p["memoryEnd"]]] as const) {
    const x = m as { linear?: number; heap?: { live: number; peak: number } | null; world?: { entities: number }; audio?: number | null } | null | undefined;
    if (x && !("error" in x)) console.log(`memory (${when}): linear ${mb(x.linear)}, heap live ${mb(x.heap?.live)} peak ${mb(x.heap?.peak)}, ${x.world?.entities ?? "?"} entities, audio ${mb(x.audio)}`);
  }
  if (p["reload"]) console.log(`reload: translated behind the gate ${p["reload"].translatedBehindGate}, after the tap ${p["reload"].translated}`);
}
console.log(`wrote ${join(out, "dogfood.json")}`);
process.exit(0);
