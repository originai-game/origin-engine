// Web cold time to the first frame with Han text, in Chrome on this machine's GPU:
//   bun cold-web.ts <og> <chrome> <project> <out-dir>
// Builds <project> for the web (`og build`, a development build: its probe has `dump`),
// serves it cold (COOP/COEP, no-store), opens it in headless Chrome with docs/ci.md's
// timing switches (WebGPU on Vulkan, or Metal on macOS), passes the play gate, and every 50 ms
// captures the drawn frame: when the first frame landed, when each text line first had
// ink, any line that had ink and lost it, and when each .face.ogb was served.
// Env: FACE_DELAY_MS holds the Han faces (FACE_DELAY_RE, default `SC|ZCOOL`) that long
// before serving them, so a pending face is exercised; SWIFTSHADER=1 puts WebGPU on
// SwiftShader (a functional check only: its times are not counted); LIMIT_MS (90000).
// FACE_DELAY_RE=ZCOOL leaves Noto Sans SC available to test fallback while ZCOOL is pending.
// Prints one JSON line and writes <out-dir>/cold-web.json and frame PNGs.
// Exits 1 on a finding: no first frame, a text line that never draws, a line that goes
// blank after it drew, or (without SWIFTSHADER) a software adapter. 2 on a usage or build error.
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { extname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { decodePng } from "../../packages/engine/src/node/png.ts";

const [og, chrome, projectArg, outArg] = process.argv.slice(2);
if (!og || !chrome || !projectArg || !outArg) {
  console.error("usage: bun cold-web.ts <og> <chrome> <project> <out-dir>");
  process.exit(2);
}
const project = resolve(projectArg), out = resolve(outArg), dist = join(project, "dist");
mkdirSync(out, { recursive: true });
rmSync(dist, { recursive: true, force: true });
const tb = performance.now();
if (Bun.spawnSync([og, "build", project], { stdout: "ignore", stderr: "inherit" }).exitCode !== 0) process.exit(2);
const buildMs = Math.round(performance.now() - tb);
const software = !!process.env.SWIFTSHADER;
const delay = Number(process.env.FACE_DELAY_MS ?? 0);
const delayRe = new RegExp(process.env.FACE_DELAY_RE ?? "SC|ZCOOL");
const limit = Number(process.env.LIMIT_MS ?? 90_000);

const types: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".json": "application/json", ".css": "text/css", ".png": "image/png" };
let t0 = performance.now();
const served: Array<[string, number]> = [];
const server = Bun.serve({
  port: 0,
  idleTimeout: 255,
  async fetch(req) {
    let path = decodeURIComponent(new URL(req.url).pathname);
    if (path.endsWith("/")) path += "index.html";
    const file = join(dist, path);
    const headers = { "cache-control": "no-store", "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "same-origin", "content-type": types[extname(file)] ?? "application/octet-stream" };
    if (!existsSync(file) || statSync(file).isDirectory()) return new Response("not found", { status: 404, headers });
    const face = path.endsWith(".face.ogb");
    if (face && delay && delayRe.test(path)) await Bun.sleep(delay);
    if (face) served.push([path.split("/").pop()!, Math.round(performance.now() - t0)]);
    return new Response(Bun.file(file), { headers });
  },
});
const gpu = software ? ["--use-angle=vulkan", "--use-vulkan=swiftshader", "--enable-features=WebGPU,Vulkan", "--enable-unsafe-swiftshader"] : process.platform === "darwin" ? ["--use-angle=metal"] : ["--use-angle=vulkan", "--enable-features=WebGPU,Vulkan", "--disable-vulkan-surface"];
const profile = mkdtempSync(join(tmpdir(), "og-cold-web-"));
const proc = Bun.spawn([chrome, "--headless=new", "--no-first-run", `--user-data-dir=${profile}`, "--window-size=1280,720", "--hide-scrollbars", "--enable-unsafe-webgpu", ...gpu, "--ignore-gpu-blocklist", "--no-sandbox", "--remote-debugging-port=0", "about:blank"], { stdout: "ignore", stderr: "ignore" });
const portFile = join(profile, "DevToolsActivePort");
while (!existsSync(portFile) || readFileSync(portFile, "utf8").split("\n").length < 2) await Bun.sleep(50);
const port = readFileSync(portFile, "utf8").split("\n")[0];
const targets = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()) as Array<{ type: string; webSocketDebuggerUrl: string }>;
const ws = new WebSocket(targets.find((t) => t.type === "page")!.webSocketDebuggerUrl);
await new Promise((ok) => (ws.onopen = ok));
let id = 0;
const waiting = new Map<number, (v: any) => void>();
const consoleLines: string[] = [];
ws.onmessage = (m) => {
  const d = JSON.parse(String(m.data));
  if (d.id && waiting.has(d.id)) waiting.get(d.id)!(d);
  if (d.method === "Target.attachedToTarget") ws.send(JSON.stringify({ id: 1_000_000 + ++id, method: "Runtime.enable", sessionId: d.params.sessionId }));
  if (d.method === "Runtime.consoleAPICalled") consoleLines.push(`${Math.round(performance.now() - t0)} ${d.params.args.map((a: any) => a.value ?? a.description ?? "").join(" ")}`);
};
const send = (method: string, params: object = {}) => new Promise<any>((ok) => { const i = ++id; waiting.set(i, ok); ws.send(JSON.stringify({ id: i, method, params })); });
// A readback on a slow adapter takes seconds; a call that never answers is given up.
const ev = async (expr: string) => (await Promise.race([send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true }), Bun.sleep(30_000).then(() => null)]))?.result?.result?.value;
await send("Runtime.enable");
await send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: false, flatten: true });
await send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
await send("Network.enable");
await send("Network.setCacheDisabled", { cacheDisabled: true });
t0 = performance.now();
await send("Page.navigate", { url: `http://127.0.0.1:${server.port}/` });

let tapped = false;
let boxes: Array<[string, number, number, number, number]> | null = null;
const firstInk = new Map<string, number>();
const lost: string[] = [];
const log: unknown[] = [];
let lastFaces = 0, stableSince = 0, saved = 0;
while (performance.now() - t0 < limit) {
  await Bun.sleep(50);
  const ms = Math.round(performance.now() - t0);
  if (!(await ev(`typeof globalThis.__runtime_probe__ === "object" && (performance.getEntriesByName("og:first-frame")[0]?.startTime ?? 0) > 0`))) continue;
  if (!tapped) {
    const r = await ev(`(() => { const b = document.querySelector("#og-gate button"); if (!b) return null; const q = b.getBoundingClientRect(); return [q.x + q.width / 2, q.y + q.height / 2]; })()`);
    if (r) for (const type of ["mousePressed", "mouseReleased"]) await send("Input.dispatchMouseEvent", { type, x: r[0], y: r[1], button: "left", clickCount: 1 });
    tapped = true;
  }
  if (!boxes) {
    const dump = await ev(`__runtime_probe__.dump({ what: "ui" }).then(d => JSON.stringify(d), e => "ERR " + e)`);
    if (typeof dump === "string" && !dump.startsWith("ERR")) {
      const found: Array<[string, number, number, number, number]> = [];
      for (const e of (JSON.parse(dump).elements ?? []) as Array<{ text?: string; textLines?: number[][] }>) for (const l of e.textLines ?? []) found.push([e.text ?? "", l[0]!, l[1]!, l[2]!, l[3]!]);
      if (found.length) boxes = found;
    }
  }
  const shot = await ev(`__runtime_probe__.capture(1280, 720, 0).then(s => s && s.png ? s.png : null, () => null)`);
  if (!shot || !boxes) continue;
  const bytes = Uint8Array.from(atob(shot), (c) => c.charCodeAt(0));
  const img = decodePng(bytes);
  const at = (x: number, y: number) => (Math.min(img.height - 1, Math.max(0, Math.round(y))) * img.width + Math.min(img.width - 1, Math.max(0, Math.round(x)))) * 4;
  const ink: Record<string, number> = {};
  for (const [text, x, y, w, h] of boxes) {
    const bg = at(x - 3, y + h / 2);
    let n = 0;
    for (let py = y; py < y + h; py++)
      for (let px = x; px < x + w; px++) {
        const i = at(px, py);
        if (Math.abs(img.rgba[i]! - img.rgba[bg]!) + Math.abs(img.rgba[i + 1]! - img.rgba[bg + 1]!) + Math.abs(img.rgba[i + 2]! - img.rgba[bg + 2]!) > 120) n++;
      }
    ink[text] = (ink[text] ?? 0) + n;
  }
  for (const [text, n] of Object.entries(ink)) {
    if (n > 0 && !firstInk.has(text)) firstInk.set(text, ms);
    if (n === 0 && firstInk.has(text)) lost.push(`${ms} ms: ${text}`);
  }
  log.push({ ms, ink, faces: served.length });
  if (saved < 4 || served.length !== lastFaces) {
    writeFileSync(join(out, `frame-${String(ms).padStart(6, "0")}.png`), bytes);
    saved++;
  }
  if (served.length !== lastFaces) stableSince = ms;
  lastFaces = served.length;
  if (firstInk.size === new Set(boxes.map((b) => b[0])).size && ms - stableSince > 5_000) break;
}
const marks = Object.fromEntries(((await ev(`performance.getEntriesByType("mark").map(m => [m.name, Math.round(m.startTime)])`)) ?? []) as Array<[string, number]>);
const adapter = (await ev(`(async () => { try { await __runtime_probe__.perf("start"); await new Promise(r => setTimeout(r, 500)); return (await __runtime_probe__.perf("report")).json?.adapter ?? null; } catch (e) { return null; } })()`)) as string | null;
const texts = [...new Set((boxes ?? []).map((b) => b[0]))];
const result = { buildMs, adapter, firstFrameMs: marks["og:first-frame"] ?? null, firstInk: Object.fromEntries(firstInk), lost, faces: served, gate: consoleLines.filter((l) => /first frame/.test(l)) };
writeFileSync(join(out, "cold-web.json"), JSON.stringify({ ...result, log }, null, 1));
console.log(JSON.stringify(result));
proc.kill();
server.stop();
rmSync(profile, { recursive: true, force: true });
const findings: string[] = [];
if (!result.firstFrameMs) findings.push(`cold-web: no first frame in ${limit} ms`);
if (!texts.length) findings.push(`cold-web: no text lines discovered in ${limit} ms`);
if (!firstInk.size) findings.push(`cold-web: no text ink captured in ${limit} ms`);
for (const t of texts) if (!firstInk.has(t)) findings.push(`cold-web: "${t}" never drew`);
for (const l of lost) findings.push(`cold-web: a line went blank after it drew, at ${l}`);
if (!software && (!adapter || /software|swiftshader/i.test(adapter))) findings.push(`cold-web: the adapter is ${adapter ?? "unknown"}, not a GPU`);
for (const f of findings) console.error(f);
process.exit(findings.length ? 1 : 0);
