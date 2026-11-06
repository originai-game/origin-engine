// An exported game's page in Safari in the iOS Simulator, driven by a real
// XCUITest (tests/web/ios/: xcodegen, then xcodebuild test): the runner
// archetype's publish build, cross-origin isolated as its _headers serve it,
// opened in Safari on an iPhone, then touched (tap, double tap, swipes that
// move Safari's toolbar, a pinch), rotated to landscape and back, sent to
// the background and brought back. An instrumentation script injected into
// the page reports its layout (device pixel ratio, the visual viewport, the
// safe-area insets, the canvas's and the shell's boxes) and its events
// (touch, pointer, gesture, resize, visibility, pagehide and pageshow) to
// this harness, which lines them up with the test's steps and checks them.
//
//   bun tests/web/ios-sim.ts [--device <udid>] [--out <dir>]
//
// --device: a booted Simulator (default: the first booted iPhone). Writes
// <out>/ios-sim.json (the checks, every step's layout and events), and the
// Simulator's screenshots as <out>/<step>.png (default dist/test-ios/out).
// Exits 1 when a check fails.
//
// The Simulator's Safari exposes navigator.gpu but finds no WebGPU adapter
// (requestAdapter() answers null on iOS 27.0, macrunner, 2026-11-05), so the
// page shows the shell's "no WebGPU" card and the engine never starts: what
// this judges is the page's layout and behaviour around it, never
// rendering. The Simulator's GPU is the Mac's, so it measures no iPhone
// performance either.

import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { serve } from "./serve.ts";
import { cookProject } from "../support/og-bin.ts";

declare const Bun: {
  serve(o: { port: number; hostname?: string; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string; env?: Record<string, string | undefined> }): { stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number> };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "tests", "fixtures", "browser-host");
const work = join(pkg, "dist", "test-ios");
const out = resolve(opt("--out") ?? join(work, "out"));
const text = (b: Uint8Array) => new TextDecoder().decode(b);

function run(cmd: string[], o: { cwd?: string; env?: Record<string, string | undefined> } = {}): string {
  const r = Bun.spawnSync(cmd, { ...o, stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`${cmd.join(" ")} exited ${r.exitCode}\n${text(r.stderr).slice(-3000)}\n${text(r.stdout).slice(-3000)}`);
  return text(r.stdout);
}

// ── the device ───────────────────────────────────────────────────────────
const devices = JSON.parse(run(["xcrun", "simctl", "list", "devices", "booted", "-j"])) as { devices: Record<string, Array<{ udid: string; name: string; state: string }>> };
const booted = Object.entries(devices.devices).flatMap(([runtime, list]) => list.map((d) => ({ ...d, runtime })));
const device = opt("--device") ? booted.find((d) => d.udid === opt("--device")) : booted.find((d) => /iPhone/i.test(d.name));
if (!device) throw new Error(`no booted iPhone Simulator${opt("--device") ? ` with udid ${opt("--device")}` : ""}: xcrun simctl boot <udid> first`);

// ── the page ─────────────────────────────────────────────────────────────
cookProject(project, repo);
const dist = join(work, "game");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
if (!built.ok) throw new Error(built.reason);

type Report = { at: number; kind: string; [k: string]: unknown };
const reports: Report[] = [];
const keys = new Set<string>();
const marks: Array<{ at: number; step: string; detail: string }> = [];
const shots: string[] = [];
mkdirSync(out, { recursive: true });
const t0 = Date.now();
const ctl = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  async fetch(req) {
    const url = new URL(req.url);
    const cors = { "access-control-allow-origin": "*", "cross-origin-resource-policy": "cross-origin" };
    if (req.method === "OPTIONS") return new Response(null, { headers: { ...cors, "access-control-allow-headers": "content-type" } });
    if (url.pathname === "/report") {
      for (const r of JSON.parse(await req.text()) as Report[]) {
        reports.push({ ...r, at: Date.now() - t0 });
        if (r.kind === "key") keys.add(String(r["key"]));
      }
      return new Response("ok", { headers: cors });
    }
    if (url.pathname === "/has") return new Response(null, { status: keys.has(url.searchParams.get("key") ?? "") ? 200 : 404 });
    if (url.pathname === "/mark") {
      marks.push({ at: Date.now() - t0, step: url.searchParams.get("step") ?? "?", detail: await req.text() });
      return new Response("ok");
    }
    if (url.pathname === "/shot") {
      const name = (url.searchParams.get("name") ?? "shot").replace(/[^a-z0-9-]/gi, "");
      writeFileSync(join(out, `${name}.png`), new Uint8Array(await req.arrayBuffer()));
      shots.push(name);
      return new Response("ok");
    }
    return new Response("not found", { status: 404 });
  },
});

// The instrumentation: the page's layout on every change and its events, posted to the harness.
const probe = `(() => {
  const CTL = ${JSON.stringify(`http://127.0.0.1:${ctl.port}`)};
  const queue = [];
  const flush = (beacon) => {
    if (!queue.length) return;
    const body = JSON.stringify(queue.splice(0));
    if (beacon && navigator.sendBeacon) navigator.sendBeacon(CTL + "/report", body);
    else fetch(CTL + "/report", { method: "POST", body, keepalive: true, mode: "cors" }).catch(() => {});
  };
  setInterval(flush, 150);
  const box = (el) => { if (!el) return null; const r = el.getBoundingClientRect(); return { x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height), hidden: el.hidden || getComputedStyle(el).display === "none" }; };
  let safe;
  const insets = () => {
    if (!safe) { safe = document.createElement("div"); safe.style.cssText = "position:fixed;visibility:hidden;pointer-events:none;left:0;top:0;padding:env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left)"; document.documentElement.appendChild(safe); }
    const s = getComputedStyle(safe);
    return { top: parseFloat(s.paddingTop), right: parseFloat(s.paddingRight), bottom: parseFloat(s.paddingBottom), left: parseFloat(s.paddingLeft) };
  };
  const layout = (why) => {
    const vv = visualViewport;
    const c = document.getElementById("og-canvas");
    const card = document.getElementById("og-error");
    const inner = card && card.firstElementChild;
    queue.push({ kind: "layout", why, dpr: devicePixelRatio, inner: [innerWidth, innerHeight], client: [document.documentElement.clientWidth, document.documentElement.clientHeight],
      vv: vv && { w: vv.width, h: vv.height, top: vv.offsetTop, left: vv.offsetLeft, pageTop: vv.pageTop, scale: vv.scale },
      screen: [screen.width, screen.height], orientation: screen.orientation ? screen.orientation.type : String(window.orientation),
      safe: insets(), scroll: [scrollX, scrollY], visibility: document.visibilityState,
      canvas: c && { ...box(c), width: c.width, height: c.height },
      card: card && { ...box(card), title: card.querySelector("h1") && card.querySelector("h1").textContent, text: card.querySelector("p") && card.querySelector("p").textContent, inner: box(inner) },
      buttons: card ? [...card.querySelectorAll("button")].map((b) => ({ label: b.textContent, ...box(b) })) : [],
      gate: box(document.getElementById("og-gate")), boot: box(document.getElementById("og-boot")), badge: box(document.getElementById("og-made-with")) });
  };
  const key = (k) => queue.push({ kind: "key", key: k });
  const point = (e) => ({ x: Math.round(e.clientX), y: Math.round(e.clientY) });
  const counts = {};
  for (const t of ["touchstart", "touchend", "touchcancel", "pointerdown", "pointerup", "pointercancel", "click", "dblclick", "gesturestart", "gestureend", "contextmenu"]) {
    addEventListener(t, (e) => queue.push({ kind: "event", type: t, pointerType: e.pointerType, touches: e.touches ? e.touches.length : undefined,
      ...(e.touches && e.changedTouches[0] ? point(e.changedTouches[0]) : e.clientX !== undefined ? point(e) : {}), scale: e.scale, target: e.target && (e.target.id || e.target.tagName),
      defaultPrevented: e.defaultPrevented }), { capture: true, passive: true });
  }
  for (const t of ["touchmove", "pointermove", "gesturechange", "wheel", "scroll"]) addEventListener(t, () => (counts[t] = (counts[t] || 0) + 1), { capture: true, passive: true });
  setInterval(() => { if (Object.keys(counts).length) { queue.push({ kind: "moves", ...counts }); for (const k in counts) delete counts[k]; } }, 500);
  for (const t of ["resize", "orientationchange", "focus", "blur", "freeze", "resume"]) addEventListener(t, () => { queue.push({ kind: "event", type: t }); layout(t); });
  visualViewport && visualViewport.addEventListener("resize", () => layout("vv-resize"));
  visualViewport && visualViewport.addEventListener("scroll", () => layout("vv-scroll"));
  document.addEventListener("visibilitychange", () => { queue.push({ kind: "event", type: "visibilitychange", state: document.visibilityState }); layout("visibility"); flush(true); });
  addEventListener("pagehide", (e) => { queue.push({ kind: "event", type: "pagehide", persisted: e.persisted }); flush(true); });
  addEventListener("pageshow", (e) => { queue.push({ kind: "event", type: "pageshow", persisted: e.persisted }); layout("pageshow"); });
  addEventListener("error", (e) => queue.push({ kind: "error", message: e.message }));
  // The shell's final state: the engine's probe, or its card (no WebGPU here).
  (async () => {
    const gpu = navigator.gpu;
    let adapter = null;
    try { adapter = gpu ? await gpu.requestAdapter() : null; } catch (e) { queue.push({ kind: "error", message: String(e) }); }
    queue.push({ kind: "gpu", navigatorGpu: !!gpu, adapter: !!adapter, crossOriginIsolated, userAgent: navigator.userAgent });
    for (let i = 0; i < 600; i++) {
      const card = document.getElementById("og-error");
      if (globalThis.__runtime_probe__ || (card && !card.hidden)) break;
      await new Promise((r) => setTimeout(r, 100));
    }
    layout("ready");
    key("ready");
  })();
})();`;
writeFileSync(join(dist, "ios-probe.js"), probe);
const index = join(dist, "index.html");
writeFileSync(index, readFileSync(index, "utf8").replace("<head>", `<head><script src="ios-probe.js"></script>`));
const game = serve({ dist, isolate: true });
const pageUrl = `http://127.0.0.1:${game.port}/game/index.html`;

// ── the test ─────────────────────────────────────────────────────────────
const xc = join(work, "xcode");
rmSync(xc, { recursive: true, force: true });
cpSync(join(here, "ios"), xc, { recursive: true });
run(["xcodegen", "generate", "--spec", join(xc, "project.yml"), "--project", xc], { cwd: xc });
const result: Record<string, unknown> = { device, page: pageUrl, xcode: run(["xcodebuild", "-version"]).trim().split("\n").join(", ") };
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
const check = (ok: boolean, what: string, detail?: unknown) => {
  checks.push({ check: what, ok, ...(ok ? {} : { detail }) });
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${ok || detail === undefined ? "" : `: ${JSON.stringify(detail).slice(0, 500)}`}`);
};
try {
  const tx = performance.now();
  // Spawned, not spawnSync: the control server above answers the test and the page meanwhile.
  const proc = Bun.spawn(
    ["xcodebuild", "test", "-project", join(xc, "OGSafariSim.xcodeproj"), "-scheme", "SafariUITests", "-destination", `platform=iOS Simulator,id=${device.udid}`,
      "-derivedDataPath", join(work, "derived"), "-resultBundlePath", join(work, `result-${Date.now()}.xcresult`), "CODE_SIGNING_ALLOWED=NO"],
    { cwd: xc, stdout: "pipe", stderr: "pipe", env: { ...process.env, TEST_RUNNER_OG_URL: pageUrl, TEST_RUNNER_OG_CTL: `http://127.0.0.1:${ctl.port}` } },
  );
  const [stdout, stderr, exitCode] = await Promise.all([new Response(proc.stdout).text(), new Response(proc.stderr).text(), proc.exited]);
  const r = { exitCode };
  result["xcodebuildMs"] = Math.round(performance.now() - tx);
  const log = stdout + stderr;
  writeFileSync(join(out, "xcodebuild.log"), log);
  check(r.exitCode === 0, "the XCUITest ran its steps", log.split("\n").filter((l) => /error|failed|Assertion/i.test(l)).slice(-15));
  await new Promise((r) => setTimeout(r, 500));

  // ── the judgement ────────────────────────────────────────────────────
  const stepAt = (step: string) => marks.find((m) => m.step === step)?.at ?? Infinity;
  const between = (a: string, b: string) => reports.filter((r) => r.at >= stepAt(a) && r.at < stepAt(b));
  const last = (kind: string, before: number) => [...reports].reverse().find((r) => r.kind === kind && r.at <= before);
  type Layout = Report & { dpr: number; inner: number[]; vv: { w: number; h: number; scale: number; top: number }; safe: { top: number; right: number; bottom: number; left: number }; canvas: { x: number; y: number; w: number; h: number }; card: { x: number; y: number; w: number; h: number; hidden: boolean; title: string; inner: { x: number; y: number; w: number; h: number } }; buttons: Array<{ label: string; x: number; y: number; w: number; h: number }>; orientation: string; visibility: string };
  const gpu = reports.find((r) => r.kind === "gpu");
  result["gpu"] = gpu;
  check(gpu?.["navigatorGpu"] === true && gpu?.["adapter"] === false, "the Simulator's Safari has navigator.gpu and no adapter (as recorded)", gpu);
  const portrait = last("layout", stepAt("tap")) as Layout | undefined;
  const landscape = last("layout", stepAt("landscape-tap")) as Layout | undefined;
  const again = last("layout", stepAt("background")) as Layout | undefined;
  result["layouts"] = { portrait, landscape, portraitAgain: again };
  check(portrait?.dpr === 3, "portrait: device pixel ratio 3", portrait?.dpr);
  check(!!portrait && portrait.safe.top > 40 && portrait.safe.bottom > 0, "portrait: the notch's safe-area insets reach the page (viewport-fit=cover)", portrait?.safe);
  check(!!portrait && portrait.vv.scale === 1, "portrait: the page is not zoomed", portrait?.vv);
  const inside = (l: Layout | undefined, b: { x: number; y: number; w: number; h: number } | undefined) =>
    !!l && !!b && b.x >= l.safe.left - 1 && b.y >= l.safe.top - 1 && b.x + b.w <= l.vv.w - l.safe.right + 1 && b.y + b.h <= l.vv.h - l.safe.bottom + 1;
  check(!!portrait && portrait.card && !portrait.card.hidden, "portrait: the shell shows its no-WebGPU card", portrait?.card);
  check(inside(portrait, portrait?.card?.inner), "portrait: the card's content is inside the safe area", { card: portrait?.card?.inner, safe: portrait?.safe, vv: portrait?.vv });
  for (const b of portrait?.buttons ?? []) check(inside(portrait, b) && b.h >= 44, `portrait: the card's "${b.label}" button is inside the safe area and at least 44 pt tall`, b);
  check(!!landscape && /landscape/.test(landscape.orientation) && landscape.vv.w > landscape.vv.h, "landscape: the page sees the rotation", landscape && { orientation: landscape.orientation, vv: landscape.vv });
  check(!!landscape && landscape.safe.left + landscape.safe.right > 40, "landscape: the notch's inset moves to the side", landscape?.safe);
  check(inside(landscape, landscape?.card?.inner), "landscape: the card's content is inside the safe area", { card: landscape?.card?.inner, safe: landscape?.safe, vv: landscape?.vv });
  check(!!landscape && landscape.vv.scale === 1, "landscape: the rotation does not zoom the page", landscape?.vv);
  check(!!again && !!portrait && again.vv.w === portrait.vv.w && again.safe.top === portrait.safe.top, "portrait again: the layout returns to portrait's", { before: portrait && { vv: portrait.vv, safe: portrait.safe }, after: again && { vv: again.vv, safe: again.safe } });
  const tap = between("tap", "double-tap").filter((r) => r.kind === "event");
  check(tap.some((e) => e["type"] === "touchstart") && tap.some((e) => e["type"] === "pointerdown" && e["pointerType"] === "touch"), "a tap arrives as touch and pointer events (pointerType touch)", tap.map((e) => `${e["type"]}:${e["pointerType"] ?? ""}`));
  const dbl = between("double-tap", "swipe-up");
  const afterDbl = last("layout", stepAt("swipe-up")) as Layout | undefined;
  check(!!afterDbl && afterDbl.vv.scale === 1, "a double tap does not zoom the page", afterDbl?.vv);
  result["doubleTap"] = dbl.filter((r) => r.kind === "event").map((e) => e["type"]);
  const pinch = last("layout", stepAt("pinch-in")) as Layout | undefined;
  result["pinch"] = { layout: pinch?.vv, events: between("pinch-out", "pinch-in").filter((r) => r.kind === "event").map((e) => `${e["type"]}${e["scale"] ? `:${(e["scale"] as number).toFixed(2)}` : ""}`) };
  check(!!pinch && pinch.vv.scale === 1, "a pinch does not zoom the page (the game owns its gestures)", pinch?.vv);
  const swipe = between("swipe-up", "pinch-out").filter((r) => r.kind === "layout") as Layout[];
  result["toolbar"] = swipe.map((l) => ({ why: l.why, vv: l.vv, inner: l.inner, scroll: l["scroll"] }));
  check(swipe.every((l) => (l["scroll"] as number[])[1] === 0), "swipes do not scroll the page under the canvas", result["toolbar"]);
  const bg = between("background", "done").filter((r) => r.kind === "event").map((e) => `${e["type"]}${e["state"] ? `:${e["state"]}` : ""}${e["persisted"] !== undefined ? `:${e["persisted"]}` : ""}`);
  result["background"] = bg;
  check(bg.includes("visibilitychange:hidden") && bg.includes("visibilitychange:visible") && bg.indexOf("visibilitychange:hidden") < bg.lastIndexOf("visibilitychange:visible"), "backgrounding hides the page and returning shows it (visibilitychange)", bg);
  const back = last("layout", Infinity) as Layout | undefined;
  check(!!back && back.visibility === "visible" && back.vv.w === portrait?.vv.w, "after returning the page is visible with portrait's layout", back && { visibility: back.visibility, vv: back.vv });
  result["errors"] = reports.filter((r) => r.kind === "error");
} catch (e) {
  check(false, "ran to the end", String(e instanceof Error ? e.stack : e));
} finally {
  ctl.stop();
  game.stop();
}
result["marks"] = marks;
result["shots"] = shots;
result["checks"] = checks;
result["reports"] = reports;
writeFileSync(join(out, "ios-sim.json"), JSON.stringify(result, null, 1) + "\n");
const failed = checks.filter((c) => !c.ok).length;
console.log(`${failed === 0 ? "PASS" : "FAIL"} iOS Simulator Safari (${device.name}, ${device.runtime.replace(/.*SimRuntime\./, "")}): ${checks.length - failed} of ${checks.length} checks; ${join(out, "ios-sim.json")}${existsSync(out) ? "" : ""}`);
process.exit(failed === 0 ? 0 : 1);
