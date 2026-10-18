// The Player wire, protocol 2 (origingame/docs/player-protocol.md), from a
// stand-in Player in headless Chrome: a page on the play origin frames the
// runner's publish build at the same origin, as play.<domain>/g/:slug frames a
// release, and speaks protocol 2 to it. Asserts: the game says `hello` with
// its substrate, reports `loading` values that only grow and are byte
// fractions, says `ready` after its first frame; it waits for `start` (its
// own gate stays down under a Player), plays on `start`, holds the clock on
// `pause` and plays again on `resume`; nothing is posted to another origin.
// og.platform through the Player: the runner's first tick asks the store for
// its best score; the shell holds that call until the Player's hello, then
// sends `save.get` as a protocol 2 `call`, and the save's `store.best`
// becomes the game's best (its `state()` facts).
//
//   bun tests/web/player.ts [--out <result.json>] [--chrome <path>]

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
const work = join(pkg, "dist", "test-player");
const outFile = resolve(opt("--out") ?? join(work, "player-result.json"));
const b = await webBuild(project, join(project, "src", "main.ts"), join(work, "publish"), true, settingsOf(project));
if (!b.ok) throw new Error(b.reason);

const player = `<!doctype html><html><head><meta charset="utf-8"></head><body style="margin:0">
<iframe id="g" src="/game/index.html" style="width:800px;height:450px;border:0"></iframe>
<script>
const g = document.getElementById("g");
window.got = [];
addEventListener("message", (e) => {
  if (e.source !== g.contentWindow || e.origin !== location.origin) return;
  const m = e.data;
  if (!m || m.og !== "player" || m.protocol !== 2) return;
  got.push({ ...m, at: Math.round(performance.now()) });
  if (m.kind === "hello") g.contentWindow.postMessage({ og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", capabilities: ["pause", "save.get", "save.set", "leaderboard.submit", "leaderboard.top"], started: false, paused: false }, location.origin);
  if (m.kind === "call") {
    const data = m.method === "save.get" ? { store: { best: { value: 4321, revision: 3 } } } : {};
    g.contentWindow.postMessage({ og: "player", protocol: 2, kind: "result", id: m.id, ok: true, data }, location.origin);
  }
});
window.send = (kind) => g.contentWindow.postMessage({ og: "player", protocol: 2, kind }, location.origin);
</script></body></html>`;

const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
const check = (ok: boolean, what: string, detail?: unknown) => {
  checks.push({ check: what, ok, ...(ok ? {} : { detail }) });
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${ok ? "" : `: ${JSON.stringify(detail).slice(0, 400)}`}`);
};
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const server = serve({ dist: join(work, "publish"), isolate: true, extra: { "/g/runner": player } });
const browser = await Browser.launch({ chrome: chromePath(args) });
let got: Array<{ kind: string; value?: number; substrate?: string; at: number }> = [];
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/g/runner`);
  await page.until("got.some((m) => m.kind === 'ready')", 60_000);
  await wait(800);
  got = await page.eval("got");
  const kinds = got.map((m) => m.kind);
  check(kinds[0] === "hello" && got[0]!.substrate === "origin-engine", "hello first, with substrate origin-engine", got[0]);
  const loading = got.filter((m) => m.kind === "loading").map((m) => m.value!);
  check(loading.length >= 3 && loading.every((v, i) => i === 0 || v > loading[i - 1]!) && loading.every((v) => v >= 0 && v <= 1), "loading values grow and stay in [0, 1]", loading);
  check(kinds.indexOf("ready") > kinds.lastIndexOf("loading"), "ready after the last loading", kinds);
  check(!kinds.includes("fail") && !kinds.includes("crash"), "no fail, no crash", kinds);
  const gate = await page.eval<boolean>("g.contentWindow.document.getElementById('og-gate').hidden");
  check(gate === true, "under a Player the shell draws no gate of its own", gate);
  const t0 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  await wait(400);
  const t1 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t0 === t1, "before start the clock holds", { t0, t1 });
  await page.eval("send('start')");
  await wait(700);
  const t2 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t2 > t1 + 20, "start plays", { t1, t2 });
  await page.eval("send('pause')");
  await wait(200);
  const t3 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  await wait(500);
  const t4 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t4 === t3, "pause holds the clock", { t3, t4 });
  await page.eval("send('resume')");
  await wait(600);
  const t5 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t5 > t4 + 20, "resume plays again", { t4, t5 });
  const calls = (await page.eval<Array<{ kind: string; method?: string; id?: string }>>("got")).filter((m) => m.kind === "call");
  check(calls.some((c) => c.method === "save.get" && typeof c.id === "string"), "og.platform's store.get reached the Player as a save.get call", calls);
  const state = await page.eval<{ facts?: { best?: number } }>("g.contentWindow.__runtime_probe__.state()");
  const best = state.facts?.best;
  check(best === 4321, "the save's store.best is the game's best score", state);
  const sample = (await page.eval<typeof got>("got")).find((m) => m.kind === "sample") as { timings?: Record<string, number>; renderer?: string } | undefined;
  check(!!sample?.timings?.["firstFrame"] && sample.renderer === "webgpu", "a sample with the first-frame time and the renderer", sample);
} finally {
  await browser.close();
  server.stop();
}
const failures = checks.filter((c) => !c.ok).length;
mkdirSync(dirname(outFile), { recursive: true });
writeFileSync(outFile, JSON.stringify({ failures, checks, messages: got }, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} Player protocol 2: ${checks.length} checks, ${failures} failed; ${outFile}`);
process.exit(failures === 0 ? 0 : 1);
