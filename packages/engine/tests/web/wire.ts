// The og-probe wire (origingame/origin/packages/contracts/src/probe-wire.ts)
// of an exported game, driven from a parent window at another origin in
// headless Chrome, the way the platform's Stage and Player reach it:
// parent http://127.0.0.1:<port>/parent.html frames
// http://localhost:<port>/game/index.html, the page told the parent's origin
// (`window.__OG_STAGE_HOST__`, as the Stage proxy injects it).
//
// For the runner archetype's dev build (every member) and publish build (the
// player members), each cross-origin isolated (the request ring) and not
// (postMessage), it asserts: the unsolicited hello; hello, tick, state,
// entities, entity, capture (full and fitted) and record answers in the
// contract's shapes; the clock held behind the play gate and released by a
// real tap; real keyboard input reaching the engine; dev-only pause, advance,
// bounds, patch (and their `unsupported` in a publish build); `refused`
// and `unsupported` errors; and silence for another wire version, for a page
// told another origin and for a page told none.
//
//   bun tests/web/wire.ts [--out <transcript.json>] [--chrome <path>]
//
// Exits 1 on the first failed assertion and prints it. Writes the transcript
// (every call and answer, PNGs elided) to --out (default
// dist/test-wire/transcript.json).

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { webBuild, settingsOf } from "../../src/node/web-build.ts";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { serve } from "./serve.ts";
import { ogBin } from "../support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "archetypes", "runner");
const work = join(pkg, "dist", "test-wire");
const outFile = resolve(opt("--out") ?? join(work, "transcript.json"));
const chrome = chromePath(args);

type Answer = { kind?: string; result?: unknown; code?: string; message?: string; silent?: true };
const transcript: Array<Record<string, unknown>> = [];
let failures = 0;
function check(ok: boolean, what: string, detail?: unknown): void {
  transcript.push({ check: what, ok, ...(ok ? {} : { detail }) });
  if (!ok) {
    failures++;
    console.error(`FAIL ${what}${detail === undefined ? "" : `: ${JSON.stringify(detail).slice(0, 600)}`}`);
  } else console.log(`ok   ${what}`);
}
const keys = (v: unknown) => (v && typeof v === "object" ? Object.keys(v as object).sort().join(",") : String(v));
const isPng = (b64: unknown) => typeof b64 === "string" && atob(b64.slice(0, 12)).startsWith("\x89PNG");
const pngSize = (b64: string) => {
  const bin = atob(b64.slice(0, 44));
  const u = (i: number) => ((bin.charCodeAt(i) << 24) | (bin.charCodeAt(i + 1) << 16) | (bin.charCodeAt(i + 2) << 8) | bin.charCodeAt(i + 3)) >>> 0;
  return { width: u(16), height: u(20) };
};

// ── the builds ─────────────────────────────────────────────────────────
if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dists: Record<string, string> = {};
for (const kind of ["dev", "publish"] as const) {
  const out = join(work, kind);
  const b = await webBuild(project, join(project, "src", "main.ts"), out, kind === "publish", settingsOf(project));
  if (!b.ok) throw new Error(b.reason);
  dists[kind] = out;
}

const parentHtml = (gameOrigin: string) => `<!doctype html><html><head><meta charset="utf-8"><title>wire host</title></head>
<body style="margin:0;background:#222">
<iframe id="g" allow="cross-origin-isolated; autoplay; fullscreen" src="${gameOrigin}/game/index.html" style="position:absolute;left:0;top:0;width:800px;height:450px;border:0"></iframe>
<script>
const origin = ${JSON.stringify(gameOrigin)};
const g = document.getElementById("g");
const hellos = [];
const pending = new Map();
let n = 0;
let foreign = 0;
addEventListener("message", (e) => {
  if (e.source !== g.contentWindow || e.origin !== origin) { foreign++; return; }
  const m = e.data;
  if (!m || m.og !== "probe" || m.v !== 1) return;
  if (m.kind === "hello") { hellos.push(m); return; }
  const p = pending.get(m.id);
  if (p) { pending.delete(m.id); p(m); }
});
window.wire = (method, params = {}, v = 1, wait = 8000) => new Promise((resolve) => {
  const id = "c" + (++n);
  const t0 = performance.now();
  pending.set(id, (m) => resolve({ ...m, ms: +(performance.now() - t0).toFixed(1) }));
  g.contentWindow.postMessage({ og: "probe", v, kind: "call", id, method, params }, origin);
  setTimeout(() => { if (pending.has(id)) { pending.delete(id); resolve({ silent: true }); } }, wait);
});
window.hellos = hellos;
window.foreign = () => foreign;
</script></body></html>`;

async function run(kind: "dev" | "publish", isolate: boolean, browser: Browser): Promise<void> {
  const label = `${kind}${isolate ? " isolated" : ""}`;
  let told: string | null = null;
  // One server, two host names: the parent at 127.0.0.1, the game at localhost (another origin).
  const pages: Record<string, string> = {};
  const live = serve({ dist: dists[kind]!, isolate, told: () => told, extra: pages });
  pages["/parent.html"] = parentHtml(`http://localhost:${live.port}`);
  told = `http://127.0.0.1:${live.port}`;
  const log: string[] = [];
  const page: Page = await browser.open(`http://127.0.0.1:${live.port}/parent.html`, log);
  const call = (method: string, params: Record<string, unknown> = {}, v = 1, wait?: number) =>
    page.eval<Answer & { ms?: number }>(`wire(${JSON.stringify(method)}, ${JSON.stringify(params)}, ${v}${wait ? `, ${wait}` : ""})`);
  const record = (method: string, params: unknown, a: Answer) => {
    const r = JSON.parse(JSON.stringify(a)) as Record<string, unknown>;
    const res = r["result"] as Record<string, unknown> | undefined;
    if (res && typeof res["png"] === "string") res["png"] = `<${(res["png"] as string).length} base64 chars>`;
    if (res && typeof res["base64"] === "string") res["base64"] = `<${(res["base64"] as string).length} base64 chars>`;
    transcript.push({ run: label, method, params, answer: r });
  };
  const c = async (method: string, params: Record<string, unknown> = {}) => {
    const a = await call(method, params);
    record(method, params, a);
    return a;
  };
  try {
    // The page says hello on its own once the engine is ready.
    const hello = await page.until<{ protocol: number; substrate: string }>("hellos.find((h) => h.protocol === 5)", 60_000);
    check(hello.protocol === 5 && hello.substrate === "origin-engine", `${label}: unsolicited hello`, hello);
    const iso = await page.evalChild<boolean>("localhost", "crossOriginIsolated");
    check(iso === isolate, `${label}: crossOriginIsolated is ${isolate}`, iso);
    const transport = await page.evalChild<string>("localhost", "document.querySelector('canvas') && 'ok'");
    check(transport === "ok", `${label}: the game page has its canvas`);

    const h = await c("hello");
    check(h.kind === "result" && keys(h.result) === "protocol,substrate" && (h.result as { protocol: number }).protocol === 5, `${label}: hello answers {protocol: 5, substrate}`, h);
    const t1 = await c("tick");
    check(h.kind === "result" && typeof t1.result === "number", `${label}: tick answers a number`, t1);
    const st = await c("state");
    const sr = st.result as { tick: number; paused: boolean; timings: { frameMs: unknown } };
    check(
      st.kind === "result" && keys(sr) === "paused,tick,timings" && keys(sr.timings) === "frameMs" && typeof sr.paused === "boolean" && (sr.timings.frameMs === null || typeof sr.timings.frameMs === "number"),
      `${label}: state answers {tick, paused, timings: {frameMs}}`,
      st,
    );
    // Behind the play gate the clock holds.
    await new Promise((r) => setTimeout(r, 400));
    const t2 = await c("tick");
    check(t2.result === t1.result, `${label}: the clock holds behind the play gate`, { before: t1.result, after: t2.result });
    // A real tap on the gate (the middle of the frame) starts it.
    await page.tap(400, 225);
    await new Promise((r) => setTimeout(r, 600));
    const t3 = await c("tick");
    check(typeof t3.result === "number" && t3.result > (t2.result as number), `${label}: a tap on the gate starts the clock`, { before: t2.result, after: t3.result });

    // Real keyboard input reaches the engine (the input ring when isolated, messages otherwise).
    if (kind === "dev") {
      const before = await page.evalChild<number>("localhost", "__runtime_probe__.accepted()");
      await page.key("Space", " ");
      await page.key("ArrowUp", "ArrowUp");
      await new Promise((r) => setTimeout(r, 300));
      const after = await page.evalChild<number>("localhost", "__runtime_probe__.accepted()");
      check(after - before >= 4, `${label}: real key events reach the engine's input layer`, { before, after });
    }
    const inp = await c("input", { event: { kind: "key", key: "Space", action: "press" } });
    check(inp.kind === "error" && inp.code === "unsupported", `${label}: input is not a wire method (unsupported)`, inp);

    const full = await c("capture");
    const fr = full.result as { tick: number; width: number; height: number; png: string };
    const canvas = await page.evalChild<{ w: number; h: number }>("localhost", "({ w: document.querySelector('canvas').width, h: document.querySelector('canvas').height })");
    check(full.kind === "result" && keys(fr) === "height,png,tick,width" && isPng(fr.png) && fr.width === canvas.w && fr.height === canvas.h, `${label}: capture answers the full frame as base64 PNG`, { got: { w: fr?.width, h: fr?.height }, canvas });
    if (fr?.png) check(pngSize(fr.png).width === fr.width, `${label}: the PNG is the size it claims`);
    const fit = await c("capture", { width: 320 });
    const ft = fit.result as { width: number; height: number; png: string };
    check(fit.kind === "result" && ft.width === 320 && Math.abs(ft.height - Math.round((320 * canvas.h) / canvas.w)) <= 1 && isPng(ft.png), `${label}: capture {width: 320} fits the frame`, { got: ft && { w: ft.width, h: ft.height } });

    const ents = await c("entities");
    const items = (ents.result as { items: Array<{ id: string }> }).items;
    check(ents.kind === "result" && Array.isArray(items) && items.some((i) => i.id === "runner"), `${label}: entities lists the runner`, ents);
    const ent = await c("entity", { id: "runner" });
    const ev = ent.result as { id: string; params: Record<string, unknown> };
    check(ent.kind === "result" && ev.id === "runner" && typeof ev.params["Tuning.gravity"] === "number", `${label}: entity answers {id, params}`, ent);
    const nobody = await c("entity", { id: "nobody" });
    check(nobody.kind === "error" && nobody.code === "refused", `${label}: entity of an unknown id is refused`, nobody);

    if (kind === "dev") {
      const b = await c("bounds", { id: "runner" });
      const g = (b.result as { items: Array<Record<string, unknown>> }).items.find((i) => i["parameter"] === "Tuning.gravity");
      check(b.kind === "result" && g?.["min"] === 10 && g?.["max"] === 80 && typeof g?.["value"] === "number", `${label}: bounds answers {items} with the declared range and value`, g);
      const pt = await c("patch", { id: "runner", parameter: "Tuning.gravity", value: 40 });
      const pr = pt.result as { applied: boolean; roundTripMs: number };
      check(pt.kind === "result" && pr.applied === true && typeof pr.roundTripMs === "number", `${label}: patch answers {applied, roundTripMs}`, pt);
      const after = await c("entity", { id: "runner" });
      check((after.result as { params: Record<string, number> }).params["Tuning.gravity"] === 40, `${label}: the patch applied`, after);
      const out = await c("patch", { id: "runner", parameter: "Tuning.gravity", value: 500 });
      check(out.kind === "error" && out.code === "refused", `${label}: a patch outside the range is refused`, out);
      const ps = await c("pause");
      check(ps.kind === "result" && (ps.result as { paused: boolean }).paused === true, `${label}: pause answers the state, paused`, ps);
      const t4 = (await c("tick")).result as number;
      await new Promise((r) => setTimeout(r, 300));
      const t5 = (await c("tick")).result as number;
      check(t5 === t4, `${label}: paused, the clock holds`, { t4, t5 });
      const adv = await c("advance", { steps: 10 });
      check(adv.kind === "result" && (adv.result as { tick: number }).tick === t5 + 10, `${label}: advance {steps: 10} takes 10 ticks`, { t5, adv });
      const rs = await c("resume");
      check(rs.kind === "result" && (rs.result as { paused: boolean }).paused === false, `${label}: resume answers the state, playing`, rs);
    } else {
      for (const [m, p] of [["pause", {}], ["resume", {}], ["advance", { steps: 1 }], ["bounds", { id: "runner" }], ["patch", { id: "runner", parameter: "Tuning.gravity", value: 40 }]] as const) {
        const a = await c(m, p);
        check(a.kind === "error" && a.code === "unsupported", `${label}: ${m} is unsupported in a publish build`, a);
      }
    }
    const bad = await c("advance", { steps: -1 });
    check(bad.kind === "error" && bad.code === "refused", `${label}: advance {steps: -1} is refused`, bad);
    const unknown = await c("frobnicate");
    check(unknown.kind === "error" && unknown.code === "unsupported", `${label}: an unknown method is unsupported`, unknown);
    const v2 = await call("tick", {}, 2, 1500);
    check(v2.silent === true, `${label}: another wire version gets no answer`, v2);
    const rec = await c("record", { seconds: 1 });
    const rr = rec.result as { mime: string; base64: string; durationMs: number } | undefined;
    check(
      (rec.kind === "result" && typeof rr?.mime === "string" && (rr?.base64.length ?? 0) > 100 && (rr?.durationMs ?? 0) >= 900) || (rec.kind === "error" && rec.code === "unsupported"),
      `${label}: record {seconds: 1} answers a recording (or unsupported)`,
      rec.kind === "error" ? rec : { mime: rr?.mime, bytes: rr?.base64.length, durationMs: rr?.durationMs },
    );
    if (kind === "dev" && !isolate) transcript.push({ run: label, recordKind: rec.kind, recordMime: rr?.mime ?? rec.code });
    const tooLong = await c("record", { seconds: 16 });
    check(tooLong.kind === "error" && tooLong.code === "refused", `${label}: record {seconds: 16} is refused`, tooLong);

    // Told another origin, the page answers nothing and says no hello.
    told = "http://example.invalid";
    const helloCount = await page.eval<number>("hellos.length");
    await page.send("Page.reload");
    await new Promise((r) => setTimeout(r, 300));
    await page.until("document.getElementById('g') && window.wire", 20_000);
    await page.until<boolean>("(async () => true)()", 5_000);
    await new Promise((r) => setTimeout(r, 4000));
    const silentHello = await call("hello", {}, 1, 2000);
    check(silentHello.silent === true, `${label}: a page told another origin answers nothing`, silentHello);
    check((await page.eval<number>("hellos.length")) === 0, `${label}: and posts no hello`, { before: helloCount });
    told = null;
    await page.send("Page.reload");
    await page.until("window.wire", 20_000);
    await new Promise((r) => setTimeout(r, 4000));
    const none = await call("tick", {}, 1, 2000);
    check(none.silent === true && (await page.eval<number>("hellos.length")) === 0, `${label}: a page told no origin is silent`, none);
  } catch (e) {
    check(false, `${label}: ran to the end`, String(e instanceof Error ? e.stack : e));
    console.error(log.slice(-30).join("\n"));
  } finally {
    live.stop();
    await page.send("Page.close").catch(() => {});
  }
}

const browser = await Browser.launch({ chrome });
try {
  for (const kind of ["dev", "publish"] as const) for (const isolate of [true, false]) await run(kind, isolate, browser);
} finally {
  await browser.close();
}
mkdirSync(dirname(outFile), { recursive: true });
writeFileSync(outFile, JSON.stringify({ chrome, failures, transcript }, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} og-probe wire: ${transcript.filter((t) => "check" in t).length} checks, ${failures} failed; transcript ${outFile}`);
process.exit(failures === 0 ? 0 : 1);
