// Firefox over WebDriver BiDi, for the reference runners (reference.ts,
// reference-gfx.ts) with --firefox: the part of cdp.ts's Browser and Page
// they use (open a page, set its viewport, navigate, evaluate, the console
// in `log`, close). Firefox has no CDP since 141. No dependency beyond bun's
// WebSocket.
//
// Firefox: --firefox <path>, $FIREFOX, or the default install. A missing
// Firefox is an error, never a skip. Headless Firefox has no WebGPU adapter
// (157 on Windows), so a WebGPU run opens a window.

import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { testTempDir } from "../support/temp";

declare module "node:fs" {
  export function mkdtempSync(prefix: string): string;
}

declare const Bun: {
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { kill(): void; exited: Promise<number> };
};

export function firefoxPath(argv: string[] = process.argv): string {
  const i = argv.indexOf("--firefox");
  const given = i >= 0 ? argv[i + 1] : undefined;
  const path =
    (given && !given.startsWith("--") ? given : undefined) ??
    process.env.FIREFOX ??
    (process.platform === "win32"
      ? "C:/Program Files/Mozilla Firefox/firefox.exe"
      : process.platform === "darwin"
        ? "/Applications/Firefox.app/Contents/MacOS/firefox"
        : "/usr/bin/firefox");
  if (!existsSync(path)) throw new Error(`no Firefox at ${path}: pass --firefox <path> or set FIREFOX`);
  return path;
}

type Pending = { resolve(v: unknown): void; reject(e: Error): void };
type Remote = { type: string; value?: unknown };

export class Firefox {
  readonly #ws: WebSocket;
  readonly #proc: { kill(): void; exited: Promise<number> };
  readonly #profile: string;
  readonly #pending = new Map<number, Pending>();
  readonly #listeners = new Set<(method: string, params: Record<string, unknown>) => void>();
  #next = 1;

  private constructor(ws: WebSocket, proc: { kill(): void; exited: Promise<number> }, profile: string) {
    this.#ws = ws;
    this.#proc = proc;
    this.#profile = profile;
    ws.addEventListener("message", (e) => {
      const m = JSON.parse(String((e as MessageEvent).data)) as { id?: number; type: string; result?: unknown; error?: string; message?: string; method?: string; params?: Record<string, unknown> };
      if (m.id !== undefined) {
        const p = this.#pending.get(m.id);
        this.#pending.delete(m.id);
        if (m.type === "error") p?.reject(new Error(`${m.error}: ${m.message}`));
        else p?.resolve(m.result);
      } else if (m.method) for (const l of this.#listeners) l(m.method, m.params ?? {});
    });
  }

  /** `headless` false opens a window (WebGPU needs one). */
  static async launch(o: { firefox?: string; headless?: boolean } = {}): Promise<Firefox> {
    const firefox = o.firefox ?? firefoxPath();
    // og-cdp-: the prefix every harness browser's profile has, for the same clean-up.
    const profile = testTempDir("og-cdp-firefox-");
    let proc: ReturnType<typeof Bun.spawn> | undefined;
    try {
      writeFileSync(
        join(profile, "user.js"),
        [
          ["browser.shell.checkDefaultBrowser", false],
          ["browser.aboutwelcome.enabled", false],
          ["datareporting.policy.dataSubmissionEnabled", false],
          ["app.update.disabledForTesting", true],
        ]
          .map(([k, v]) => `user_pref(${JSON.stringify(k)}, ${JSON.stringify(v)});`)
          .join("\n"),
      );
      proc = Bun.spawn(
        [firefox, ...(o.headless === false ? [] : ["--headless"]), "--no-remote", "--profile", profile, "--remote-debugging-port=0", "about:blank"],
        { stdout: "ignore", stderr: "ignore" },
      );
      // Firefox writes the BiDi server's address into the profile once it listens.
      const info = join(profile, "WebDriverBiDiServer.json");
      const t0 = performance.now();
      let url = "";
      while (!url) {
        if (performance.now() - t0 > 30_000) throw new Error("Firefox did not open its WebDriver BiDi port in 30 s");
        await new Promise((r) => setTimeout(r, 50));
        try {
          const j = JSON.parse(readFileSync(info, "utf8")) as { ws_host: string; ws_port: number };
          url = `ws://${j.ws_host}:${j.ws_port}/session`;
        } catch {}
      }
      const ws = new WebSocket(url);
      await new Promise<void>((resolve, reject) => {
        ws.addEventListener("open", () => resolve());
        ws.addEventListener("error", () => reject(new Error("cannot connect to Firefox's WebDriver BiDi socket")));
      });
      const browser = new Firefox(ws, proc, profile);
      await browser.send("session.new", { capabilities: {} });
      await browser.send("session.subscribe", { events: ["log.entryAdded"] });
      return browser;
    } catch (error) {
      if (proc) { proc.kill(); await proc.exited; }
      rmSync(profile, { recursive: true, force: true });
      throw error;
    }
  }

  send<T = Record<string, unknown>>(method: string, params: Record<string, unknown> = {}): Promise<T> {
    const id = this.#next++;
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.#ws.send(JSON.stringify({ id, method, params }));
    });
  }

  on(fn: (method: string, params: Record<string, unknown>) => void): () => void {
    this.#listeners.add(fn);
    return () => this.#listeners.delete(fn);
  }

  /** A new tab at `url`; console messages and exceptions go to `log`. */
  async open(url: string, log: string[] = []): Promise<FirefoxPage> {
    const { context } = await this.send<{ context: string }>("browsingContext.create", { type: "tab" });
    const page = new FirefoxPage(this, context, log);
    if (url !== "about:blank") await page.send("Page.navigate", { url });
    return page;
  }

  async close(): Promise<void> {
    const closed = await Promise.race([
      this.send("browser.close").then(() => this.#proc.exited).then(() => true, () => false),
      new Promise<boolean>((r) => setTimeout(() => r(false), 5000)),
    ]);
    try {
      this.#ws.close();
    } catch {
      /* already closed */
    }
    if (!closed) this.#proc.kill();
    await this.#proc.exited;
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

export class FirefoxPage {
  readonly #browser: Firefox;
  readonly context: string;
  readonly log: string[];

  constructor(browser: Firefox, context: string, log: string[]) {
    this.#browser = browser;
    this.context = context;
    this.log = log;
    browser.on((method, params) => {
      if (method !== "log.entryAdded" || (params["source"] as { context?: string } | undefined)?.context !== context) return;
      const text = String(params["text"] ?? "");
      // cdp.ts's lines: "<console type>: ..." and "exception: ..." (reference-gfx.ts fails a scene on the latter).
      log.push(params["type"] === "javascript" ? `exception: ${text}` : `${params["method"] ?? params["level"]}: ${text}`);
    });
  }

  /** The CDP commands the browser tests send to a page, in BiDi. */
  async send(method: string, params: Record<string, unknown> = {}): Promise<void> {
    if (method === "Emulation.setDeviceMetricsOverride") {
      await this.#browser.send("browsingContext.setViewport", {
        context: this.context,
        viewport: { width: params["width"], height: params["height"] },
        devicePixelRatio: params["deviceScaleFactor"] ?? 1,
      });
    } else if (method === "Page.navigate") {
      await this.#browser.send("browsingContext.navigate", { context: this.context, url: params["url"], wait: "none" });
    } else if (method === "Page.reload") {
      await this.#browser.send("browsingContext.reload", { context: this.context, wait: "none" });
    } else if (method === "Page.close") {
      await this.#browser.send("browsingContext.close", { context: this.context });
    } else throw new Error(`bidi.ts: no ${method} (cdp.ts's Page has it)`);
  }

  /** Evaluate an expression (awaiting a Promise) in the page, or in a frame's context; the value goes through JSON. */
  async eval<T = unknown>(expression: string, context = this.context): Promise<T> {
    const r = await this.#browser.send<{ type: string; result?: Remote; exceptionDetails?: { text: string } }>("script.evaluate", {
      expression: `(async () => JSON.stringify(await (${expression})))()`,
      target: { context },
      awaitPromise: true,
      resultOwnership: "none",
    });
    if (r.type === "exception") throw new Error(r.exceptionDetails?.text ?? "exception");
    return (r.result?.type === "string" ? JSON.parse(r.result.value as string) : undefined) as T;
  }

  /** Wait until `expression` is truthy (polled), and answer its value (`true` where it does not go through JSON). */
  async until<T = true>(expression: string, timeoutMs = 30_000): Promise<T> {
    const t0 = performance.now();
    for (;;) {
      try {
        const v = await this.eval<T | null>(`(async () => { const v = await (${expression}); if (!v) return null; try { return JSON.stringify(v) === undefined ? true : v; } catch { return true; } })()`);
        if (v) return v;
      } catch {
        /* the page is still loading */
      }
      if (performance.now() - t0 > timeoutMs) throw new Error(`timed out after ${timeoutMs} ms waiting for ${expression}`);
      await new Promise((r) => setTimeout(r, 50));
    }
  }

  /** Evaluate in the child frame whose URL contains `part` (cross-origin frames too: BiDi reaches every context). */
  async evalChild<T = unknown>(part: string, expression: string): Promise<T> {
    const { contexts } = await this.#browser.send<{ contexts: Array<{ context: string; url: string; children?: unknown[] }> }>("browsingContext.getTree", { root: this.context });
    type Node = { context: string; url: string; children?: Node[] };
    const all: Node[] = [];
    const walk = (n: Node) => {
      all.push(n);
      for (const c of n.children ?? []) walk(c);
    };
    for (const c of contexts as Node[]) for (const k of c.children ?? []) walk(k);
    const child = all.find((n) => n.url.includes(part));
    if (!child) throw new Error(`no frame at ${part} (frames: ${all.map((n) => n.url).join(", ") || "none"})`);
    return this.eval<T>(expression, child.context);
  }

  /**
   * The frame an input action goes to: Firefox dispatches a top-level
   * context's actions in that document only, never into a cross-origin
   * iframe (out of process), so each action goes to the frame that would
   * receive it, the one under the point (`at`, with the point in its own
   * coordinates) or the one with focus (no `at`), descending through iframes.
   */
  async #inputTarget(at?: { x: number; y: number }): Promise<{ context: string; x: number; y: number }> {
    type Node = { context: string; url: string; children?: Node[] };
    const { contexts } = await this.#browser.send<{ contexts: Node[] }>("browsingContext.getTree", { root: this.context });
    let node = contexts[0]!;
    let x = at?.x ?? 0, y = at?.y ?? 0;
    for (;;) {
      const kids = node.children ?? [];
      if (kids.length === 0) break;
      const frames = await this.eval<Array<{ src: string; left: number; top: number; right: number; bottom: number; focused: boolean }>>(
        `[...document.querySelectorAll("iframe")].map((f) => { const r = f.getBoundingClientRect(); return { src: f.src, left: r.left + f.clientLeft, top: r.top + f.clientTop, right: r.right, bottom: r.bottom, focused: document.activeElement === f }; })`,
        node.context,
      );
      const hit = frames.find((f) => (at ? x >= f.left && x < f.right && y >= f.top && y < f.bottom : f.focused));
      const kid = hit && kids.find((k) => k.url.split("#")[0] === hit.src.split("#")[0]);
      if (!hit || !kid) break;
      x -= hit.left;
      y -= hit.top;
      node = kid;
    }
    return { context: node.context, x: Math.round(x), y: Math.round(y) };
  }

  async #pointer(pointerType: "touch" | "mouse", x: number, y: number): Promise<void> {
    const t = await this.#inputTarget({ x, y });
    await this.#browser.send("input.performActions", {
      context: t.context,
      actions: [{ type: "pointer", id: pointerType, parameters: { pointerType }, actions: [{ type: "pointerMove", x: t.x, y: t.y }, { type: "pointerDown", button: 0 }, { type: "pointerUp", button: 0 }] }],
    });
  }

  /** A tap (touch) at CSS pixel x, y of the page. */
  tap(x: number, y: number): Promise<void> {
    return this.#pointer("touch", x, y);
  }

  click(x: number, y: number): Promise<void> {
    return this.#pointer("mouse", x, y);
  }

  /**
   * A key press, named as cdp.ts names it (`code`, `key`). WebDriver sends a
   * key by its value and the browser derives the code from a US layout, so
   * a named key goes as WebDriver's code point for it.
   */
  async key(code: string, key = code): Promise<void> {
    const value = WEBDRIVER_KEYS[key] ?? (key.length === 1 ? key : undefined);
    if (value === undefined) throw new Error(`bidi.ts: no WebDriver value for key ${key} (code ${code})`);
    await this.#browser.send("input.performActions", {
      context: (await this.#inputTarget()).context,
      actions: [{ type: "key", id: "keyboard", actions: [{ type: "keyDown", value }, { type: "keyUp", value }] }],
    });
  }
}

/** WebDriver's code points for the named keys the tests press (the WebDriver spec's keyboard table). */
const WEBDRIVER_KEYS: Record<string, string> = {
  Enter: "\uE007",
  Escape: "\uE00C",
  Tab: "\uE004",
  Backspace: "\uE003",
  ArrowLeft: "\uE012",
  ArrowUp: "\uE013",
  ArrowRight: "\uE014",
  ArrowDown: "\uE015",
  Shift: "\uE008",
  Control: "\uE009",
};
