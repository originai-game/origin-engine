// Firefox over WebDriver BiDi, for the reference runners (reference.ts,
// reference-gfx.ts) with --firefox: the part of cdp.ts's Browser and Page
// they use (open a page, set its viewport, navigate, evaluate, the console
// in `log`, close). Firefox has no CDP since 141. No dependency beyond bun's
// WebSocket.
//
// Firefox: --firefox <path>, $FIREFOX, or the default install. A missing
// Firefox is an error, never a skip. Headless Firefox has no WebGPU adapter
// (157 on Windows), so a WebGPU run opens a window; WebGL2 runs headless.

import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

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
    // og-cdp-: the prefix every harness browser's profile has, for the same clean-up.
    const profile = mkdtempSync(join(tmpdir(), "og-cdp-firefox-"));
    writeFileSync(
      join(profile, "user.js"),
      [
        ["browser.shell.checkDefaultBrowser", false],
        ["browser.aboutwelcome.enabled", false],
        ["datareporting.policy.dataSubmissionEnabled", false],
        ["app.update.disabledForTesting", true],
        // The adapter's own name in the run's renderer line, not a bucketed one.
        ["webgl.sanitize-unmasked-renderer", false],
      ]
        .map(([k, v]) => `user_pref(${JSON.stringify(k)}, ${JSON.stringify(v)});`)
        .join("\n"),
    );
    const proc = Bun.spawn(
      [o.firefox ?? firefoxPath(), ...(o.headless === false ? [] : ["--headless"]), "--no-remote", "--profile", profile, "--remote-debugging-port=0", "about:blank"],
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

  /** The two CDP commands the reference runners send to a page, in BiDi. */
  async send(method: string, params: Record<string, unknown> = {}): Promise<void> {
    if (method === "Emulation.setDeviceMetricsOverride") {
      await this.#browser.send("browsingContext.setViewport", {
        context: this.context,
        viewport: { width: params["width"], height: params["height"] },
        devicePixelRatio: params["deviceScaleFactor"] ?? 1,
      });
    } else if (method === "Page.navigate") {
      await this.#browser.send("browsingContext.navigate", { context: this.context, url: params["url"], wait: "none" });
    } else throw new Error(`bidi.ts: no ${method} (cdp.ts's Page has it)`);
  }

  /** Evaluate an expression (awaiting a Promise) in the page; the value goes through JSON. */
  async eval<T = unknown>(expression: string): Promise<T> {
    const r = await this.#browser.send<{ type: string; result?: Remote; exceptionDetails?: { text: string } }>("script.evaluate", {
      expression: `(async () => JSON.stringify(await (${expression})))()`,
      target: { context: this.context },
      awaitPromise: true,
      resultOwnership: "none",
    });
    if (r.type === "exception") throw new Error(r.exceptionDetails?.text ?? "exception");
    return (r.result?.type === "string" ? JSON.parse(r.result.value as string) : undefined) as T;
  }

  /** Wait until `expression` is truthy (polled; the value itself need not go through JSON). */
  async until(expression: string, timeoutMs = 30_000): Promise<true> {
    const t0 = performance.now();
    for (;;) {
      try {
        if (await this.eval<boolean>(`!!(${expression})`)) return true;
      } catch {
        /* the page is still loading */
      }
      if (performance.now() - t0 > timeoutMs) throw new Error(`timed out after ${timeoutMs} ms waiting for ${expression}`);
      await new Promise((r) => setTimeout(r, 50));
    }
  }
}
