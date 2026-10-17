// The main-thread shell (docs/design.md §2.2): it owns the page, transfers
// the canvas to the engine Worker as an OffscreenCanvas, and pumps input into
// the SharedArrayBuffer ring when the page is cross-origin isolated (falling
// back to postMessage otherwise). It never runs game code.

import { createRing, writeRing, type InjectedInput } from "./input.ts";

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
}

const BUTTONS = ["left", "middle", "right"] as const;

export class Shell {
  readonly worker: Worker;
  readonly tier: string;
  /** `ring` when input crosses through the SharedArrayBuffer, `message` otherwise. */
  readonly transport: "ring" | "message";
  readonly #pending = new Map<number, { resolve(v: unknown): void; reject(e: Error): void }>();
  readonly #listeners: Array<() => void> = [];
  #next = 1;

  private constructor(worker: Worker, tier: string, transport: "ring" | "message") {
    this.worker = worker;
    this.tier = tier;
    this.transport = transport;
    worker.addEventListener("message", (e: MessageEvent) => {
      const m = e.data as { type: string; id?: number; value?: unknown; error?: string };
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
    const ready = new Promise<{ tier: string; transport: "ring" | "message" }>((resolve, reject) => {
      const on = (e: MessageEvent) => {
        const m = e.data as { type: string; tier?: string; transport?: "ring" | "message"; message?: string };
        if (m.type === "ready") {
          worker.removeEventListener("message", on);
          resolve({ tier: m.tier ?? "none", transport: m.transport ?? "message" });
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
        autoplay: options.autoplay ?? true,
      },
      [offscreen],
    );
    const { tier, transport } = await ready;
    const shell = new Shell(worker, tier, transport);
    shell.#wire(canvas, ring);
    return shell;
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
