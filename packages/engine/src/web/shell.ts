// The main-thread shell (docs/design.md §2.2): it owns the page, transfers
// the canvas to the engine Worker as an OffscreenCanvas, and pumps input into
// the SharedArrayBuffer ring when the page is cross-origin isolated (falling
// back to postMessage otherwise). It never runs game code.
//
// Input: keyboard (KeyboardEvent.code; the keys that would scroll or move
// focus are kept from the page), pointer events of every type (mouse, pen,
// touch), up to five at once: each pressed pointer gets the lowest free
// engine id (0, the primary pointer og.Input follows, while none is held;
// then 1 to 4), which og.Pointers reports with its position and press
// point, so a stick and a button can be held together. Gamepads are polled
// on the display frame (gamepad.ts) and delivered both as og.Gamepad states
// and, for games that only read keys, as the keys of their standard mapping.
//
// Accessibility: the UI's accessibility tree is mirrored into the page over
// the canvas (a11y.ts), and the page's `prefers-reduced-motion: reduce`,
// `prefers-contrast: more` and `prefers-reduced-transparency: reduce` reach
// the UI with the viewport (og-ui's theme takes them over the game's
// settings; glass becomes an opaque surface), followed as they change.

import { createRing, writeRing, type InjectedInput } from "./input.ts";
import { TextEntry, type TextTarget } from "./text-input.ts";
import { A11yMirror, type A11yTree } from "./a11y.ts";
import { laneChoice } from "./lanes.ts";
import { createRpcRings, readFrames, waitForFrames, writeFrame, type RpcRings } from "./rpc-ring.ts";
import { localPlatform, storagePersistence, type PlatformCall, type PlatformHost } from "../platform-local.ts";

export interface ShellOptions {
  /** The play surface. Its pixel size is set from its CSS size and the device pixel ratio. */
  readonly canvas: HTMLCanvasElement;
  /** The game's engine Worker (a module Worker whose entry calls startWorker). */
  readonly worker: Worker;
  /** URLs of the wasm-bindgen glue (origin-engine.js) and module (origin-engine_bg.wasm), as the Worker resolves them. */
  readonly glue: string;
  readonly wasm: string;
  /**
   * The lanes variant's glue and module (origin-engine-lanes.js and
   * origin-engine-lanes_bg.wasm) and its size in bytes, when the build ships
   * one: the shell runs it when the page is cross-origin isolated and the
   * device has the cores for it (laneChoice).
   */
  readonly lanes?: { readonly glue: string; readonly wasm: string; readonly bytes?: number };
  /** Use the WebGL2 fallback instead of WebGPU. */
  readonly webgl?: boolean;
  /** A replay seed; omitted is GameDesc.seed. */
  readonly seed?: number;
  /** Tick and render with the display (default true); false leaves ticks to `call("advance", n)`. */
  readonly autoplay?: boolean;
  /** The URL base (as the Worker resolves it) whose `cooked/` bundle the Worker fetches before the engine starts. */
  readonly root?: string;
  /** The Basis transcoder's script and module URLs (as the Worker resolves them). */
  readonly basis?: { readonly js: string; readonly wasm: string };
  /** The project's physics profile (project.ogt `profile`); omitted is 3d. */
  readonly profile?: "2d" | "3d";
  /** Boot progress, 0 to 1, with the stage (`wasm`, `bundle`, `fetch`, `transcode`, `ready`). */
  readonly onProgress?: (value: number, stage: string) => void;
  /** The highest device pixel ratio the canvas is drawn at (default 2). */
  readonly dprCap?: number;
  /** Bytes the boot fetches (the wasm, the cooked bundle), so progress is the bytes received. */
  readonly expect?: { readonly wasm?: number; readonly cooked?: number };
  /** Lines of the engine's log and the Worker's console. */
  readonly onLog?: (lines: readonly string[]) => void;
  /** The engine failed after it was ready (a trap, a Rust panic, a failing system, a hang): the line to show and what it was. */
  readonly onCrash?: (message: string, error: HostError) => void;
  /** The first frame was rendered. */
  readonly onFirstFrame?: () => void;
  /** Without autoplay, take one tick before the first frame (the first screen behind a play gate). */
  readonly prime?: boolean;
  /** The locale og.i18n is asked for; default: `navigator.language`. */
  readonly locale?: string;
  /** What answers og.platform's calls; default: the browser's own storage (`setPlatform` changes it, a Player bridge for instance). */
  readonly platform?: PlatformHost;
}

/** The pixel size of a canvas laid out at `cssWidth`×`cssHeight` on this display, with the DPR cap. */
export function pixelSize(cssWidth: number, cssHeight: number, devicePixelRatio: number, dprCap = 2): { width: number; height: number; dpr: number } {
  const dpr = Math.max(0.5, Math.min(devicePixelRatio || 1, dprCap > 0 ? dprCap : 2));
  return { width: Math.max(1, Math.round(cssWidth * dpr)), height: Math.max(1, Math.round(cssHeight * dpr)), dpr };
}

/** The page's preferences og-ui follows, each with the viewport field it sets. */
export const PAGE_PREFERENCES = [
  ["reducedMotion", "(prefers-reduced-motion: reduce)"],
  ["highContrast", "(prefers-contrast: more)"],
  ["reducedTransparency", "(prefers-reduced-transparency: reduce)"],
] as const;

/** The page's motion, contrast and transparency preferences, as the viewport carries them to og-ui. */
export function pagePreferences(): { reducedMotion: boolean; highContrast: boolean; reducedTransparency: boolean } {
  const query = (q: string) => globalThis.matchMedia?.(q).matches ?? false;
  const [motion, contrast, transparency] = PAGE_PREFERENCES.map(([, q]) => query(q));
  return { reducedMotion: motion!, highContrast: contrast!, reducedTransparency: transparency! };
}

/** Keys the page would act on itself (scroll, focus); the shell keeps them for the game. */
const KEPT_KEYS = new Set(["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Tab", "PageUp", "PageDown", "Home", "End", "Backspace"]);

export { LANES_MAX_THREADS, laneChoice } from "./lanes.ts";

/**
 * Why the engine stopped, as the Worker or the shell's frame deadline
 * reported it (docs/web-export.md §Errors): `panic`, a Rust panic on the
 * engine's thread or inside a lane (`lane` names the lanes site; the wasm
 * recorded it before it trapped); `error`, a failing system, a trap without
 * a panic or an uncaught error; `hang`, a playing engine Worker that sent
 * no frame for FRAME_DEADLINE_MS.
 */
export interface HostError {
  readonly kind: "panic" | "error" | "hang";
  readonly message: string;
  /** The tick the world was running, when known. */
  readonly tick: number | null;
  /** The lanes site a panic happened in (`anim`, `extract`, `cull`, `physics`); null on the engine's thread. */
  readonly lane: string | null;
  /** A panic's `file:line:column`. */
  readonly location: string | null;
}

/**
 * How long a playing, visible page may go without a frame from the engine
 * Worker before the shell gives up on it (a Worker stuck in a loop, or in
 * `Atomics.wait` for a lane that never ends): it terminates the Worker and
 * reports a `hang`. A panicking lane is caught sooner, at the lane boundary.
 */
export const FRAME_DEADLINE_MS = 15_000;

/** What the Worker reported when the engine was ready. */
export interface Ready {
  /** The mode the engine got: lanes on with the pool's threads, or why it runs single-threaded. */
  readonly lanes: { readonly on: boolean; readonly threads: number; readonly why: string };
  readonly tier: string;
  readonly transport: "ring" | "message";
  readonly rpc: "ring" | "message";
  /** The encodings the bundle's Basis textures reached the renderer in (`bc7+bc5`, `astc4x4`, `etc2+eacrg11`, `rgba8`), or null without a bundle. */
  readonly textureFormat: string | null;
  readonly files: number;
  readonly textures: number;
}

const BUTTONS = ["left", "middle", "right"] as const;

/**
 * How long the boot may go without a word from the Worker (progress, a log
 * line) before it fails: generous, since a slow network moves the progress
 * and only a hung Worker says nothing for this long.
 */
export const BOOT_DEADLINE_MS = 120_000;

export class Shell {
  readonly worker: Worker;
  readonly tier: string;
  /** `ring` when input crosses through the SharedArrayBuffer, `message` otherwise. */
  readonly transport: "ring" | "message";
  /** What the Worker reported at start. */
  readonly ready: Ready;
  readonly #pending = new Map<number, { resolve(v: unknown): void; reject(e: Error): void }>();
  readonly #rpcPending = new Map<number, { resolve(v: unknown): void; reject(e: Error): void }>();
  readonly #listeners: Array<() => void> = [];
  readonly #rpc: RpcRings | undefined;
  #onLog: ShellOptions["onLog"];
  #onCrash: ShellOptions["onCrash"];
  #onFirstFrame: ShellOptions["onFirstFrame"];
  #next = 1;
  #rpcNext = 1;
  #pumping = false;
  #ring: SharedArrayBuffer | undefined;
  #dprCap = 2;
  /** Engine pointer ids (0 to 4) of the page pointers held down, by PointerEvent.pointerId. */
  #pointers = new Map<number, number>();
  #playing = false;
  /** Why the engine no longer answers (it crashed, or the shell was disposed); every call after it fails with it. */
  #stopped: Error | null = null;
  /** The hidden textarea that takes the page's text for the UI's focused field (text-input.ts). */
  #text: TextEntry | undefined;
  /** The DOM mirror of the UI's accessibility tree (a11y.ts). */
  #a11y: A11yMirror | undefined;
  /** The viewport last sent to the Worker (the page's preferences are re-read into it when they change). */
  #viewport: Record<string, unknown> = {};
  /** What stopped the engine, once something did. */
  #error: HostError | null = null;
  /** When the Worker last said it plays (`beat`), and the tick it reached. */
  #beat = 0;
  #beatTick: number | null = null;
  /** When the deadline was last looked at: a late look means the page itself was blocked. */
  #watched = 0;
  #watchdog: ReturnType<typeof setInterval> | undefined;
  #platform: PlatformHost = localPlatform(storagePersistence(`og.platform:${globalThis.location?.pathname ?? "/"}`));

  private constructor(worker: Worker, ready: Ready, rpc: RpcRings | undefined) {
    this.worker = worker;
    this.tier = ready.tier;
    this.transport = ready.transport;
    this.ready = ready;
    this.#rpc = rpc;
    worker.addEventListener("message", (e: MessageEvent) => {
      const m = e.data as { type: string; id?: number; value?: unknown; error?: string; frame?: string; lines?: string[]; message?: string };
      if (m.type === "log" && m.lines) {
        this.#onLog?.(m.lines);
        return;
      }
      if (m.type === "crash") {
        const message = m.message ?? "the engine stopped";
        const error = (m as unknown as { error?: HostError }).error ?? { kind: "error", message, tick: this.#beatTick, lane: null, location: null };
        this.#crashed(message, error);
        return;
      }
      if (m.type === "beat") {
        this.#beat = performance.now();
        this.#beatTick = (m as { tick?: number | null }).tick ?? this.#beatTick;
        return;
      }
      if (m.type === "frame") {
        this.#onFirstFrame?.();
        return;
      }
      if (m.type === "text") {
        this.#text?.update((m as unknown as { target: TextTarget | null }).target);
        return;
      }
      if (m.type === "copy") {
        this.#text?.copied((m as unknown as { text: string }).text);
        return;
      }
      if (m.type === "a11y") {
        this.#a11y?.update((m as unknown as { tree: A11yTree }).tree);
        return;
      }
      if (m.type === "rpc" && m.frame !== undefined) {
        this.#answer(m.frame);
        return;
      }
      if (m.type === "platform") {
        const calls = (m as unknown as { calls: PlatformCall[] }).calls ?? [];
        this.#platform.handle(calls, (request, answer) => worker.postMessage({ type: "platform-answer", request, answer }));
        return;
      }
      if (m.type !== "result" || m.id === undefined) return;
      const p = this.#pending.get(m.id);
      this.#pending.delete(m.id);
      if (m.error !== undefined) p?.reject(new Error(m.error));
      else p?.resolve(m.value);
    });
  }

  /** Start the engine in the Worker and wire the page's input to it. */
  static async start(options: ShellOptions): Promise<Shell> {
    const { canvas, worker } = options;
    const size = pixelSize(canvas.clientWidth || 1, canvas.clientHeight || 1, globalThis.devicePixelRatio || 1, options.dprCap);
    const dpr = size.dpr;
    canvas.width = size.width;
    canvas.height = size.height;
    // Logs and a crash during the boot reach the page as well as after it.
    const early = (e: MessageEvent) => {
      const m = e.data as { type: string; lines?: string[] };
      if (m.type === "log" && m.lines) options.onLog?.(m.lines);
    };
    worker.addEventListener("message", early);
    const offscreen = canvas.transferControlToOffscreen();
    const ring = globalThis.crossOriginIsolated ? createRing() : undefined;
    const rpc = globalThis.crossOriginIsolated ? createRpcRings() : undefined;
    // The boot fails (and the Worker stops) when the Worker says so, when it
    // does not load or throws while loading (game.js missing, a syntax error,
    // a top-level throw: an `error` event and no message), when its message
    // cannot be read, or when it says nothing for BOOT_DEADLINE_MS.
    const ready = new Promise<Ready>((resolve, reject) => {
      let deadline: ReturnType<typeof setTimeout> | undefined;
      const settle = () => {
        clearTimeout(deadline);
        worker.removeEventListener("message", on);
        worker.removeEventListener("error", onError);
        worker.removeEventListener("messageerror", onMessageError);
      };
      const failed = (message: string, error?: HostError) => {
        settle();
        worker.terminate();
        reject(Object.assign(new Error(message), { error: error ?? { kind: "error", message, tick: null, lane: null, location: null } }));
      };
      const arm = () => {
        clearTimeout(deadline);
        deadline = setTimeout(() => failed(`the engine Worker said nothing for ${BOOT_DEADLINE_MS / 1000} s while starting`), BOOT_DEADLINE_MS);
      };
      const onError = (e: Event) => {
        const message = (e as ErrorEvent).message;
        failed(message ? `the engine Worker failed: ${message}` : "the engine Worker did not load (game.js is missing, or it failed while loading)");
      };
      const onMessageError = () => failed("a message from the engine Worker could not be read");
      const on = (e: MessageEvent) => {
        arm();
        const m = e.data as { type: string; value?: number; stage?: string; message?: string } & Partial<Ready>;
        if (m.type === "progress") options.onProgress?.(m.value ?? 0, m.stage ?? "");
        else if (m.type === "ready") {
          settle();
          resolve({
            lanes: m.lanes ?? { on: false, threads: 1, why: "the Worker did not report lanes" },
            tier: m.tier ?? "none",
            transport: m.transport ?? "message",
            rpc: m.rpc ?? "message",
            textureFormat: m.textureFormat ?? null,
            files: m.files ?? 0,
            textures: m.textures ?? 0,
          });
        } else if (m.type === "error") failed(m.message ?? "the engine did not start", (m as { error?: HostError }).error);
      };
      worker.addEventListener("message", on);
      worker.addEventListener("error", onError);
      worker.addEventListener("messageerror", onMessageError);
      arm();
    });
    const choice = laneChoice(!!options.lanes, !!globalThis.crossOriginIsolated, globalThis.navigator?.hardwareConcurrency ?? 0);
    worker.postMessage(
      {
        type: "start",
        glue: options.glue,
        wasm: options.wasm,
        ...("threads" in choice ? { lanes: { ...options.lanes!, threads: choice.threads } } : { lanesWhy: choice.why }),
        canvas: offscreen,
        webgl: options.webgl ?? false,
        seed: options.seed,
        ring,
        rpc,
        autoplay: options.autoplay ?? true,
        root: options.root,
        basis: options.basis,
        profile: options.profile,
        locale: options.locale ?? globalThis.navigator?.language,
        ...(options.expect ? { expect: options.expect } : {}),
        ...(options.prime ? { prime: true } : {}),
        viewport: { width: canvas.clientWidth || canvas.width / dpr, height: canvas.clientHeight || canvas.height / dpr, dpr, coarsePointer: globalThis.matchMedia?.("(pointer: coarse)").matches ?? false, ...pagePreferences() },
      },
      [offscreen],
    );
    let answered: Ready;
    try {
      answered = await ready;
    } finally {
      worker.removeEventListener("message", early);
    }
    const shell = new Shell(worker, answered, rpc);
    shell.#ring = ring;
    shell.#dprCap = options.dprCap ?? 2;
    shell.#onLog = options.onLog;
    shell.#onCrash = options.onCrash;
    shell.#onFirstFrame = options.onFirstFrame;
    shell.#playing = options.autoplay ?? true;
    shell.#viewport = { width: canvas.clientWidth || canvas.width / dpr, height: canvas.clientHeight || canvas.height / dpr, dpr, coarsePointer: globalThis.matchMedia?.("(pointer: coarse)").matches ?? false };
    shell.#beat = shell.#watched = performance.now();
    shell.#watchdog = setInterval(() => shell.#watch(), 1000);
    if (options.platform) shell.setPlatform(options.platform);
    else shell.setPlatform(shell.#platform);
    shell.#wire(canvas, ring);
    return shell;
  }

  /**
   * Call one probe member (JSON-RPC 2.0) through the request ring when the
   * page is cross-origin isolated, else through postMessage. Resolves with
   * the member's result (a protocol-5 refusal is `false` or `null`); rejects
   * with the JSON-RPC error's message, its `code` on the error. A `capture` resolves once its
   * readback has landed (the Worker retries it on the following frames).
   */
  probe(method: string, params: Record<string, unknown> = {}): Promise<unknown> {
    if (this.#stopped) return Promise.reject(this.#stopped);
    const id = this.#rpcNext++;
    const frame = JSON.stringify({ jsonrpc: "2.0", id, method, params });
    return new Promise((resolve, reject) => {
      this.#rpcPending.set(id, { resolve, reject });
      if (this.#rpc && writeFrame(this.#rpc.requests, frame)) void this.#pump();
      else this.worker.postMessage({ type: "rpc", frame });
    });
  }

  #answer(frame: string): void {
    const m = JSON.parse(frame) as { id?: number; result?: unknown; error?: { code?: number; message?: string } };
    const p = typeof m.id === "number" ? this.#rpcPending.get(m.id) : undefined;
    if (!p) return;
    this.#rpcPending.delete(m.id!);
    // The JSON-RPC code rides on the error: -32601 is a member this build does not serve.
    if (m.error) p.reject(Object.assign(new Error(m.error.message ?? "probe error"), { code: m.error.code ?? -32603 }));
    else p.resolve(m.result);
  }

  /** Read answers from the ring while calls are outstanding. */
  async #pump(): Promise<void> {
    if (this.#pumping || !this.#rpc) return;
    this.#pumping = true;
    try {
      while (this.#rpcPending.size > 0 && !this.#stopped) {
        await waitForFrames(this.#rpc.answers, 100);
        for (const frame of readFrames(this.#rpc.answers)) this.#answer(frame);
      }
    } finally {
      this.#pumping = false;
    }
  }

  /** Call the engine in the Worker: `advance`, `command`, `hash`, `frame`, or `probe` with a member name. */
  call(method: string, ...args: unknown[]): Promise<unknown> {
    if (this.#stopped) return Promise.reject(this.#stopped);
    const id = this.#next++;
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve, reject });
      this.worker.postMessage({ type: "call", id, method, args });
    });
  }

  /** What answers og.platform's calls from now on (og.platform.info reports its host and capabilities). */
  setPlatform(host: PlatformHost): void {
    this.#platform = host;
    this.worker.postMessage({ type: "platform-host", host: host.host, capabilities: host.capabilities });
  }

  /** What stopped the engine (a panic, an error, a hang), or null while it runs. */
  get error(): HostError | null {
    return this.#error;
  }

  /** The engine failed: nothing waits for an answer it will not give, and the page hears why once. */
  #crashed(message: string, error: HostError): void {
    if (this.#error) return;
    this.#error = error;
    this.#stop(`the engine stopped: ${message}`);
    clearInterval(this.#watchdog);
    this.#onCrash?.(message, error);
  }

  /**
   * The frame deadline: a playing page whose Worker sent no `beat` for
   * FRAME_DEADLINE_MS is stuck; the Worker is terminated and the page told.
   * A hidden page (no display frames) and a paused clock do not count.
   */
  #watch(): void {
    const now = performance.now();
    const late = now - this.#watched > 3000;
    this.#watched = now;
    const hidden = globalThis.document?.visibilityState === "hidden";
    // A page that was blocked or throttled itself has not been hearing the Worker: start over.
    if (!this.#playing || hidden || this.#stopped || late) {
      this.#beat = now;
      return;
    }
    if (now - this.#beat < FRAME_DEADLINE_MS) return;
    const tick = this.#beatTick;
    const message = `the engine Worker sent no frame for ${FRAME_DEADLINE_MS / 1000} s${tick === null ? "" : ` (last at tick ${tick})`}: it is stuck, so it was stopped`;
    this.#crashed(message, { kind: "hang", message, tick, lane: null, location: null });
    this.worker.terminate();
  }

  /** Start or stop ticking with the display. */
  play(on: boolean): void {
    this.#playing = on;
    this.#beat = performance.now();
    this.worker.postMessage({ type: "play", on });
  }

  /** Whether the clock ticks with the display (the last `play`). */
  get playing(): boolean {
    return this.#playing;
  }

  /**
   * The canvas was laid out at `cssWidth`×`cssHeight`: resize the render
   * target (device pixels, with the DPR cap) and tell the UI its viewport.
   */
  resize(cssWidth: number, cssHeight: number, safeArea?: Record<string, number>): { width: number; height: number; dpr: number } {
    const size = pixelSize(cssWidth, cssHeight, globalThis.devicePixelRatio || 1, this.#dprCap);
    const viewport = {
      width: cssWidth,
      height: cssHeight,
      dpr: size.dpr,
      ...(safeArea ? { safeArea } : {}),
      coarsePointer: globalThis.matchMedia?.("(pointer: coarse)").matches ?? false,
    };
    this.#viewport = viewport;
    this.worker.postMessage({ type: "resize", width: size.width, height: size.height, viewport: { ...viewport, ...pagePreferences() } });
    return size;
  }

  /** The DOM mirror of the UI's accessibility tree (a11y.ts). */
  get a11y(): A11yMirror | undefined {
    return this.#a11y;
  }

  /** Deliver one event as page input (the ring when isolated, else a message). */
  inject(event: InjectedInput): void {
    this.#send(this.#ring, event);
  }

  #send(ring: SharedArrayBuffer | undefined, event: InjectedInput): void {
    if (ring) writeRing(ring, event);
    else this.worker.postMessage({ type: "input", event });
  }

  #wire(canvas: HTMLCanvasElement, ring: SharedArrayBuffer | undefined): void {
    const listen = <K extends keyof HTMLElementEventMap>(target: EventTarget, type: K, fn: (e: HTMLElementEventMap[K]) => void) => {
      target.addEventListener(type, fn as EventListener, { passive: false });
      this.#listeners.push(() => target.removeEventListener(type, fn as EventListener));
    };
    // When an event happened on a clock the Worker shares (its `at` within the tick, input.ts stampAt).
    const stamp = (e: Event) => performance.timeOrigin + e.timeStamp;
    // The UI's text fields take the page's text through a hidden textarea
    // (text-input.ts); its keys still reach the engine, except an input
    // method's while it composes.
    const text = new TextEntry(canvas, (e) => this.#send(ring, e));
    this.#text = text;
    this.#listeners.push(() => text.dispose());
    // The accessibility tree over the canvas; what a screen reader does there reaches the UI as input.
    const mirror = new A11yMirror(canvas, (e) => this.#send(ring, e));
    this.#a11y = mirror;
    this.#listeners.push(() => mirror.dispose());
    // The page's motion, contrast and transparency preferences, followed as the reader changes them.
    for (const [, q] of PAGE_PREFERENCES) {
      const media = globalThis.matchMedia?.(q);
      if (!media?.addEventListener) continue;
      const changed = () => this.worker.postMessage({ type: "viewport", viewport: { ...this.#viewport, ...pagePreferences() } });
      media.addEventListener("change", changed);
      this.#listeners.push(() => media.removeEventListener("change", changed));
    }
    const typing = (e: KeyboardEvent) => {
      if (text.owns(e.target)) return e.isComposing || text.composing || e.keyCode === 229;
      // The mirror's elements (a slider's native range among them) pass their keys to the game.
      if (mirror.owns(e.target)) return false;
      const t = e.target as { tagName?: string; isContentEditable?: boolean } | null;
      return !!t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable === true);
    };
    listen(globalThis as unknown as EventTarget, "keydown", (e) => {
      if (typing(e) || !e.code) return;
      // In the text target Space types and shortcuts copy, cut and paste.
      const shortcut = text.owns(e.target) && (e.ctrlKey || e.metaKey || e.code === "Space");
      if (KEPT_KEYS.has(e.code) && !shortcut) e.preventDefault();
      if (!e.repeat) this.#send(ring, { kind: "key", key: e.code, action: "down", time: stamp(e) });
    });
    listen(globalThis as unknown as EventTarget, "keyup", (e) => {
      if (typing(e) || !e.code) return;
      this.#send(ring, { kind: "key", key: e.code, action: "up", time: stamp(e) });
    });
    // Relative to the canvas's CSS box, whichever element the event landed on.
    const at = (e: PointerEvent | WheelEvent) => {
      const r = canvas.getBoundingClientRect();
      return { x: e.clientX - r.left, y: e.clientY - r.top };
    };
    const point = (e: PointerEvent, action: "move" | "down" | "up", id: number) => {
      const p = at(e);
      this.#send(ring, {
        kind: "pointer",
        action,
        x: p.x,
        y: p.y,
        button: action === "move" ? "none" : (BUTTONS[e.button] ?? "left"),
        ...(id > 0 ? { id } : {}),
        time: stamp(e),
      });
    };
    listen(canvas, "pointerdown", (e) => {
      let id = this.#pointers.get(e.pointerId);
      if (id === undefined) {
        const taken = new Set(this.#pointers.values());
        id = [0, 1, 2, 3, 4].find((i) => !taken.has(i));
        if (id === undefined) return; // a sixth finger: ignored
        this.#pointers.set(e.pointerId, id);
      }
      try {
        canvas.setPointerCapture(e.pointerId);
      } catch {
        /* a synthetic event has no active pointer */
      }
      if (e.pointerType !== "mouse") e.preventDefault();
      point(e, "down", id);
    });
    listen(canvas, "pointermove", (e) => {
      const id = this.#pointers.get(e.pointerId);
      if (id !== undefined) point(e, "move", id);
      // A hovering mouse moves the primary pointer while nothing is held.
      else if (this.#pointers.size === 0 && e.pointerType === "mouse") point(e, "move", 0);
    });
    const end = (e: PointerEvent) => {
      const id = this.#pointers.get(e.pointerId);
      if (id === undefined) return;
      this.#pointers.delete(e.pointerId);
      point(e, "up", id);
    };
    listen(canvas, "pointerup", end);
    listen(canvas, "pointercancel", end);
    listen(canvas, "contextmenu", (e) => e.preventDefault());
    listen(canvas, "wheel", (e) => {
      e.preventDefault();
      const p = at(e);
      this.#send(ring, { kind: "wheel", x: p.x, y: p.y, deltaX: e.deltaX, deltaY: e.deltaY });
    });
  }

  /** The engine answers nothing more: every pending probe and call fails with `reason`, and so does every later one. */
  #stop(reason: string): void {
    if (this.#stopped) return;
    this.#stopped = Object.assign(new Error(reason), { code: -32603 });
    for (const pending of [this.#pending, this.#rpcPending]) {
      for (const p of pending.values()) p.reject(this.#stopped);
      pending.clear();
    }
  }

  /** Remove the page listeners and stop the Worker. */
  dispose(): void {
    clearInterval(this.#watchdog);
    this.#stop("the shell was disposed");
    for (const off of this.#listeners.splice(0)) off();
    this.worker.terminate();
  }
}
