// `og run --device`: the publish build played in a real browser, timed.
//
//   bun src/node/device.ts --dist <publish build> --project <root> --device <target> --out <dir>
//       [--seconds N (default 5)] [--port P] [--headed] [--webgl] [--chrome <path>]
//       [--cert <pem> --key <pem>] [--size WxH] [--json]
//   bun src/node/device.ts --preflight --device <target> [options]   (og runs it before the build:
//       the options and the target's tools only)
//
// The build (og build --publish, which og made) is served on 0.0.0.0 with its
// `_headers` and COOP/COEP/CORP, and the LAN URL is printed. `<target>`:
//
//   chrome            this machine's Chrome (--chrome, $CHROME, $CHROME_PATH, the
//                     agent-browser install, /Applications, PATH), headless unless
//                     --headed, over the DevTools protocol at http://127.0.0.1:<port>/
//   android[:serial]  Chrome on a phone over adb: `adb reverse` serves the page at the
//                     phone's own localhost (a secure context, so isolation holds) and
//                     `adb forward ... localabstract:chrome_devtools_remote` reaches
//                     its DevTools; the same driver, with touch
//   iphone[:udid]     Safari on an iPhone over WebDriver (safaridriver, macOS), at the
//   ios[:udid]        LAN URL; isolation and WebGPU need a secure context there, so
//                     pass --cert/--key (mkcert) to serve HTTPS
//
// The page is opened cold, the play gate passed with a real tap or click,
// and the engine's own telemetry read: `og:first-frame` (ms from navigation
// to the first frame drawn behind the gate), `__runtime_probe__.hello()`
// (the tier), and after a second of warm-up `__og_timings__` (the Worker's
// played frames: display interval and engine work) over --seconds. Writes
// <out>/device.json and <out>/report.md in og polish's shape (scores,
// findings with a fix each, then the measures) and prints one line (or one
// JSON line with --json). A missing prerequisite exits 1 with one line
// naming the fix; a page that fails (error card, no first frame, the gate
// not passing) exits 1 with what it saw.

import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { networkInterfaces, tmpdir } from "node:os";

declare module "node:fs" {
  export function mkdtempSync(prefix: string): string;
  export function readdirSync(path: string): string[];
}
declare module "node:path" {
  export function normalize(path: string): string;
}
import { normalize } from "node:path";

interface Proc {
  kill(): void;
  readonly exited: Promise<number>;
}
declare const Bun: {
  serve(o: {
    port: number;
    hostname: string;
    tls?: { cert: string; key: string };
    fetch(req: Request): Response | Promise<Response>;
  }): { port: number; stop(force?: boolean): void };
  file(path: string): Blob & { exists(): Promise<boolean> };
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): Proc;
  spawnSync(cmd: string[], o?: { stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  which(cmd: string): string | null;
};

const argv = process.argv.slice(2);
/** The last occurrence wins, so a caller can override a forwarded option. */
const opt = (name: string) => {
  const i = argv.lastIndexOf(name);
  return i >= 0 ? argv[i + 1] : undefined;
};
const flag = (name: string) => argv.includes(name);

/** A missing prerequisite or a failed page: one line, exit 1. */
class Refusal extends Error {}
const refuse = (line: string): never => {
  throw new Refusal(line);
};

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
const decode = (b: Uint8Array) => new TextDecoder().decode(b);

// ── serving ───────────────────────────────────────────────────────────

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".png": "image/png",
  ".ogb": "application/octet-stream",
};

const ISOLATION: Record<string, string> = {
  "cross-origin-opener-policy": "same-origin",
  "cross-origin-embedder-policy": "require-corp",
  "cross-origin-resource-policy": "cross-origin",
};

/** The build's `_headers` (Netlify/Cloudflare Pages format) as pattern → headers. */
function readHeaders(dist: string): Array<{ pattern: string; headers: Record<string, string> }> {
  const file = join(dist, "_headers");
  if (!existsSync(file)) return [];
  const rules: Array<{ pattern: string; headers: Record<string, string> }> = [];
  for (const line of readFileSync(file, "utf8").split("\n")) {
    if (!line.trim()) continue;
    if (!/^\s/.test(line)) rules.push({ pattern: line.trim(), headers: {} });
    else if (rules.length) {
      const [k, ...v] = line.trim().split(":");
      rules[rules.length - 1]!.headers[k!.trim().toLowerCase()] = v.join(":").trim();
    }
  }
  return rules;
}

const matches = (pattern: string, path: string) =>
  pattern === path || (pattern.endsWith("*") && path.startsWith(pattern.slice(0, -1))) || (pattern.startsWith("/*.") && path.endsWith(pattern.slice(2)));

interface Served {
  readonly port: number;
  readonly scheme: "http" | "https";
  /** The headers sent for the page and the wasm, as the device received them. */
  readonly sent: Record<string, Record<string, string>>;
  /** Paths answered, in order (the first load's). */
  readonly paths: string[];
  stop(): void;
}

function serveBuild(dist: string, port: number, tls: { cert: string; key: string } | null): Served {
  const rules = readHeaders(dist);
  const sent: Record<string, Record<string, string>> = {};
  const paths: string[] = [];
  const server = Bun.serve({
    port,
    hostname: "0.0.0.0",
    ...(tls ? { tls } : {}),
    async fetch(req) {
      const url = new URL(req.url);
      let rel = normalize(decodeURIComponent(url.pathname));
      if (rel.endsWith("/")) rel += "index.html";
      const path = join(dist, rel);
      if (paths.length < 400) paths.push(url.pathname);
      if (!path.startsWith(dist) || !(await Bun.file(path).exists())) return new Response("not found", { status: 404, headers: { "cache-control": "no-store", ...ISOLATION } });
      const headers: Record<string, string> = {};
      for (const r of rules) if (matches(r.pattern, rel)) Object.assign(headers, r.headers);
      Object.assign(headers, ISOLATION);
      // Every load is a cold one: the device's HTTP cache never answers.
      headers["cache-control"] = "no-store";
      headers["content-type"] = TYPES[rel.slice(rel.lastIndexOf("."))] ?? "application/octet-stream";
      if (rel === "/index.html" || rel.endsWith(".wasm")) sent[rel === "/index.html" ? "/" : rel] = { ...headers };
      return new Response(Bun.file(path), { headers });
    },
  });
  return { port: server.port, scheme: tls ? "https" : "http", sent, paths, stop: () => server.stop(true) };
}

/** This machine's LAN IPv4 addresses (what a phone on the same network opens). */
function lanAddresses(): string[] {
  const out: string[] = [];
  for (const list of Object.values(networkInterfaces())) for (const a of list ?? []) if (!a.internal && (a.family === "IPv4" || a.family === 4)) out.push(a.address);
  return out;
}

// ── drivers ───────────────────────────────────────────────────────────

/** What the measurement needs of a browser: open a page, evaluate, tap at a point or an element, close. */
interface Driver {
  readonly name: string;
  open(url: string): Promise<void>;
  /** An expression's value (Promises awaited), by value. */
  eval<T>(expression: string): Promise<T>;
  /** A real tap (or click) on the element `selector` matches. */
  tap(selector: string): Promise<void>;
  /** Console errors and uncaught exceptions seen so far (CDP only). */
  errors(): string[];
  close(): Promise<void>;
}

/** A DevTools protocol client on a browser endpoint, with one page attached (flattened sessions). */
class Cdp implements Driver {
  readonly name: string;
  readonly #ws: WebSocket;
  readonly #pending = new Map<number, { resolve(v: unknown): void; reject(e: Error): void }>();
  readonly #errors: string[] = [];
  readonly #touch: boolean;
  readonly #onClose: () => Promise<void>;
  #next = 1;
  #session: string | null = null;
  #target: string | null = null;

  private constructor(name: string, ws: WebSocket, touch: boolean, onClose: () => Promise<void>) {
    this.name = name;
    this.#ws = ws;
    this.#touch = touch;
    this.#onClose = onClose;
    ws.addEventListener("message", (e) => {
      const m = JSON.parse(String((e as MessageEvent).data)) as { id?: number; result?: unknown; error?: { message: string }; method?: string; params?: Record<string, unknown>; sessionId?: string };
      if (m.id !== undefined) {
        const p = this.#pending.get(m.id);
        this.#pending.delete(m.id);
        if (m.error) p?.reject(new Error(m.error.message));
        else p?.resolve(m.result);
        return;
      }
      if (m.sessionId !== this.#session) return;
      if (m.method === "Runtime.consoleAPICalled" && m.params?.["type"] === "error") {
        const args = (m.params["args"] as Array<{ value?: unknown; description?: string }>) ?? [];
        this.#errors.push(args.map((a) => (a.value !== undefined ? String(a.value) : (a.description ?? ""))).join(" ").slice(0, 400));
      } else if (m.method === "Runtime.exceptionThrown") {
        const d = m.params?.["exceptionDetails"] as { text?: string; exception?: { description?: string } } | undefined;
        this.#errors.push((d?.exception?.description ?? d?.text ?? "exception").split("\n")[0]!.slice(0, 400));
      }
    });
  }

  static async connect(name: string, url: string, touch: boolean, onClose: () => Promise<void>): Promise<Cdp> {
    const ws = new WebSocket(url);
    await new Promise<void>((ok, fail) => {
      ws.addEventListener("open", () => ok());
      ws.addEventListener("error", () => fail(new Error(`could not connect to DevTools at ${url}`)));
    });
    return new Cdp(name, ws, touch, onClose);
  }

  send<T = unknown>(method: string, params: Record<string, unknown> = {}, session: string | null = this.#session): Promise<T> {
    const id = this.#next++;
    this.#ws.send(JSON.stringify({ id, method, params, ...(session ? { sessionId: session } : {}) }));
    return new Promise<T>((resolve, reject) => this.#pending.set(id, { resolve: resolve as (v: unknown) => void, reject }));
  }

  async open(url: string): Promise<void> {
    const { targetId } = await this.send<{ targetId: string }>("Target.createTarget", { url: "about:blank" }, null);
    this.#target = targetId;
    const { sessionId } = await this.send<{ sessionId: string }>("Target.attachToTarget", { targetId, flatten: true }, null);
    this.#session = sessionId;
    await this.send("Page.enable");
    await this.send("Runtime.enable");
    await this.send("Page.bringToFront");
    await this.send("Page.navigate", { url });
  }

  async eval<T>(expression: string): Promise<T> {
    const r = await this.send<{ result: { value?: T }; exceptionDetails?: { text: string; exception?: { description?: string } } }>("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
      userGesture: false,
    });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value as T;
  }

  async tap(selector: string): Promise<void> {
    const at = await this.eval<{ x: number; y: number } | null>(
      `(() => { const e = document.querySelector(${JSON.stringify(selector)}); if (!e) return null; const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`,
    );
    if (!at) refuse(`the page has no ${selector} to tap`);
    const { x, y } = at!;
    if (this.#touch) {
      await this.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x, y }] });
      await sleep(60);
      await this.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
    } else {
      await this.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
      await this.send("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: "left", buttons: 1, clickCount: 1 });
      await sleep(40);
      await this.send("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: "left", buttons: 0, clickCount: 1 });
    }
  }

  errors(): string[] {
    return [...this.#errors];
  }

  async close(): Promise<void> {
    if (this.#target) await this.send("Target.closeTarget", { targetId: this.#target }, null).catch(() => {});
    this.#ws.close();
    await this.#onClose();
  }
}

/** The local Chrome: --chrome, $CHROME, $CHROME_PATH, the agent-browser install, /Applications, PATH. */
function findChrome(): string {
  const given = opt("--chrome") ?? process.env.CHROME ?? process.env.CHROME_PATH;
  if (given) {
    if (!existsSync(given)) refuse(`no Chrome at ${given}: point --chrome (or CHROME) at the browser's executable`);
    return given;
  }
  const home = process.env.HOME ?? "";
  const candidates: string[] = [];
  const installs = join(home, ".agent-browser", "browsers");
  if (existsSync(installs))
    for (const d of readdirSync(installs).filter((n) => n.startsWith("chrome-")).sort().reverse())
      candidates.push(
        process.platform === "darwin"
          ? join(installs, d, "Google Chrome for Testing.app", "Contents", "MacOS", "Google Chrome for Testing")
          : join(installs, d, "chrome"),
      );
  if (process.platform === "darwin")
    candidates.push(
      "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
      "/Applications/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
      "/Applications/Chromium.app/Contents/MacOS/Chromium",
    );
  for (const c of candidates) if (existsSync(c)) return c;
  for (const name of ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"]) {
    const p = Bun.which(name);
    if (p) return p;
  }
  return refuse("no Chrome found: install Google Chrome, or pass --chrome <path> (or set CHROME)");
}

async function launchChrome(width: number, height: number): Promise<Driver> {
  const chrome = findChrome();
  const profile = mkdtempSync(join(tmpdir(), "og-device-"));
  const proc = Bun.spawn(
    [
      chrome,
      ...(flag("--headed") ? [] : ["--headless=new"]),
      "--no-first-run",
      "--no-default-browser-check",
      `--user-data-dir=${profile}`,
      `--window-size=${width},${height}`,
      "--hide-scrollbars",
      ...(process.platform === "darwin" ? ["--use-angle=metal"] : []),
      "--enable-unsafe-webgpu",
      "--enable-features=WebGPU",
      "--ignore-gpu-blocklist",
      "--enable-unsafe-swiftshader",
      "--autoplay-policy=user-gesture-required",
      "--remote-debugging-port=0",
      "about:blank",
    ],
    { stdout: "ignore", stderr: "ignore" },
  );
  const portFile = join(profile, "DevToolsActivePort");
  const t0 = performance.now();
  while (!existsSync(portFile) || readFileSync(portFile, "utf8").split("\n").length < 2) {
    if (performance.now() - t0 > 20_000) {
      proc.kill();
      refuse(`${chrome} did not open its DevTools port in 20 s: is it a Chrome (or Chromium) executable?`);
    }
    await sleep(50);
  }
  const [port, path] = readFileSync(portFile, "utf8").trim().split("\n");
  return Cdp.connect(`chrome (${basename(chrome)}${flag("--headed") ? ", headed" : ", headless"})`, `ws://127.0.0.1:${port}${path}`, false, async () => {
    proc.kill();
    await proc.exited;
    rmSync(profile, { recursive: true, force: true });
  });
}

const NO_ADB = "no adb on PATH: install Android platform-tools (`brew install android-platform-tools`, or the SDK's platform-tools)";
const NO_SAFARIDRIVER = "og run --device iphone needs macOS with safaridriver (it ships with Safari; run `safaridriver --enable` once)";

function adb(serial: string | null, ...args: string[]): string {
  const r = Bun.spawnSync(["adb", ...(serial ? ["-s", serial] : []), ...args], { stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) refuse(`adb ${args.join(" ")} failed: ${decode(r.stderr).trim().split("\n")[0]} (is the phone unlocked and USB debugging allowed?)`);
  return decode(r.stdout).trim();
}

/** Chrome on an Android phone: the page at its own localhost (adb reverse), DevTools forwarded to this machine. */
async function connectAndroid(wanted: string | null, port: number): Promise<{ driver: Driver; url: string }> {
  if (!Bun.which("adb")) refuse(NO_ADB);
  const devices = adb(null, "devices")
    .split("\n")
    .slice(1)
    .map((l) => l.trim().split(/\s+/))
    .filter((p) => p[1] === "device")
    .map((p) => p[0]!);
  if (devices.length === 0) refuse("no Android device: connect one with USB debugging on (Settings > Developer options) and accept the prompt; `adb devices` lists it");
  if (wanted && !devices.includes(wanted)) refuse(`no Android device ${wanted}: adb lists ${devices.join(", ")}`);
  if (!wanted && devices.length > 1) refuse(`several Android devices (${devices.join(", ")}): pass --device android:<serial>`);
  const serial = wanted ?? devices[0]!;
  adb(serial, "reverse", `tcp:${port}`, `tcp:${port}`);
  adb(serial, "shell", "am", "start", "-n", "com.android.chrome/com.google.android.apps.chrome.Main", "-d", "about:blank");
  const forwarded = Number(adb(serial, "forward", "tcp:0", "localabstract:chrome_devtools_remote"));
  let ws: string | null = null;
  for (let i = 0; i < 50 && !ws; i++) {
    try {
      ws = ((await (await fetch(`http://127.0.0.1:${forwarded}/json/version`)).json()) as { webSocketDebuggerUrl?: string }).webSocketDebuggerUrl ?? null;
    } catch {
      await sleep(200);
    }
  }
  const cleanup = async () => {
    Bun.spawnSync(["adb", "-s", serial, "forward", "--remove", `tcp:${forwarded}`]);
    Bun.spawnSync(["adb", "-s", serial, "reverse", "--remove", `tcp:${port}`]);
  };
  if (!ws) {
    await cleanup();
    refuse(`Chrome on ${serial} does not answer DevTools: open Chrome on the phone once, and allow USB debugging`);
  }
  const model = adb(serial, "shell", "getprop", "ro.product.model");
  const driver = await Cdp.connect(`android ${model} (${serial})`, ws!.replace(/^ws:\/\/[^/]+/, `ws://127.0.0.1:${forwarded}`), true, cleanup);
  return { driver, url: `http://localhost:${port}/` };
}

const ELEMENT = "element-6066-11e4-a52e-4f735466cecf";
const IPHONE_FIX =
  "on the iPhone turn on Settings > Apps > Safari > Advanced > Web Inspector and Remote Automation; on the Mac run `safaridriver --enable` once and pair the phone in Xcode (Window > Devices and Simulators)";

/** Safari on an iPhone over WebDriver (safaridriver). */
async function connectIphone(udid: string | null): Promise<Driver> {
  if (process.platform !== "darwin" || !Bun.which("safaridriver")) refuse(NO_SAFARIDRIVER);
  const probe = Bun.serve({ port: 0, hostname: "127.0.0.1", fetch: () => new Response("") });
  const port = probe.port;
  probe.stop(true);
  const proc = Bun.spawn(["safaridriver", "-p", String(port)], { stdout: "ignore", stderr: "ignore" });
  const base = `http://127.0.0.1:${port}`;
  let up = false;
  for (let i = 0; i < 50 && !up; i++) {
    try {
      up = (await fetch(`${base}/status`)).ok;
    } catch {
      await sleep(100);
    }
  }
  if (!up) {
    proc.kill();
    refuse(`safaridriver did not start: ${IPHONE_FIX}`);
  }
  const call = async <T>(method: string, path: string, body?: unknown): Promise<T> => {
    const r = await fetch(`${base}${path}`, { method, headers: { "content-type": "application/json" }, ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
    const j = (await r.json()) as { value: T & { error?: string; message?: string } };
    if (!r.ok) throw new Error(`${j.value.error ?? r.status}: ${j.value.message ?? ""}`);
    return j.value;
  };
  let session: string;
  try {
    const s = await call<{ sessionId: string }>("POST", "/session", {
      capabilities: { alwaysMatch: { browserName: "safari", platformName: "iOS", ...(udid ? { "safari:deviceUDID": udid } : {}) } },
    });
    session = s.sessionId;
  } catch (e) {
    proc.kill();
    return refuse(`safaridriver could not open Safari on the iPhone (${(e as Error).message.split("\n")[0]}): ${IPHONE_FIX}`);
  }
  await call("POST", `/session/${session}/timeouts`, { script: 120_000, pageLoad: 120_000 });
  return {
    name: `iphone${udid ? ` (${udid})` : ""}`,
    open: async (url) => void (await call("POST", `/session/${session}/url`, { url })),
    eval: async <T>(expression: string) => {
      const r = await call<{ ok?: T; err?: string }>("POST", `/session/${session}/execute/async`, {
        script: `const done = arguments[arguments.length - 1]; Promise.resolve().then(() => (${expression})).then((v) => done({ ok: v }), (e) => done({ err: String(e && e.stack || e) }));`,
        args: [],
      });
      if (r.err !== undefined) throw new Error(r.err);
      return r.ok as T;
    },
    tap: async (selector) => {
      const el = await call<Record<string, string>>("POST", `/session/${session}/element`, { using: "css selector", value: selector });
      await call("POST", `/session/${session}/element/${el[ELEMENT]}/click`, {});
    },
    errors: () => [],
    close: async () => {
      await call("DELETE", `/session/${session}`).catch(() => {});
      proc.kill();
      await proc.exited;
    },
  };
}

// ── measuring ─────────────────────────────────────────────────────────

interface Stats {
  count: number;
  mean: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
}

/** Nearest-rank percentiles, ms to 0.01. */
function stats(xs: readonly number[]): Stats {
  const s = [...xs].sort((a, b) => a - b);
  const r = (x: number) => Math.round(x * 100) / 100;
  const at = (p: number) => r(s[Math.min(s.length - 1, Math.max(0, Math.ceil((p / 100) * s.length) - 1))] ?? 0);
  return { count: s.length, mean: r(s.reduce((a, b) => a + b, 0) / Math.max(1, s.length)), p50: at(50), p95: at(95), p99: at(99), max: r(s[s.length - 1] ?? 0) };
}

interface DeviceInfo {
  userAgent: string;
  platform: string;
  mobile: boolean;
  dpr: number;
  viewport: { width: number; height: number };
  screen: { width: number; height: number };
  canvas: { width: number; height: number } | null;
  hardwareConcurrency: number | null;
  deviceMemory: number | null;
  crossOriginIsolated: boolean;
  isSecureContext: boolean;
  gpu: { vendor: string; architecture: string; description: string } | null;
  webgl: { vendor: string; renderer: string } | null;
}

const DEVICE_INFO = `(async () => {
  const nav = navigator;
  let gpu = null;
  try {
    const a = nav.gpu ? await nav.gpu.requestAdapter() : null;
    const i = a ? (a.info || (a.requestAdapterInfo ? await a.requestAdapterInfo() : null)) : null;
    if (i) gpu = { vendor: i.vendor || "", architecture: i.architecture || "", description: i.description || "" };
  } catch {}
  let webgl = null;
  try {
    const gl = document.createElement("canvas").getContext("webgl2");
    if (gl) {
      const x = gl.getExtension("WEBGL_debug_renderer_info");
      webgl = { vendor: String(gl.getParameter(x ? x.UNMASKED_VENDOR_WEBGL : gl.VENDOR)), renderer: String(gl.getParameter(x ? x.UNMASKED_RENDERER_WEBGL : gl.RENDERER)) };
      const lose = gl.getExtension("WEBGL_lose_context");
      if (lose) lose.loseContext();
    }
  } catch {}
  const c = document.querySelector("canvas");
  const ua = nav.userAgentData;
  return {
    userAgent: nav.userAgent,
    platform: (ua && ua.platform) || nav.platform || "",
    mobile: ua ? !!ua.mobile : /Mobi|iPhone|Android/.test(nav.userAgent),
    dpr: devicePixelRatio,
    viewport: { width: innerWidth, height: innerHeight },
    screen: { width: screen.width, height: screen.height },
    canvas: c ? { width: Math.round(c.clientWidth * devicePixelRatio), height: Math.round(c.clientHeight * devicePixelRatio) } : null,
    hardwareConcurrency: nav.hardwareConcurrency || null,
    deviceMemory: nav.deviceMemory || null,
    crossOriginIsolated: !!globalThis.crossOriginIsolated,
    isSecureContext: !!globalThis.isSecureContext,
    gpu,
    webgl,
  };
})()`;

/** Poll `expression` until it is truthy (its value) or `ms` pass (null). */
async function until<T>(d: Driver, expression: string, ms: number): Promise<T | null> {
  const t0 = performance.now();
  for (;;) {
    let v: T | null = null;
    try {
      v = await d.eval<T>(expression);
    } catch {
      // The page is navigating; ask again.
    }
    if (v) return v;
    if (performance.now() - t0 > ms) return null;
    await sleep(100);
  }
}

const mark = (name: string) => `(performance.getEntriesByName(${JSON.stringify(name)})[0] || {}).startTime || 0`;
const FAILED = `(() => { const c = document.getElementById("og-error"); return c && !c.hidden ? (c.textContent || "").trim().replace(/\\s+/g, " ").slice(0, 1500) : ""; })()`;

interface Timings {
  frames: number;
  intervals: number[];
  work: number[];
}

interface Measured {
  firstFrameMs: number;
  readyMs: number;
  startMs: number;
  tier: string;
  seconds: number;
  frames: number;
  ticks: number;
  interval: Stats;
  work: Stats;
  device: DeviceInfo;
  errors: string[];
}

async function measure(d: Driver, url: string, seconds: number): Promise<Measured> {
  await d.open(url);
  const failed = async () => (await d.eval<string>(FAILED).catch(() => "")) || "";
  const probed = await until<boolean>(d, `typeof __runtime_probe__ === "object" && typeof __og_timings__ === "function" && (${mark("og:first-frame")}) > 0 || !!(${FAILED})`, 90_000);
  const card = await failed();
  if (card) refuse(`the page failed: ${card}`);
  if (!probed) refuse(`no first frame in 90 s at ${url}${d.errors().length ? `: ${d.errors()[0]}` : ""}`);
  const firstFrameMs = Math.round(await d.eval<number>(mark("og:first-frame")));
  const readyMs = Math.round(await d.eval<number>(mark("og:ready")));
  const hello = await d.eval<{ tier?: string }>("__runtime_probe__.hello()");
  // The play gate: a real tap passes it (a project with `gate: none` has started already).
  const gated = await until<string>(d, `(${mark("og:start")}) > 0 ? "started" : (document.getElementById("og-gate") && !document.getElementById("og-gate").hidden ? "gate" : "")`, 30_000);
  if (!gated) refuse("the play gate never showed and play never started");
  if (gated === "gate") {
    await d.tap("#og-gate button");
    if (!(await until<number>(d, mark("og:start"), 10_000))) refuse(`a tap on the play gate did not start play${(await failed()) ? `: ${await failed()}` : ""}`);
  }
  const startMs = Math.round(await d.eval<number>(mark("og:start")));
  await sleep(1000);
  await d.eval("__og_timings__(true)");
  const t0 = await d.eval<number>("__runtime_probe__.tick()");
  const began = performance.now();
  await sleep(seconds * 1000);
  const t = await d.eval<Timings>("__og_timings__()");
  const t1 = await d.eval<number>("__runtime_probe__.tick()");
  const elapsed = (performance.now() - began) / 1000;
  const after = await failed();
  if (after) refuse(`the page failed while playing: ${after}`);
  if (t.intervals.length === 0) refuse(`no frame played in ${seconds} s after the gate (the Worker's clock did not run)`);
  const device = await d.eval<DeviceInfo>(DEVICE_INFO);
  return {
    firstFrameMs,
    readyMs,
    startMs,
    tier: hello.tier ?? "unknown",
    seconds: Math.round(elapsed * 100) / 100,
    frames: t.frames,
    ticks: t1 - t0,
    interval: stats(t.intervals),
    work: stats(t.work),
    device,
    errors: d.errors(),
  };
}

// ── the report ────────────────────────────────────────────────────────

interface Finding {
  measure: string;
  where: string;
  detail: string;
  fix: string;
}

/** 100 at `good`, 0 at `bad`, linear between (og polish's band). */
const band = (x: number, good: number, bad: number) => Math.round(100 * Math.max(0, Math.min(1, (bad - x) / (bad - good))));

/** The design's first-frame budgets (docs/design.md §7): mid-range Android on 4G 2.0 s, iPhone 13 1.5 s. */
const BUDGET_MS: Record<string, number> = { chrome: 2000, android: 2000, iphone: 1500 };

function judge(kind: string, m: Measured, served: Served): { scores: { firstFrame: number; frameTime: number }; findings: Finding[] } {
  const budget = BUDGET_MS[kind]!;
  const findings: Finding[] = [];
  if (m.firstFrameMs > budget)
    findings.push({
      measure: "first frame",
      where: "boot",
      detail: `${m.firstFrameMs} ms from navigation to the first frame, over the ${budget} ms budget`,
      fix: "og-game §6: keep the first group small (stream the rest), load in `start`, and check the size table of og build --publish",
    });
  if (m.interval.p95 > 20)
    findings.push({
      measure: "frame time",
      where: m.tier,
      detail: `p95 display interval ${m.interval.p95} ms (p99 ${m.interval.p99}, max ${m.interval.max}): frames are missed`,
      fix: "og-game §14: fewer draws and particles, less per-tick TypeScript; og polish names the step and render cost",
    });
  if (m.work.p95 > 16.7)
    findings.push({
      measure: "engine work",
      where: m.tier,
      detail: `p95 engine work per frame ${m.work.p95} ms, over a 60 Hz frame`,
      fix: "og-game §14: the tick or the render submission is too heavy for this device",
    });
  if (!m.device.crossOriginIsolated)
    findings.push({
      measure: "isolation",
      where: m.device.isSecureContext ? "headers" : `${served.scheme} at the LAN address`,
      detail: m.device.isSecureContext ? "the page is not cross-origin isolated although COOP/COEP were sent" : "not a secure context, so no cross-origin isolation (and no WebGPU)",
      fix: m.device.isSecureContext ? "a resource is missing CORP or COEP; read device.json's serve headers" : "serve over localhost (Android: adb reverse, which --device android does) or HTTPS with --cert/--key (mkcert)",
    });
  if (m.errors.length)
    findings.push({ measure: "console", where: "page", detail: `${m.errors.length} console error(s); first: ${m.errors[0]}`, fix: "read device.json's errors and fix the first" });
  return { scores: { firstFrame: band(m.firstFrameMs, budget / 2, budget * 2), frameTime: band(m.interval.p95, 17, 33.3) }, findings };
}

function markdown(r: Report): string {
  const m = r.measured;
  const d = m.device;
  const L: string[] = [];
  L.push(`# og run --device: ${r.project}`, "");
  L.push(`${r.date}, ${r.driver}, ${m.tier}. The publish build served at ${r.url}${r.lan.length ? ` (LAN ${r.lan.map((a) => `${r.scheme}://${a}:${r.port}/`).join(", ")})` : ""}, opened cold, the gate tapped, then ${m.seconds} s of play timed in the Worker.`, "");
  L.push("## Scores", "");
  L.push("| | Score | Measure |", "|---|---|---|");
  L.push(`| First frame | ${r.scores.firstFrame} | ms from navigation to the first frame (${r.budgetMs / 2} ms: 100, ${r.budgetMs * 2} ms: 0; budget ${r.budgetMs} ms) |`);
  L.push(`| Frame time | ${r.scores.frameTime} | p95 display interval of the played frames (17 ms: 100, 33.3 ms: 0) |`, "");
  L.push("## Fixes", "");
  if (r.findings.length === 0) L.push("No finding crossed a threshold.", "");
  else {
    L.push("| Measure | Where | Found | Fix |", "|---|---|---|---|");
    for (const f of r.findings) L.push(`| ${f.measure} | ${f.where} | ${f.detail.replace(/\|/g, "\\|")} | ${f.fix} |`);
    L.push("");
  }
  L.push("## Device", "", "| | |", "|---|---|");
  L.push(`| User agent | ${d.userAgent} |`);
  L.push(`| Platform | ${d.platform}${d.mobile ? ", mobile" : ""} |`);
  L.push(`| Tier | ${m.tier} |`);
  if (d.gpu) L.push(`| WebGPU adapter | ${[d.gpu.vendor, d.gpu.architecture, d.gpu.description].filter(Boolean).join(", ") || "(no info)"} |`);
  if (d.webgl) L.push(`| WebGL2 renderer | ${d.webgl.renderer} (${d.webgl.vendor}) |`);
  L.push(`| DPR, viewport, screen | ${d.dpr}, ${d.viewport.width}×${d.viewport.height} CSS px, ${d.screen.width}×${d.screen.height} |`);
  if (d.canvas) L.push(`| Canvas | ${d.canvas.width}×${d.canvas.height} device px |`);
  L.push(`| Cores, memory | ${d.hardwareConcurrency ?? "n/a"}, ${d.deviceMemory === null ? "n/a" : `${d.deviceMemory} GB`} |`);
  L.push(`| Isolated, secure | ${d.crossOriginIsolated ? "yes" : "no"}, ${d.isSecureContext ? "yes" : "no"} |`, "");
  L.push("## Frames", "", "| | count | mean | p50 | p95 | p99 | max |", "|---|---|---|---|---|---|---|");
  for (const [name, s] of [["Display interval (ms)", m.interval], ["Engine work (ms)", m.work]] as const) L.push(`| ${name} | ${s.count} | ${s.mean} | ${s.p50} | ${s.p95} | ${s.p99} | ${s.max} |`);
  L.push("");
  L.push(`First frame ${m.firstFrameMs} ms, ready ${m.readyMs} ms, play started ${m.startMs} ms after navigation; ${m.frames} frames and ${m.ticks} ticks in ${m.seconds} s (${r.fps} fps, ${r.ticksPerSecond} ticks/s).`, "");
  L.push("## Served", "", "| Path | COOP | COEP | CORP |", "|---|---|---|---|");
  for (const [path, h] of Object.entries(r.served)) L.push(`| ${path} | ${h["cross-origin-opener-policy"] ?? "-"} | ${h["cross-origin-embedder-policy"] ?? "-"} | ${h["cross-origin-resource-policy"] ?? "-"} |`);
  L.push("");
  return L.join("\n") + "\n";
}

interface Report {
  project: string;
  date: string;
  target: string;
  driver: string;
  url: string;
  scheme: string;
  port: number;
  lan: string[];
  budgetMs: number;
  scores: { firstFrame: number; frameTime: number };
  findings: Finding[];
  fps: number;
  ticksPerSecond: number;
  measured: Measured;
  served: Record<string, Record<string, string>>;
}

async function main(): Promise<void> {
  const dist = opt("--dist");
  const project = opt("--project");
  const target = opt("--device");
  const out = opt("--out");
  const preflight = flag("--preflight");
  if (!target || (!preflight && (!dist || !project || !out)))
    refuse("usage: bun src/node/device.ts --dist <build> --project <root> --device chrome|android[:serial]|iphone[:udid] --out <dir> [--seconds N] [--port P] [--headed] [--webgl] [--chrome <path>] [--cert <pem> --key <pem>] [--size WxH] [--json] (--preflight: only check the options and the target's tools)");
  const [kindRaw, ...rest] = target!.split(":");
  const kind = kindRaw === "ios" ? "iphone" : kindRaw!;
  if (!(kind in BUDGET_MS)) refuse(`--device ${target}: the targets are chrome, android[:serial] and iphone[:udid] (or ios[:udid])`);
  const id = rest.join(":") || null;
  const seconds = Number(opt("--seconds") ?? 5);
  if (!(seconds > 0)) refuse(`--seconds ${opt("--seconds")}: a positive number of seconds`);
  const [w, h] = (opt("--size") ?? "1280x720").split("x").map(Number);
  if (!(w! > 0 && h! > 0)) refuse(`--size ${opt("--size")}: WxH, as 1280x720`);
  const cert = opt("--cert");
  const key = opt("--key");
  if (!!cert !== !!key) refuse("--cert and --key go together: a PEM certificate and its key (mkcert <LAN address> makes both)");
  for (const f of [cert, key]) if (f && !existsSync(f)) refuse(`no file ${f}`);
  const tls = cert && key ? { cert: readFileSync(cert, "utf8"), key: readFileSync(key, "utf8") } : null;
  // The target's tools, before og spends a build on it.
  if (kind === "chrome") findChrome();
  else if (kind === "android" && !Bun.which("adb")) refuse(NO_ADB);
  else if (kind === "iphone" && (process.platform !== "darwin" || !Bun.which("safaridriver"))) refuse(NO_SAFARIDRIVER);
  if (preflight) return;
  if (!existsSync(join(dist!, "index.html"))) refuse(`${dist} has no index.html: og run --device builds it with og build --publish`);

  const served = serveBuild(resolve(dist!), Number(opt("--port") ?? 0), tls);
  const lan = lanAddresses();
  const query = flag("--webgl") ? "?webgl=1" : "";
  process.stderr.write(`og run --device: serving ${dist} at ${served.scheme}://127.0.0.1:${served.port}/${lan.map((a) => `, ${served.scheme}://${a}:${served.port}/`).join("")} (COOP/COEP)\n`);
  let driver: Driver | null = null;
  try {
    let url = `${served.scheme}://127.0.0.1:${served.port}/${query}`;
    if (kind === "chrome") driver = await launchChrome(w!, h!);
    else if (kind === "android") {
      const a = await connectAndroid(id, served.port);
      driver = a.driver;
      url = a.url + query;
    } else {
      if (lan.length === 0) refuse("no LAN address: join the iPhone's Wi-Fi network");
      driver = await connectIphone(id);
      url = `${served.scheme}://${lan[0]}:${served.port}/${query}`;
      if (!tls) process.stderr.write("og run --device: an http LAN page is not a secure context on the iPhone (no isolation, WebGL2 only); --cert/--key serve HTTPS\n");
    }
    const m = await measure(driver, url, seconds);
    const { scores, findings } = judge(kind, m, served);
    const report: Report = {
      project: basename(resolve(project!)),
      date: new Date().toISOString().slice(0, 10),
      target: target!,
      driver: driver.name,
      url,
      scheme: served.scheme,
      port: served.port,
      lan,
      budgetMs: BUDGET_MS[kind]!,
      scores,
      findings,
      fps: Math.round((m.frames / m.seconds) * 10) / 10,
      ticksPerSecond: Math.round((m.ticks / m.seconds) * 10) / 10,
      measured: m,
      served: served.sent,
    };
    const dir = resolve(out!);
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, "device.json"), JSON.stringify(report, null, 1) + "\n");
    writeFileSync(join(dir, "report.md"), markdown(report));
    if (flag("--json")) process.stdout.write(JSON.stringify({ project: report.project, target, tier: m.tier, firstFrameMs: m.firstFrameMs, interval: m.interval, scores, findings: findings.length, report: join(dir, "report.md") }) + "\n");
    else
      process.stdout.write(
        `og run --device ${target}: ${m.tier}, first frame ${m.firstFrameMs} ms, frame p50 ${m.interval.p50} / p95 ${m.interval.p95} / p99 ${m.interval.p99} ms; ${findings.length} finding(s); ${join(dir, "report.md")}\n`,
      );
  } finally {
    await driver?.close().catch(() => {});
    served.stop();
  }
}

if (import.meta.main) {
  main().then(
    () => process.exit(0),
    (e) => {
      process.stderr.write(`og run --device: ${e instanceof Refusal ? e.message : e instanceof Error ? (e.stack ?? e.message) : String(e)}\n`);
      process.exit(1);
    },
  );
}
