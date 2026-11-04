// Safari over classic WebDriver (safaridriver), for the browser runners
// (reference.ts, reference-gfx.ts, wire.ts) with --safari: the part of
// cdp.ts's Browser and Page they use (open a page, size its viewport,
// navigate, reload, close, evaluate in the page or one of its frames,
// pointer and key input, screenshots). Safari has no CDP and its WebDriver
// has no console events, so the page's console errors, uncaught errors and
// rejections are collected in the page (a hook installed after each
// navigation) and handed back with every evaluation into `log`, in cdp.ts's
// lines ("error: ...", "exception: ..."). No dependency beyond fetch.
//
// safaridriver ships with macOS (/usr/bin/safaridriver); `safaridriver
// --enable` (once, with sudo) allows it to drive Safari. A missing driver is
// an error, never a skip. Safari runs one WebDriver session at a time on a
// machine, so runs that use it go through scripts/lock.sh.
//
// --safari-simulator <udid> drives Safari in that booted iOS Simulator
// instead of the Mac's (safaridriver's useSimulator), when the Simulator
// offers a WebDriver host.

import { existsSync } from "node:fs";

declare const Bun: {
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { kill(): void; exited: Promise<number> };
  serve(o: { port: number; fetch(req: Request): Response }): { port: number; stop(): void };
};

export function safariDriverPath(argv: string[] = process.argv): string {
  const i = argv.indexOf("--safari");
  const given = i >= 0 ? argv[i + 1] : undefined;
  const path = (given && !given.startsWith("--") ? given : undefined) ?? process.env.SAFARIDRIVER ?? "/usr/bin/safaridriver";
  if (!existsSync(path)) throw new Error(`no safaridriver at ${path}: Safari's WebDriver ships with macOS (pass --safari <path> or set SAFARIDRIVER)`);
  return path;
}

/** The element reference key of the WebDriver protocol. */
const ELEMENT = "element-6066-11e4-a52e-4f735466cecf";

/** WebDriver's key values for the DOM `code`s the runners press (single characters are sent as they are). */
const KEYS: Record<string, string> = {
  ArrowUp: "\uE013", ArrowDown: "\uE015", ArrowLeft: "\uE012", ArrowRight: "\uE014",
  Enter: "\uE007", Escape: "\uE00C", Tab: "\uE004", Backspace: "\uE003", Space: "\uE00D",
  ShiftLeft: "\uE008", ControlLeft: "\uE009", AltLeft: "\uE00A", MetaLeft: "\uE03D",
};

/**
 * The page-side hook: console.error and console.warn, uncaught errors and
 * unhandled rejections into a buffer the next evaluation drains.
 */
const HOOK = `(() => {
  if (globalThis.__og_wd_log__) return;
  const buf = (globalThis.__og_wd_log__ = []);
  const text = (a) => { try { return a instanceof Error ? (String(a) + (a.stack ? "\\n" + a.stack : "")) : typeof a === "object" ? JSON.stringify(a) : String(a); } catch { return String(a); } };
  for (const level of ["error", "warn"]) {
    const was = console[level].bind(console);
    console[level] = (...a) => { buf.push(level + ": " + a.map(text).join(" ")); was(...a); };
  }
  addEventListener("error", (e) => buf.push("exception: " + (e.error ? text(e.error) : e.message)));
  addEventListener("unhandledrejection", (e) => buf.push("exception: " + text(e.reason)));
})()`;

export class Safari {
  readonly #base: string;
  readonly #proc: { kill(): void; exited: Promise<number> };
  readonly #session: string;
  /** The window handle each page owns; the session's current window is one of them. */
  #current = "";
  readonly simulator: boolean;
  /** What the session reported (browserVersion, platformVersion, ...). */
  readonly capabilities: Record<string, unknown>;

  private constructor(base: string, proc: { kill(): void; exited: Promise<number> }, session: string, capabilities: Record<string, unknown>, simulator: boolean) {
    this.#base = base;
    this.#proc = proc;
    this.#session = session;
    this.capabilities = capabilities;
    this.simulator = simulator;
  }

  static async launch(o: { driver?: string; width?: number; height?: number; simulator?: string } = {}): Promise<Safari> {
    // A free port: bind one, let it go, hand it to safaridriver.
    const probe = Bun.serve({ port: 0, fetch: () => new Response("") });
    const port = probe.port;
    probe.stop();
    const proc = Bun.spawn([o.driver ?? safariDriverPath(), "-p", String(port)], { stdout: "ignore", stderr: "ignore" });
    const base = `http://127.0.0.1:${port}`;
    const t0 = performance.now();
    for (;;) {
      const up = await fetch(`${base}/status`).then((r) => r.ok, () => false);
      if (up) break;
      if (performance.now() - t0 > 20_000) {
        proc.kill();
        throw new Error("safaridriver did not answer on its port in 20 s");
      }
      await new Promise((r) => setTimeout(r, 100));
    }
    const caps = o.simulator
      ? { browserName: "safari", platformName: "iOS", "safari:useSimulator": true, "safari:deviceUDID": o.simulator }
      : { browserName: "safari", platformName: "mac" };
    // Safari admits one automation session per machine: a run that finds it
    // taken waits for it (scripts/lock.sh serialises ours; this covers a
    // session another tool left closing).
    let created: { value: { sessionId?: string; capabilities?: Record<string, unknown>; error?: string; message?: string } } | undefined;
    for (let tries = 0; ; tries++) {
      created = (await (await fetch(`${base}/session`, { method: "POST", body: JSON.stringify({ capabilities: { alwaysMatch: caps } }) })).json()) as typeof created;
      if (created!.value.sessionId) break;
      if (tries >= 2) {
        proc.kill();
        throw new Error(`safaridriver did not create a session: ${created!.value.error}: ${created!.value.message} (once: sudo safaridriver --enable)`);
      }
      await new Promise((r) => setTimeout(r, 3000));
    }
    const browser = new Safari(base, proc, created!.value.sessionId!, created!.value.capabilities ?? {}, !!o.simulator);
    await browser.send("POST", "/timeouts", { script: 30 * 60_000, pageLoad: 5 * 60_000, implicit: 0 });
    browser.#current = await browser.send<string>("GET", "/window");
    if (!o.simulator && o.width && o.height) await browser.send("POST", "/window/rect", { x: 0, y: 0, width: o.width, height: o.height + 100 });
    return browser;
  }

  /** One WebDriver command of this session; throws the driver's error. */
  async send<T = unknown>(method: "GET" | "POST" | "DELETE", path: string, body?: unknown): Promise<T> {
    const r = await fetch(`${this.#base}/session/${this.#session}${path}`, { method, ...(method === "POST" ? { body: JSON.stringify(body ?? {}) } : {}) });
    const j = (await r.json()) as { value: T & { error?: string; message?: string } };
    if (!r.ok) throw new Error(`WebDriver ${method} ${path}: ${j.value?.error}: ${j.value?.message}`);
    return j.value;
  }

  /** Make `handle` the session's current window (each page switches to its own before a command). */
  async focus(handle: string): Promise<void> {
    if (this.#current === handle) return;
    await this.send("POST", "/window", { handle });
    this.#current = handle;
  }

  #first = true;
  /** A page at `url` (the session's first window, then a new tab each); errors in it go to `log`. */
  async open(url: string, log: string[] = []): Promise<SafariPage> {
    let handle: string;
    if (this.#first) {
      this.#first = false;
      handle = this.#current;
    } else {
      handle = (await this.send<{ handle: string }>("POST", "/window/new", { type: "tab" })).handle;
    }
    const page = new SafariPage(this, handle, log);
    if (url !== "about:blank") await page.send("Page.navigate", { url });
    return page;
  }

  /** Called by a page that closed its window: the session moves to another. */
  async closed(handle: string): Promise<void> {
    if (this.#current !== handle) return;
    const left = await this.send<string[]>("GET", "/window/handles").catch(() => [] as string[]);
    this.#current = "";
    if (left[0]) await this.focus(left[0]);
  }

  /** This Mac's Safari reaches the test server at 127.0.0.1 as it is: nothing to reverse (cdp.ts Browser.reverse is for a phone). */
  reverse(_port: number): void {}

  async close(): Promise<void> {
    await Promise.race([this.send("DELETE", "").catch(() => {}), new Promise((r) => setTimeout(r, 10_000))]);
    this.#proc.kill();
    await this.#proc.exited;
  }
}

export class SafariPage {
  readonly #browser: Safari;
  readonly handle: string;
  readonly log: string[];

  constructor(browser: Safari, handle: string, log: string[]) {
    this.#browser = browser;
    this.handle = handle;
    this.log = log;
  }

  async #cmd<T = unknown>(method: "GET" | "POST" | "DELETE", path: string, body?: unknown): Promise<T> {
    await this.#browser.focus(this.handle);
    return this.#browser.send<T>(method, path, body);
  }

  /** The CDP commands the runners send to a page, in WebDriver. */
  async send(method: string, params: Record<string, unknown> = {}): Promise<void> {
    if (method === "Emulation.setDeviceMetricsOverride") {
      // The window's outer size is set; the chrome around the viewport is
      // measured and added, so the viewport is the size asked for. The
      // device pixel ratio is the display's (Safari cannot override it):
      // the runners' pages fix their canvas's pixel size with dprCap.
      if (this.#browser.simulator) return;
      const want = { w: Number(params["width"]), h: Number(params["height"]) };
      for (let i = 0; i < 3; i++) {
        const rect = await this.#cmd<{ width: number; height: number }>("GET", "/window/rect");
        const inner = await this.eval<{ w: number; h: number }>("({ w: innerWidth, h: innerHeight })");
        if (inner.w === want.w && inner.h === want.h) return;
        await this.#cmd("POST", "/window/rect", { width: rect.width + want.w - inner.w, height: rect.height + want.h - inner.h });
      }
    } else if (method === "Page.navigate") {
      await this.#cmd("POST", "/url", { url: params["url"] });
      await this.#hook();
    } else if (method === "Page.reload") {
      await this.#cmd("POST", "/refresh");
      await this.#hook();
    } else if (method === "Page.close") {
      // The session's last window stays, emptied: closing it would end the session.
      const all = await this.#cmd<string[]>("GET", "/window/handles");
      if (all.length <= 1) return void (await this.#cmd("POST", "/url", { url: "about:blank" }));
      await this.#cmd("DELETE", "/window");
      await this.#browser.closed(this.handle);
    } else throw new Error(`webdriver.ts: no ${method} (cdp.ts's Page has it)`);
  }

  async #hook(): Promise<void> {
    await this.#cmd("POST", "/execute/sync", { script: `${HOOK}; return true;`, args: [] }).catch(() => {});
  }

  /**
   * Evaluate an expression (awaiting a Promise) as CDP's Runtime.evaluate
   * does: global scope, its completion value, through JSON. The page's
   * collected errors come back with it into `log`.
   */
  async eval<T = unknown>(expression: string): Promise<T> {
    const script = `const done = arguments[arguments.length - 1];
      const drain = () => { const b = globalThis.__og_wd_log__; return b ? b.splice(0) : []; };
      Promise.resolve().then(() => (0, eval)(arguments[0])).then(
        (v) => done({ v: v === undefined ? undefined : JSON.stringify(v), log: drain() }),
        (e) => done({ e: String((e && (e.stack ? (e.message + "\\n" + e.stack) : e.message)) || e), log: drain() }));`;
    const r = await this.#cmd<{ v?: string; e?: string; log?: string[] }>("POST", "/execute/async", { script, args: [expression] });
    if (r?.log?.length) this.log.push(...r.log);
    if (r?.e !== undefined) throw new Error(r.e);
    return (r?.v === undefined || r.v === null ? undefined : JSON.parse(r.v)) as T;
  }

  /** Wait until `expression` is truthy (polled). */
  async until<T = unknown>(expression: string, timeoutMs = 30_000): Promise<T> {
    const t0 = performance.now();
    for (;;) {
      try {
        const v = await this.eval<T>(expression);
        if (v) return v;
      } catch {
        /* the page is still loading */
      }
      if (performance.now() - t0 > timeoutMs) throw new Error(`timed out after ${timeoutMs} ms waiting for ${expression}`);
      await new Promise((r) => setTimeout(r, 50));
    }
  }

  /** Resolve once the page has run `n` animation frames. */
  async afterFrames(n: number): Promise<void> {
    await this.eval(`new Promise((r) => { let k = 0; const f = () => (++k >= ${n} ? r(true) : requestAnimationFrame(f)); requestAnimationFrame(f); })`);
  }

  /** Evaluate in the frame whose URL contains `part` (a frame of the page's top document). */
  async evalChild<T = unknown>(part: string, expression: string): Promise<T> {
    const frame = await this.#cmd<Record<string, string> | null>("POST", "/execute/sync", {
      script: "return [...document.querySelectorAll('iframe')].find((f) => f.src.includes(arguments[0])) || null;",
      args: [part],
    });
    if (!frame?.[ELEMENT]) throw new Error(`no frame at ${part}`);
    await this.#cmd("POST", "/frame", { id: frame });
    try {
      await this.#hook();
      return await this.eval<T>(expression);
    } finally {
      await this.#cmd("POST", "/frame/parent");
    }
  }

  async screenshot(): Promise<Uint8Array> {
    const data = await this.#cmd<string>("GET", "/screenshot");
    return Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
  }

  async #pointer(type: "touch" | "mouse", x: number, y: number): Promise<void> {
    await this.#cmd("POST", "/actions", {
      actions: [{ type: "pointer", id: type, parameters: { pointerType: type }, actions: [
        { type: "pointerMove", x: Math.round(x), y: Math.round(y), origin: "viewport", duration: 0 },
        { type: "pointerDown", button: 0 },
        { type: "pointerUp", button: 0 },
      ] }],
    });
    await this.#cmd("DELETE", "/actions").catch(() => {});
  }

  /** A tap at CSS pixel x, y: a touch where Safari has touch input (iOS), else the mouse's press. */
  async tap(x: number, y: number): Promise<void> {
    await this.#pointer(this.#browser.simulator ? "touch" : "mouse", x, y);
  }

  async click(x: number, y: number): Promise<void> {
    await this.#pointer("mouse", x, y);
  }

  async key(code: string, key = code): Promise<void> {
    const value = KEYS[code] ?? (key.length === 1 ? key : undefined);
    if (value === undefined) throw new Error(`webdriver.ts: no WebDriver key for ${code}`);
    await this.#cmd("POST", "/actions", { actions: [{ type: "key", id: "keyboard", actions: [{ type: "keyDown", value }, { type: "keyUp", value }] }] });
    await this.#cmd("DELETE", "/actions").catch(() => {});
  }
}
