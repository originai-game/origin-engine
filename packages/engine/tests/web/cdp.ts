// Headless Chrome for Testing over the DevTools protocol, for the browser
// tests of the web export (wire.ts, export.ts): launch, open a page, evaluate,
// screenshot, dispatch real input. No dependency beyond bun's WebSocket.
//
// Chrome: --chrome <path>, $CHROME, or the agent-browser install
// (~/.agent-browser/browsers/chrome-150.0.7871.115). A missing Chrome is an
// error, never a skip.
//
// --android [serial] (Browser.android): Chrome on a phone over adb instead,
// its DevTools socket forwarded here and the test server's ports reversed to
// the phone, so the page's 127.0.0.1 and localhost are this machine's (a
// secure context, as on the desktop). The phone's Chrome is the user's own:
// a run opens its tabs there and closes only those.

import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

declare module "node:fs" {
  export function mkdtempSync(prefix: string): string;
}

declare const Bun: {
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { kill(): void; exited: Promise<number> };
  spawnSync(cmd: string[], o?: { stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

export function chromePath(argv: string[] = process.argv): string {
  const i = argv.indexOf("--chrome");
  const path =
    (i >= 0 ? argv[i + 1] : undefined) ??
    process.env.CHROME ??
    (process.platform === "darwin"
      ? `${process.env.HOME}/.agent-browser/browsers/chrome-150.0.7871.115/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`
      : `${process.env.HOME}/.agent-browser/browsers/chrome-150.0.7871.115/chrome`);
  if (!existsSync(path)) throw new Error(`no Chrome at ${path}: pass --chrome <path> or set CHROME (Chrome for Testing 150 is what the evidence used)`);
  return path;
}

/** Poll `read` (the test's own state, a server's requests) until it is truthy; throws naming `what` after `timeoutMs`. */
export async function waitFor<T>(read: () => T, what: string, timeoutMs = 30_000): Promise<NonNullable<T>> {
  const t0 = performance.now();
  for (;;) {
    const v = read();
    if (v) return v as NonNullable<T>;
    if (performance.now() - t0 > timeoutMs) throw new Error(`timed out after ${timeoutMs} ms waiting for ${what}`);
    await new Promise((r) => setTimeout(r, 50));
  }
}

type Pending = { resolve(v: unknown): void; reject(e: Error): void };

/** The device an `--android [serial]` run attaches to, when the arguments ask for one. */
export function androidOf(argv: string[] = process.argv): { serial: string | null } | undefined {
  const i = argv.indexOf("--android");
  if (i < 0) return undefined;
  const next = argv[i + 1];
  return { serial: next && !next.startsWith("--") ? next : null };
}

function adb(serial: string | null, ...args: string[]): string {
  const r = Bun.spawnSync(["adb", ...(serial ? ["-s", serial] : []), ...args], { stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`adb ${args.join(" ")} failed: ${new TextDecoder().decode(r.stderr).trim()} (is the phone unlocked and USB debugging allowed?)`);
  return new TextDecoder().decode(r.stdout).trim();
}

export class Browser {
  readonly #ws: WebSocket;
  /** The Chrome this run launched; null on a phone, whose Chrome stays open (only this run's tabs close). */
  readonly #proc: { kill(): void; exited: Promise<number> } | null;
  readonly #profile: string;
  /** The phone a `Browser.android` run attached to: its serial and the ports adb forwards and reverses. */
  readonly #android: { serial: string; forwarded: number; reversed: number[] } | null;
  readonly #targets = new Set<string>();
  readonly #pending = new Map<number, Pending>();
  readonly #listeners = new Set<(method: string, params: Record<string, unknown>, session?: string) => void>();
  #next = 1;
  /** The device a run on a phone attached to (model and serial), else null. */
  readonly device: string | null;

  private constructor(
    ws: WebSocket,
    proc: { kill(): void; exited: Promise<number> } | null,
    profile: string,
    android: { serial: string; forwarded: number; reversed: number[]; model: string } | null = null,
  ) {
    this.#ws = ws;
    this.#proc = proc;
    this.#profile = profile;
    this.#android = android;
    this.device = android ? `${android.model} (${android.serial})` : null;
    ws.addEventListener("message", (e) => {
      const m = JSON.parse(String((e as MessageEvent).data)) as { id?: number; result?: unknown; error?: { message: string }; method?: string; params?: Record<string, unknown>; sessionId?: string };
      if (m.id !== undefined) {
        const p = this.#pending.get(m.id);
        this.#pending.delete(m.id);
        if (m.error) p?.reject(new Error(m.error.message));
        else p?.resolve(m.result);
      } else if (m.method) for (const l of this.#listeners) l(m.method, m.params ?? {}, m.sessionId);
    });
  }

  /** `webgpu` false leaves out the switches that turn WebGPU on (unsupported.ts turns it off). */
  static async launch(o: { chrome?: string; width?: number; height?: number; flags?: string[]; webgpu?: boolean } = {}): Promise<Browser> {
    // os.tmpdir(): TMPDIR else /tmp on Unix, %TEMP% on Windows (where "/tmp" is \tmp on the current drive).
    const profile = mkdtempSync(join(tmpdir(), "og-cdp-"));
    const proc = Bun.spawn(
      [
        o.chrome ?? chromePath(),
        "--headless=new", "--no-first-run", "--no-default-browser-check", `--user-data-dir=${profile}`,
        // References are English: the host's UI language must not pick the locale.
        "--lang=en-US", "--accept-lang=en-US",
        `--window-size=${o.width ?? 1280},${o.height ?? 720}`, "--hide-scrollbars",
        ...(process.platform === "darwin" ? ["--use-angle=metal"] : []),
        ...(o.webgpu === false ? [] : ["--enable-unsafe-webgpu", "--enable-features=WebGPU"]),
        "--ignore-gpu-blocklist", "--disable-gpu-vsync",
        "--autoplay-policy=user-gesture-required", "--remote-debugging-port=0",
        ...(o.flags ?? []),
        "about:blank",
      ],
      { stdout: "ignore", stderr: "ignore" },
    );
    const portFile = join(profile, "DevToolsActivePort");
    const t0 = performance.now();
    let lines: string[] = [];
    while (lines.length < 2) {
      if (performance.now() - t0 > 20_000) throw new Error("Chrome did not open its DevTools port in 20 s");
      await new Promise((r) => setTimeout(r, 50));
      // Not written yet, or still open in Chrome (EBUSY on Windows): try again.
      try {
        lines = readFileSync(portFile, "utf8").trim().split("\n");
      } catch {}
    }
    const [port, path] = lines;
    const ws = new WebSocket(`ws://127.0.0.1:${port}${path}`);
    await new Promise<void>((resolve, reject) => {
      ws.addEventListener("open", () => resolve());
      ws.addEventListener("error", () => reject(new Error("cannot connect to Chrome's DevTools socket")));
    });
    return new Browser(ws, proc, profile);
  }

  /**
   * Chrome on an Android phone (`adb devices` lists it as `device`; one, or
   * the one `serial` names): its DevTools socket on a forwarded port, and
   * each of `reverse` (the test server's ports) reversed so the phone's
   * 127.0.0.1:<port> is this machine's.
   */
  static async android(o: { serial?: string | null; reverse: number[] }): Promise<Browser> {
    const devices = adb(null, "devices")
      .split("\n")
      .slice(1)
      .map((l) => l.trim().split(/\s+/))
      .filter((p) => p[1] === "device")
      .map((p) => p[0]!);
    if (devices.length === 0) throw new Error("no Android device: connect one with USB debugging on and accept the prompt (`adb devices` lists it as device)");
    if (o.serial && !devices.includes(o.serial)) throw new Error(`no Android device ${o.serial}: adb lists ${devices.join(", ") || "none"}`);
    if (!o.serial && devices.length > 1) throw new Error(`several Android devices (${devices.join(", ")}): pass --android <serial>`);
    const serial = o.serial ?? devices[0]!;
    for (const p of o.reverse) adb(serial, "reverse", `tcp:${p}`, `tcp:${p}`);
    adb(serial, "shell", "am", "start", "-n", "com.android.chrome/com.google.android.apps.chrome.Main", "-d", "about:blank");
    const forwarded = Number(adb(serial, "forward", "tcp:0", "localabstract:chrome_devtools_remote"));
    let url: string | undefined;
    for (let i = 0; i < 50 && !url; i++) {
      try {
        url = ((await (await fetch(`http://127.0.0.1:${forwarded}/json/version`)).json()) as { webSocketDebuggerUrl?: string }).webSocketDebuggerUrl;
      } catch {
        await new Promise((r) => setTimeout(r, 200));
      }
    }
    if (!url) throw new Error(`Chrome on ${serial} does not answer DevTools: unlock the phone and open Chrome once`);
    const ws = new WebSocket(url.replace(/^ws:\/\/[^/]+/, `ws://127.0.0.1:${forwarded}`));
    await new Promise<void>((resolve, reject) => {
      ws.addEventListener("open", () => resolve());
      ws.addEventListener("error", () => reject(new Error(`cannot connect to the DevTools socket of Chrome on ${serial}`)));
    });
    const model = adb(serial, "shell", "getprop", "ro.product.model");
    return new Browser(ws, null, "", { serial, forwarded, reversed: o.reverse, model });
  }

  /** `Browser.android` when the arguments carry --android [serial], else `Browser.launch`. */
  static async from(argv: string[], o: { chrome?: string; width?: number; height?: number; flags?: string[]; webgpu?: boolean; reverse: number[] }): Promise<Browser> {
    const phone = androidOf(argv);
    return phone ? Browser.android({ serial: phone.serial, reverse: o.reverse }) : Browser.launch(o);
  }

  /** A server started after the browser: on a phone its port is reversed too (closed with the browser); on this machine nothing to do. */
  reverse(port: number): void {
    if (!this.#android || this.#android.reversed.includes(port)) return;
    adb(this.#android.serial, "reverse", `tcp:${port}`, `tcp:${port}`);
    this.#android.reversed.push(port);
  }

  send<T = Record<string, unknown>>(method: string, params: Record<string, unknown> = {}, sessionId?: string): Promise<T> {
    const id = this.#next++;
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.#ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }

  on(fn: (method: string, params: Record<string, unknown>, session?: string) => void): () => void {
    this.#listeners.add(fn);
    return () => this.#listeners.delete(fn);
  }

  /** A new tab at `url`, attached; console messages and exceptions go to `log`. */
  async open(url: string, log: string[] = []): Promise<Page> {
    const { targetId } = await this.send<{ targetId: string }>("Target.createTarget", { url: "about:blank" });
    this.#targets.add(targetId);
    const { sessionId } = await this.send<{ sessionId: string }>("Target.attachToTarget", { targetId, flatten: true });
    const page = new Page(this, sessionId, log);
    await page.send("Page.enable");
    await page.send("Runtime.enable");
    await page.attachFrames();
    await page.send("Page.navigate", { url });
    return page;
  }

  async close(): Promise<void> {
    if (this.#android) {
      const a = this.#android;
      for (const t of this.#targets) await Promise.race([this.send("Target.closeTarget", { targetId: t }).catch(() => {}), new Promise((r) => setTimeout(r, 2000))]);
      try {
        this.#ws.close();
      } catch {
        /* already closed */
      }
      Bun.spawnSync(["adb", "-s", a.serial, "forward", "--remove", `tcp:${a.forwarded}`], { stdout: "ignore", stderr: "ignore" });
      for (const p of a.reversed) Bun.spawnSync(["adb", "-s", a.serial, "reverse", "--remove", `tcp:${p}`], { stdout: "ignore", stderr: "ignore" });
      return;
    }
    // Chrome closes itself and every process it started: on Windows a kill
    // ends the process Bun spawned and leaves the browser, its GPU and
    // renderer processes running with the profile open. The kill stays for
    // a browser that does not answer within 5 s.
    const closed = await Promise.race([
      this.send("Browser.close").then(() => this.#proc!.exited).then(() => true, () => false),
      new Promise<boolean>((r) => setTimeout(() => r(false), 5000)),
    ]);
    try {
      this.#ws.close();
    } catch {
      /* already closed */
    }
    if (!closed) this.#proc!.kill();
    await this.#proc!.exited;
    // Windows: the last child process can hold a file of the profile for a
    // moment after the browser has exited (EBUSY).
    for (let tries = 0; ; tries++) {
      try {
        rmSync(this.#profile, { recursive: true, force: true });
        return;
      } catch (e) {
        if ((e as { code?: string }).code !== "EBUSY" || tries >= 50) throw e;
        await new Promise((r) => setTimeout(r, 100));
      }
    }
  }
}

export class Page {
  readonly #browser: Browser;
  readonly session: string;
  readonly log: string[];
  /** Execution contexts by frame id (the default world of each frame). */
  readonly contexts = new Map<string, number>();
  /** Out-of-process frames attached to this page (cross-origin iframes): url at attach time by session. */
  readonly children = new Map<string, string>();
  /** The Workers this page started (and those they started), by session: their type and script URL. */
  readonly workers = new Map<string, { type: string; url: string }>();
  /** Runs in each new Worker before its first line (prepareWorkers). */
  #prepare: ((worker: { type: string; url: string }, session: string) => Promise<void>) | undefined;

  constructor(browser: Browser, session: string, log: string[]) {
    this.#browser = browser;
    this.session = session;
    this.log = log;
    browser.on((method, params, s) => {
      // A Worker started by one of this page's Workers (the lanes pool): held for prepare too.
      if (s !== undefined && this.workers.has(s) && method === "Target.attachedToTarget") {
        this.#held(params);
        return;
      }
      if (s !== session) return;
      if (method === "Runtime.consoleAPICalled") {
        const args = (params["args"] as Array<{ value?: unknown; description?: string }>) ?? [];
        log.push(`${params["type"]}: ${args.map((a) => (a.value !== undefined ? String(a.value) : (a.description ?? ""))).join(" ")}`);
      } else if (method === "Runtime.exceptionThrown") {
        const d = params["exceptionDetails"] as { text?: string; exception?: { description?: string } };
        log.push(`exception: ${d.exception?.description ?? d.text}`);
      } else if (method === "Target.attachedToTarget") {
        const info = params["targetInfo"] as { url: string; type: string };
        const child = params["sessionId"] as string;
        // Frames only: Workers and service workers attach too.
        if (info.type === "iframe" || info.type === "page") this.children.set(child, info.url);
        else if (this.#prepare) this.#held(params);
        else void browser.send("Runtime.runIfWaitingForDebugger", {}, child).catch(() => {});
        if (info.type === "iframe" || info.type === "page") void browser.send("Runtime.runIfWaitingForDebugger", {}, child).catch(() => {});
      } else if (method === "Runtime.executionContextCreated") {
        const c = params["context"] as { id: number; auxData?: { frameId?: string; isDefault?: boolean } };
        if (c.auxData?.isDefault && c.auxData.frameId) this.contexts.set(c.auxData.frameId, c.id);
      }
    });
  }

  send<T = Record<string, unknown>>(method: string, params: Record<string, unknown> = {}): Promise<T> {
    return this.#browser.send<T>(method, params, this.session);
  }

  /**
   * Hold every Worker this page starts (and every Worker those start) at
   * its first line, run `prepare` in it, then let it go: a Worker's own
   * globals can be set before its script reads them (published.ts installs
   * its test clock in the engine Worker this way). Call before navigating.
   */
  async prepareWorkers(prepare: (worker: { type: string; url: string }, session: string) => Promise<void>): Promise<void> {
    this.#prepare = prepare;
    await this.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: true, flatten: true });
  }

  /** A Worker attached and waiting: prepare it, hold the Workers it starts, then run it. */
  #held(params: Record<string, unknown>): void {
    const info = params["targetInfo"] as { type: string; url: string };
    const child = params["sessionId"] as string;
    const worker = { type: info.type, url: info.url };
    this.workers.set(child, worker);
    const b = this.#browser;
    void (async () => {
      try {
        await b.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: true, flatten: true }, child).catch(() => {});
        await this.#prepare?.(worker, child);
      } finally {
        await b.send("Runtime.runIfWaitingForDebugger", {}, child).catch(() => {});
      }
    })();
  }

  /** Evaluate an expression (awaiting a Promise) in one of this page's Workers (`workers`). */
  async evalWorker<T = unknown>(session: string, expression: string): Promise<T> {
    const r = await this.#browser.send<{ result: { value?: T }; exceptionDetails?: { text: string; exception?: { description?: string } } }>(
      "Runtime.evaluate",
      { expression, awaitPromise: true, returnByValue: true },
      session,
    );
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value as T;
  }

  /** Attach to cross-origin frames as they appear (call before navigating to a page that has them). */
  async attachFrames(): Promise<void> {
    await this.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: false, flatten: true });
  }

  /** Evaluate in the out-of-process frame whose URL contains `part`. */
  async evalChild<T = unknown>(part: string, expression: string): Promise<T> {
    // Attached frames change URL after attaching: ask each where it is.
    let session: string | undefined;
    for (const child of this.children.keys()) {
      const r = await this.#browser
        .send<{ result: { value?: string } }>("Runtime.evaluate", { expression: "location.href", returnByValue: true }, child)
        .catch(() => null);
      if (r?.result.value?.includes(part)) session = child;
    }
    if (!session) {
      // An in-process frame: its default context in this page's session.
      const frame = (await this.frames()).find((f) => f.url.includes(part));
      const context = frame ? this.contexts.get(frame.id) : undefined;
      if (!context) throw new Error(`no frame at ${part} (attached: ${[...this.children.values()].join(", ") || "none"})`);
      return this.eval<T>(expression, context);
    }
    const r = await this.#browser.send<{ result: { value?: T }; exceptionDetails?: { text: string; exception?: { description?: string } } }>(
      "Runtime.evaluate",
      { expression, awaitPromise: true, returnByValue: true },
      session,
    );
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value as T;
  }

  /** Evaluate an expression (awaiting a Promise) in the page, or in a frame's context. */
  async eval<T = unknown>(expression: string, contextId?: number): Promise<T> {
    const r = await this.send<{ result: { value?: T }; exceptionDetails?: { text: string; exception?: { description?: string } } }>("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
      ...(contextId ? { contextId } : {}),
    });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value as T;
  }

  /** Wait until `expression` is truthy (polled). */
  async until<T = unknown>(expression: string, timeoutMs = 30_000, contextId?: number): Promise<T> {
    const t0 = performance.now();
    for (;;) {
      try {
        const v = await this.eval<T>(expression, contextId);
        if (v) return v;
      } catch {
        /* the page is still loading */
      }
      if (performance.now() - t0 > timeoutMs) throw new Error(`timed out after ${timeoutMs} ms waiting for ${expression}`);
      await new Promise((r) => setTimeout(r, 50));
    }
  }

  /** Resolve once the page has run `n` animation frames (requestAnimationFrame callbacks). */
  async afterFrames(n: number): Promise<void> {
    await this.eval(`new Promise((r) => { let k = 0; const f = () => (++k >= ${n} ? r(true) : requestAnimationFrame(f)); requestAnimationFrame(f); })`);
  }

  /**
   * Wait until `expression`'s value is the same before and after `frames`
   * animation frames of the page (a clock that settled or holds), and answer
   * it; throws `what` did not hold after `timeoutMs`, with the last values.
   */
  async held<T>(expression: string, what: string, o: { frames?: number; timeoutMs?: number } = {}): Promise<T> {
    const t0 = performance.now();
    const timeoutMs = o.timeoutMs ?? 30_000;
    let seen: unknown[] = [];
    for (;;) {
      const a = await this.eval<T>(expression);
      await this.afterFrames(o.frames ?? 30);
      const b = await this.eval<T>(expression);
      if (JSON.stringify(a) === JSON.stringify(b)) return b;
      seen = [a, b];
      if (performance.now() - t0 > timeoutMs) throw new Error(`timed out after ${timeoutMs} ms waiting for ${what} to hold (last ${JSON.stringify(seen)})`);
    }
  }

  async screenshot(): Promise<Uint8Array> {
    const { data } = await this.send<{ data: string }>("Page.captureScreenshot", { format: "png" });
    return Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
  }

  /** A tap (touch) at CSS pixel x, y of the page. */
  async tap(x: number, y: number, id = 0): Promise<void> {
    await this.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x, y, id }] });
    await this.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  }

  async click(x: number, y: number): Promise<void> {
    for (const type of ["mousePressed", "mouseReleased"]) await this.send("Input.dispatchMouseEvent", { type, x, y, button: "left", clickCount: 1 });
  }

  async key(code: string, key = code): Promise<void> {
    for (const type of ["keyDown", "keyUp"]) await this.send("Input.dispatchKeyEvent", { type, code, key });
  }

  /** The frame tree's child frames (id, url). */
  async frames(): Promise<Array<{ id: string; url: string }>> {
    const { frameTree } = await this.send<{ frameTree: { frame: { id: string; url: string }; childFrames?: Array<{ frame: { id: string; url: string } }> } }>("Page.getFrameTree");
    return (frameTree.childFrames ?? []).map((c) => c.frame);
  }
}
