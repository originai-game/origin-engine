// The main-thread shell (docs/design.md §2.2): it owns the page, transfers
// the canvas to the engine Worker as an OffscreenCanvas, and pumps input into
// the SharedArrayBuffer ring when the page is cross-origin isolated (falling
// back to postMessage otherwise). It never runs game code.

import { createRing, writeRing, type InjectedInput } from "./input.ts";
import { createRpcRings, readFrames, waitForFrames, writeFrame, type RpcRings } from "./rpc-ring.ts";

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
  /** Boot progress, 0 to 1, with the stage (`wasm`, `bundle`, `fetch`, `transcode`, `ready`). */
  readonly onProgress?: (value: number, stage: string) => void;
}

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
  #next = 1;
  #rpcNext = 1;
  #pumping = false;

  private constructor(worker: Worker, ready: Ready, rpc: RpcRings | undefined) {
    this.worker = worker;
    this.tier = ready.tier;
    this.transport = ready.transport;
    this.ready = ready;
    this.#rpc = rpc;
    worker.addEventListener("message", (e: MessageEvent) => {
      const m = e.data as { type: string; id?: number; value?: unknown; error?: string; frame?: string };
      if (m.type === "rpc" && m.frame !== undefined) {
        this.#answer(m.frame);
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
    const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
    canvas.width = Math.max(1, Math.floor(canvas.clientWidth * dpr));
    canvas.height = Math.max(1, Math.floor(canvas.clientHeight * dpr));
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
      },
      [offscreen],
    );
    const shell = new Shell(worker, await ready, rpc);
    shell.#wire(canvas, ring);
    return shell;
  }

  /**
   * Call one probe member (JSON-RPC 2.0) through the request ring when the
   * page is cross-origin isolated, else through postMessage. Resolves with
   * the member's result (a protocol-5 refusal is `false` or `null`); rejects
   * with the JSON-RPC error's message. A `capture` resolves once its
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
    const m = JSON.parse(frame) as { id?: number; result?: unknown; error?: { message?: string } };
    const p = typeof m.id === "number" ? this.#rpcPending.get(m.id) : undefined;
    if (!p) return;
    this.#rpcPending.delete(m.id!);
    if (m.error) p.reject(new Error(m.error.message ?? "probe error"));
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

  /** Start or stop ticking with the display. */
  play(on: boolean): void {
    this.worker.postMessage({ type: "play", on });
  }

  #send(ring: SharedArrayBuffer | undefined, event: InjectedInput): void {
    if (ring) writeRing(ring, event);
    else this.worker.postMessage({ type: "input", event });
  }

  #wire(canvas: HTMLCanvasElement, ring: SharedArrayBuffer | undefined): void {
    const listen = <K extends keyof HTMLElementEventMap>(target: EventTarget, type: K, fn: (e: HTMLElementEventMap[K]) => void) => {
      target.addEventListener(type, fn as EventListener);
      this.#listeners.push(() => target.removeEventListener(type, fn as EventListener));
    };
    listen(globalThis as unknown as EventTarget, "keydown", (e) => {
      if (!e.repeat) this.#send(ring, { kind: "key", key: e.code, action: "down" });
    });
    listen(globalThis as unknown as EventTarget, "keyup", (e) => this.#send(ring, { kind: "key", key: e.code, action: "up" }));
    const point = (e: PointerEvent, action: "move" | "down" | "up") =>
      this.#send(ring, { kind: "pointer", action, x: e.offsetX, y: e.offsetY, button: action === "move" ? "none" : (BUTTONS[e.button] ?? "left") });
    listen(canvas, "pointermove", (e) => point(e, "move"));
    listen(canvas, "pointerdown", (e) => point(e, "down"));
    listen(canvas, "pointerup", (e) => point(e, "up"));
    listen(canvas, "wheel", (e) => this.#send(ring, { kind: "wheel", x: e.offsetX, y: e.offsetY, deltaX: e.deltaX, deltaY: e.deltaY }));
  }

  /** Remove the page listeners and stop the Worker. */
  dispose(): void {
    for (const off of this.#listeners.splice(0)) off();
    this.worker.terminate();
  }
}
