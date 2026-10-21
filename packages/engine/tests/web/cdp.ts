// Headless Chrome for Testing over the DevTools protocol, for the browser
// tests of the web export (wire.ts, export.ts): launch, open a page, evaluate,
// screenshot, dispatch real input. No dependency beyond bun's WebSocket.
//
// Chrome: --chrome <path>, $CHROME, or the agent-browser install
// (~/.agent-browser/browsers/chrome-150.0.7871.115). A missing Chrome is an
// error, never a skip.

import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";

declare module "node:fs" {
  export function mkdtempSync(prefix: string): string;
}

declare const Bun: {
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { kill(): void; exited: Promise<number> };
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

type Pending = { resolve(v: unknown): void; reject(e: Error): void };

export class Browser {
  readonly #ws: WebSocket;
  readonly #proc: { kill(): void; exited: Promise<number> };
  readonly #profile: string;
  readonly #pending = new Map<number, Pending>();
  readonly #listeners = new Set<(method: string, params: Record<string, unknown>, session?: string) => void>();
  #next = 1;

  private constructor(ws: WebSocket, proc: { kill(): void; exited: Promise<number> }, profile: string) {
    this.#ws = ws;
    this.#proc = proc;
    this.#profile = profile;
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

  static async launch(o: { chrome?: string; width?: number; height?: number; flags?: string[] } = {}): Promise<Browser> {
    const profile = mkdtempSync(join(process.env.TMPDIR ?? "/tmp", "og-cdp-"));
    const proc = Bun.spawn(
      [
        o.chrome ?? chromePath(),
        "--headless=new", "--no-first-run", "--no-default-browser-check", `--user-data-dir=${profile}`,
        `--window-size=${o.width ?? 1280},${o.height ?? 720}`, "--hide-scrollbars",
        ...(process.platform === "darwin" ? ["--use-angle=metal"] : []),
        "--enable-unsafe-webgpu", "--enable-features=WebGPU", "--ignore-gpu-blocklist", "--disable-gpu-vsync",
        "--autoplay-policy=user-gesture-required", "--remote-debugging-port=0",
        ...(o.flags ?? []),
        "about:blank",
      ],
      { stdout: "ignore", stderr: "ignore" },
    );
    const portFile = join(profile, "DevToolsActivePort");
    const t0 = performance.now();
    while (!existsSync(portFile) || readFileSync(portFile, "utf8").split("\n").length < 2) {
      if (performance.now() - t0 > 20_000) throw new Error("Chrome did not open its DevTools port in 20 s");
      await new Promise((r) => setTimeout(r, 50));
    }
    const [port, path] = readFileSync(portFile, "utf8").trim().split("\n");
    const ws = new WebSocket(`ws://127.0.0.1:${port}${path}`);
    await new Promise<void>((resolve, reject) => {
      ws.addEventListener("open", () => resolve());
      ws.addEventListener("error", () => reject(new Error("cannot connect to Chrome's DevTools socket")));
    });
    return new Browser(ws, proc, profile);
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
    const { sessionId } = await this.send<{ sessionId: string }>("Target.attachToTarget", { targetId, flatten: true });
    const page = new Page(this, sessionId, log);
    await page.send("Page.enable");
    await page.send("Runtime.enable");
    await page.attachFrames();
    await page.send("Page.navigate", { url });
    return page;
  }

  async close(): Promise<void> {
    try {
      this.#ws.close();
    } catch {
      /* already closed */
    }
    this.#proc.kill();
    await this.#proc.exited;
    rmSync(this.#profile, { recursive: true, force: true });
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
