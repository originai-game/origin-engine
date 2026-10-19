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

import { createRing, writeRing, type InjectedInput } from "./input.ts";
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
  /** The engine failed after it was ready (a trap, a failing system). */
  readonly onCrash?: (message: string) => void;
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

/** Keys the page would act on itself (scroll, focus); the shell keeps them for the game. */
const KEPT_KEYS = new Set(["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Tab", "PageUp", "PageDown", "Home", "End", "Backspace"]);

/** What the Worker reported when the engine was ready. */
export interface Ready {
  readonly tier: string;
  readonly transport: "ring" | "message";
  readonly rpc: "ring" | "message";
  /** The GPU texture format this device would take (`rgba8` is uploaded today), or null without a bundle. */
  readonly textureFormat: string | null;
  readonly files: number;
  readonly textures: number;
}

const BUTTONS = ["left", "middle", "right"] as const;

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
        this.#onCrash?.(m.message ?? "the engine stopped");
        return;
      }
      if (m.type === "frame") {
        this.#onFirstFrame?.();
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
    const ready = new Promise<Ready>((resolve, reject) => {
      const on = (e: MessageEvent) => {
        const m = e.data as { type: string; value?: number; stage?: string; message?: string } & Partial<Ready>;
        if (m.type === "progress") options.onProgress?.(m.value ?? 0, m.stage ?? "");
        else if (m.type === "ready") {
          worker.removeEventListener("message", on);
          resolve({
            tier: m.tier ?? "none",
            transport: m.transport ?? "message",
            rpc: m.rpc ?? "message",
            textureFormat: m.textureFormat ?? null,
            files: m.files ?? 0,
            textures: m.textures ?? 0,
          });
        } else if (m.type === "error") {
          worker.removeEventListener("message", on);
          reject(new Error(m.message));
        }
      };
      worker.addEventListener("message", on);
    });
    worker.postMessage(
      {
        type: "start",
        glue: options.glue,
        wasm: options.wasm,
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
        viewport: { width: canvas.clientWidth || canvas.width / dpr, height: canvas.clientHeight || canvas.height / dpr, dpr, coarsePointer: globalThis.matchMedia?.("(pointer: coarse)").matches ?? false },
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
      while (this.#rpcPending.size > 0) {
        await waitForFrames(this.#rpc.answers, 100);
        for (const frame of readFrames(this.#rpc.answers)) this.#answer(frame);
      }
    } finally {
      this.#pumping = false;
    }
  }

  /** Call the engine in the Worker: `advance`, `command`, `hash`, `frame`, or `probe` with a member name. */
  call(method: string, ...args: unknown[]): Promise<unknown> {
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

  /** Start or stop ticking with the display. */
  play(on: boolean): void {
    this.#playing = on;
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
    this.worker.postMessage({ type: "resize", width: size.width, height: size.height, viewport });
    return size;
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
    const typing = (e: KeyboardEvent) => {
      const t = e.target as { tagName?: string; isContentEditable?: boolean } | null;
      return !!t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable === true);
    };
    listen(globalThis as unknown as EventTarget, "keydown", (e) => {
      if (typing(e) || !e.code) return;
      if (KEPT_KEYS.has(e.code)) e.preventDefault();
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

  /** Remove the page listeners and stop the Worker. */
  dispose(): void {
    for (const off of this.#listeners.splice(0)) off();
    this.worker.terminate();
  }
}
