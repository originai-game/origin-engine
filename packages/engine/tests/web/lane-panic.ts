// A Rust panic inside a lane, in headless Chrome: the page must show the
// crash, not hang (docs/web-export.md §Errors). The lane-panic fixture
// (2,304 boxes, so every frame's CPU frustum cull splits across the lanes
// pool; extraction reads og.Mesh from its columns and never splits) is
// built with `og build --publish`'s web half and served with its COOP/COEP,
// so the page boots the lanes variant; after a tap on the gate the test asks
// the engine Worker (over the DevTools protocol, the Worker's own `fault`
// call: og_web_lanes_fault) to make the next cull item panic. That
// item runs on a pool Worker, whose wasm aborts there; the engine thread,
// waiting at the lane boundary (og-lanes), is woken by the recorded panic
// and panics in turn, so the engine Worker traps and reports a structured
// crash. Checked: the shell's crash reaches the page within
// CRASH_DEADLINE_MS of the fault, `__runtime_probe__.error` is
// `{ kind: "panic", lane: "cull", tick, message, location }`, and the
// error card shows the panic's line. The same build served without
// isolation runs single-threaded: the same fault panics on the engine
// thread and is reported with `lane: null`.
//
//   bun tests/web/lane-panic.ts [--out <dir>] [--chrome <path>]
//
// Needs dist/web-publish with the lanes variant (scripts/build.sh). Writes
// <out>/lane-panic.json and a screenshot of each card (default
// dist/test-lane-panic/out). Exits 1 when a check fails.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { serve } from "./serve.ts";
import { ogBin } from "../support/og-bin.ts";
import { FRAME_DEADLINE_MS } from "../../src/web/shell.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
};

/** How long after the fault the page may take to show the crash. */
export const CRASH_DEADLINE_MS = 5_000;

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const work = join(pkg, "dist", "test-lane-panic");
const out = resolve(opt("--out") ?? join(work, "out"));
mkdirSync(out, { recursive: true });
const chrome = chromePath(args);

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(ok && detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 600)}` : ""}`);
}
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

const project = join(pkg, "tests", "fixtures", "lane-panic");
if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dist = join(work, "build");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
check(built.manifest?.lanes !== null && built.manifest?.variant === "3d", "the publish build ships the 3D variant and the lanes variant", { variant: built.manifest?.variant, lanes: built.manifest?.lanes });

type HostError = { kind: string; message: string; tick: number | null; lane: string | null; location: string | null };
type Lanes = { on: boolean; threads: number; why: string };

/** The engine Worker's DevTools session (a dedicated Worker of the page, auto-attached). */
function workerSession(browser: Browser, page: Page): Promise<string> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("the engine Worker never attached")), 60_000);
    const off = browser.on((method, params, session) => {
      if (session !== page.session || method !== "Target.attachedToTarget") return;
      const info = params["targetInfo"] as { type: string; url: string };
      if (info.type === "worker" && /\/game\.js/.test(info.url)) {
        clearTimeout(timer);
        off();
        resolve(params["sessionId"] as string);
      }
    });
  });
}

const results: Record<string, unknown> = {};
const browser = await Browser.launch({ chrome });
try {
  for (const isolate of [true, false]) {
    const label = isolate ? "isolated (lanes)" : "not isolated (single-threaded)";
    const server = serve({ dist, isolate });
    const log: string[] = [];
    const page = await browser.open("about:blank", log);
    try {
      const engineWorker = workerSession(browser, page);
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
      await page.until("window.__runtime_probe__", 60_000);
      const lanes = await page.eval<Lanes>("__runtime_probe__.lanes");
      check(isolate ? lanes.on && lanes.threads > 1 : !lanes.on, `${label}: lanes ${isolate ? `on, ${lanes.threads} Workers` : "off"}`, lanes);
      await page.until("performance.getEntriesByName('og:first-frame')[0]?.startTime", 30_000);
      await page.tap(640, 360);
      await page.until("(async () => (await __runtime_probe__.tick()) > 30)()", 30_000);
      check((await page.eval<unknown>("__runtime_probe__.error")) === null, `${label}: no error while it plays`);
      const session = await engineWorker;
      // The Worker's own `fault` call, as the shell would send it.
      const armed = await browser.send<{ result: { value?: unknown } }>(
        "Runtime.evaluate",
        { expression: `self.onmessage({ data: { type: "call", id: 1e9, method: "fault", args: ["cull"] } }), "sent"`, returnByValue: true },
        session,
      );
      const t0 = performance.now();
      check(armed.result.value === "sent", `${label}: the fault is armed in the engine Worker`, armed);
      let error: HostError | null = null;
      // Past the shell's frame deadline too, so a Worker that hangs instead is seen as the hang it is.
      while (!error && performance.now() - t0 < FRAME_DEADLINE_MS + 5_000) {
        error = await page.eval<HostError | null>("__runtime_probe__.error").catch(() => null);
        if (!error) await wait(20);
      }
      const ms = Math.round(performance.now() - t0);
      check(error !== null && ms <= CRASH_DEADLINE_MS, `${label}: the page has the crash ${ms} ms after the fault (deadline ${CRASH_DEADLINE_MS} ms)`, { error, ms });
      check(error?.kind === "panic", `${label}: __runtime_probe__.error is a panic`, error);
      check(/a fault injected at the cull site/.test(error?.message ?? ""), `${label}: with the panic's message`, error?.message);
      check(typeof error?.tick === "number" && error.tick > 30, `${label}: and the tick it happened at`, error?.tick);
      check(/og-lanes\/src\/lib\.rs:\d+:\d+/.test(error?.location ?? ""), `${label}: and where`, error?.location);
      check(isolate ? error?.lane === "cull" : error?.lane === null, `${label}: ${isolate ? "inside a lane (cull)" : "on the engine thread (lane null)"}`, error?.lane);
      const card = await page
        .until<{ title: string; text: string }>("(() => { const c = document.getElementById('og-error'); return c && !c.hidden && { title: c.querySelector('h1').textContent, text: c.querySelector('p').textContent }; })()", 5_000)
        .catch(() => null);
      check(!!card && /panic at tick \d+/.test(card.text) && (isolate ? /in a lane \(cull\)/.test(card.text) : !/in a lane/.test(card.text)), `${label}: the error card shows the panic`, card);
      const shot = `lane-panic-${isolate ? "lanes" : "single"}.png`;
      writeFileSync(join(out, shot), await page.screenshot());
      // A later probe call is refused at once, not left waiting.
      const t1 = performance.now();
      const after = await page.eval<string>("__runtime_probe__.tick().then(() => 'answered', (e) => 'refused: ' + e.message)");
      check(after.startsWith("refused") && performance.now() - t1 < 2_000, `${label}: a probe call after the crash is refused at once`, after);
      results[label] = { lanes, error, ms, card, shot, log: log.slice(-12) };
    } catch (e) {
      check(false, `${label}: runs`, { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
  }
} finally {
  await browser.close();
}
writeFileSync(join(out, "lane-panic.json"), JSON.stringify({ deadlineMs: CRASH_DEADLINE_MS, results, checks }, null, 2) + "\n");
console.log(`${checks.length - failures} of ${checks.length} checks passed (${join(out, "lane-panic.json")})`);
process.exit(failures === 0 ? 0 : 1);
