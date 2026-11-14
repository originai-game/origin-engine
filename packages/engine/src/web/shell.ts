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
// settings; glass, scrims and translucent fills become opaque), followed as
// they change. So does the platform's keyboard: on Apple platforms
// `macKeys` gives og-ui's text controls macOS's editing keys (Command for
// the shortcuts and the line's ends, Option by words, Control+A and E). A
// Mac's browser sends no keyup for a key released while Command is held,
// so the shell releases those keys itself when Command goes up.
//
// Focus: a key held when the window loses focus (Alt+Tab, a click into
// the browser's own UI, the tab hidden) never sends its keyup to the page,
// so on `blur` and on the page going hidden the shell sends an up for every
// key it sent down (releaseHeld) and the engine holds nothing.
//
// Rumble: a frame's `rumble` (og.input.rumble) plays on the pads'
// vibration actuators (gamepad.ts playRumble).
//
// Pointer lock (mouse-look): a pointer move carries the mouse's movementX/Y
// (og.Pointers.movement sums them per tick). When a frame's answer says the
// game wants the lock (og.PointerLock.wanted, the Worker's `pointer-lock`
// message), the next press on the canvas calls requestPointerLock (browsers
// grant it only to a gesture), with unadjustedMovement when the player wants
// raw input (og.input.setMouse) and the browser has not refused it; when it
// no longer does, exitPointerLock. pointer-lock.ts's state machine decides
// every step, so Escape, focus loss and a refused or unanswered request
// never leave the game stuck: the next click locks again.
// pointerlockchange is reported to the engine as a `pointerLock` input, so
// og.PointerLock.locked follows the page (Escape releases it), and what the
// browser granted as og.input.mouse. While locked, the absolute position the
// shell sends stays where the lock began (the UI's hover and clicks land
// there), and only the movement moves.
//
// Presentation (og.render.setDisplay's page asks, each frame's answer):
// fullscreen is entered on the frame that asks for it (within the click's
// activation on the settings page) or on the next press or key when the
// browser wanted a gesture, left when it no longer asks, and the page's
// fullscreenchange is reported back (Escape turns the setting off). The
// power preference is kept in the page's storage for the next load, whose
// adapter it picks (an adapter is chosen once). The device pixel ratio is
// followed with matchMedia on `resolution`: a window moved to a monitor of
// another DPR resizes the canvas and the UI's viewport before the next
// frame. The keyboard layout's printed keys reach og.input's glyphs
// (keyboard-layout.ts).

import { writeRing, type InjectedInput } from "./input.ts";
import type { CursorImage } from "../runtime.ts";
import type { TextEntry, TextTarget } from "./text-input.ts";
import type { A11yMirror, A11yTree } from "./a11y.ts";
import { playHaptics, type HapticPulse } from "./haptics.ts";
import { playRumble, type RumbleRequest } from "./gamepad.ts";
import type { RpcRings } from "./rpc-ring.ts";
import type { PlatformCall, PlatformHost, PlatformNotice } from "../platform-local.ts";
import type { BootPhases } from "./worker/state.ts";
import type { TraceDocument } from "../trace.ts";
import { bootWorker, type HostError, type Ready } from "./shell/boot.ts";
import { FrameDeadline } from "./shell/deadline.ts";
import { wireInput } from "./shell/input.ts";
import { PlatformBridge, type RoomHost } from "./shell/platform.ts";
import { Presentation } from "./shell/presentation.ts";
import { Requests } from "./shell/requests.ts";
import { pagePreferences, pixelSize, viewportExtremes } from "./shell/viewport.ts";

export { BOOT_DEADLINE_MS, webGpuMissing, type HostError, type Ready } from "./shell/boot.ts";
export { FRAME_DEADLINE_MS } from "./shell/deadline.ts";
export { releaseHeld } from "./shell/input.ts";
export type { RoomHost } from "./shell/platform.ts";
export { PAGE_PREFERENCES, macKeys, pagePreferences, pixelSize, storedLowPower, viewportExtremes, watchPixelRatio } from "./shell/viewport.ts";
export { LANES_MAX_THREADS, laneChoice } from "./lanes.ts";

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
  /** A replay seed; omitted is GameDesc.seed. */
  readonly seed?: number;
  /** Tick and render with the display (default true); false leaves ticks to `call("advance", n)`. */
  readonly autoplay?: boolean;
  /** The URL base (as the Worker resolves it) whose `cooked/` bundle the Worker fetches before the engine starts. */
  readonly root?: string;
  /** The Basis transcoder's script and module URLs (as the Worker resolves them). */
  readonly basis?: { readonly js: string; readonly wasm: string };
  /** The rav1d AV1 fallback's module URL (as the Worker resolves it), fetched only where WebCodecs cannot decode AV1. */
  readonly av1?: string;
  /** The project's physics profile (project.ogt `profile`); omitted is 3d. */
  readonly profile?: "2d" | "3d";
  /** project.ogt's text: its tick, seed, scenes and scene fill what GameDesc leaves unset (CreateOptions.project). */
  readonly project?: string;
  /** Boot progress, 0 to 1, with the stage (`wasm`, `bundle`, `fetch`, `transcode`, `ready`). */
  readonly onProgress?: (value: number, stage: string, fraction?: number) => void;
  /** Fork s61: a loading stage the game reported itself (worker global `ogBootStage`), for a page that draws its own loading screen. */
  readonly onStage?: (id: string, fraction: number) => void;
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
  /** Record the run's trace from the boot on, the boot scene's loads included (`trace()` reads it; `trace("start")` begins one later). */
  readonly trace?: boolean;
  /** Count the played frames for field telemetry (`call("telemetry")` takes the counts; frames.ts). */
  readonly telemetry?: boolean;
  /** Fork s60: project.ogt `publish { pipelines }` (src/web/pipelines.ts): `sync`, `async` or `prewarm`; absent: auto, picked at the adapter. */
  readonly pipelines?: "sync" | "async" | "prewarm";
  /** A file of the bundle did not arrive, at the boot or later: its path under `cooked/` and why (`http_<status>`, `timeout`, `network`). */
  readonly onLoadFailure?: (failure: { readonly file: string; readonly reason: string }) => void;
  /** The inspector saved scenes on this page, which writes no files (development builds): each one's source and whole new text. */
  readonly onScenesSaved?: (scenes: readonly { readonly source: string; readonly text: string }[]) => void;
  /**
   * Ask the browser for the low-power GPU adapter (og.render's PowerPreference
   * lowPower). Omitted: what the player chose last on this page
   * (og.render.setDisplay's powerPreference, kept in the page's storage under
   * `displayKey`), else high performance.
   */
  readonly lowPower?: boolean;
  /** The storage key the player's power preference is kept under (default: `og.display:<path>`). */
  readonly displayKey?: string;
}

/**
 * The engine Worker as the page holds it. Its concerns live under shell/:
 * the boot (boot.ts), the calls and probe requests waiting for an answer
 * (requests.ts), the frame deadline (deadline.ts), the page's input
 * (input.ts), what a frame's answer asks of the page (presentation.ts),
 * og.platform and the rooms (platform.ts), and what the shell measures of
 * the page (viewport.ts).
 */
export class Shell {
  readonly worker: Worker;
  readonly tier: string;
  /** `ring` when input crosses through the SharedArrayBuffer, `message` otherwise. */
  readonly transport: "ring" | "message";
  /** What the Worker reported at start. */
  readonly ready: Ready;
  /** Where the Worker's boot went (worker.ts BootPhases), once the first frame is drawn. */
  boot: BootPhases | null = null;
  readonly #requests: Requests;
  readonly #deadline = new FrameDeadline();
  readonly #presentation: Presentation;
  readonly #platform: PlatformBridge;
  readonly #listeners: Array<() => void> = [];
  #onLog: ShellOptions["onLog"];
  #onCrash: ShellOptions["onCrash"];
  #onFirstFrame: ShellOptions["onFirstFrame"];
  #onLoadFailure: ShellOptions["onLoadFailure"];
  #onScenesSaved: ShellOptions["onScenesSaved"];
  #onProgress: ShellOptions["onProgress"];
  #onStage: ShellOptions["onStage"];
  #ring: SharedArrayBuffer | undefined;
  #dprCap = 2;
  #playing = false;
  /** The hidden textarea that takes the page's text for the UI's focused field (text-input.ts). */
  #text: TextEntry | undefined;
  /** The DOM mirror of the UI's accessibility tree (a11y.ts). */
  #a11y: A11yMirror | undefined;
  /** The viewport last sent to the Worker (the page's preferences are re-read into it when they change). */
  #viewport: Record<string, unknown> = {};
  /** What stopped the engine, once something did. */
  #error: HostError | null = null;
  /** The canvas's CSS box, safe area and covered region as the page last laid it out: what a DPR change resizes again. */
  #layout: { width: number; height: number; safeArea?: Record<string, number>; covered?: Record<string, number> } | null = null;

  private constructor(worker: Worker, ready: Ready, rpc: RpcRings | undefined, canvas: HTMLCanvasElement, displayKey: string) {
    this.worker = worker;
    this.tier = ready.tier;
    this.transport = ready.transport;
    this.ready = ready;
    const post = (message: Record<string, unknown>) => worker.postMessage(message);
    this.#requests = new Requests(worker, rpc);
    this.#presentation = new Presentation(canvas, post, displayKey);
    this.#platform = new PlatformBridge(post, () => this.#deadline.tick, (on) => this.play(on));
    worker.addEventListener("message", (e: MessageEvent) => {
      const m = e.data as { type: string; id?: number; value?: unknown; error?: string; frame?: string; lines?: string[]; message?: string };
      if (m.type === "log" && m.lines) {
        this.#onLog?.(m.lines);
        return;
      }
      if (m.type === "crash") {
        const message = m.message ?? "the engine stopped";
        const error = (m as unknown as { error?: HostError }).error ?? { kind: "error", message, tick: this.#deadline.tick, lane: null, location: null };
        this.#crashed(message, error);
        return;
      }
      if (m.type === "load-failure") {
        this.#onLoadFailure?.(m as unknown as { file: string; reason: string });
        return;
      }
      if (m.type === "beat") {
        this.#deadline.beat((m as { tick?: number | null }).tick);
        // A host that answers later (an ad's answer after the resume) or sends notices on its own.
        this.#platform.beat();
        return;
      }
      // Fork s61: the stages after "ready" (the first tick, the first frame's pipelines) move the bar too.
      if (m.type === "progress") {
        const p = m as unknown as { value?: number; stage?: string; fraction?: number };
        this.#onProgress?.(p.value ?? 0, p.stage ?? "", p.fraction);
        return;
      }
      // Fork s61: the game's own loading stages (its loading screen's, after the engine's).
      if (m.type === "og-stage") {
        const p = m as unknown as { id?: string; fraction?: number };
        this.#onStage?.(p.id ?? "", p.fraction ?? 0);
        return;
      }
      // Fork s60: the pipelines' counts when the warm-up ends, over the first frame's.
      if (m.type === "pipelines") {
        const stats = (m as { stats?: BootPhases["pipelines"] }).stats;
        if (stats) this.boot = { ...(this.boot ?? {}), pipelines: stats };
        return;
      }
      if (m.type === "frame") {
        this.boot = (m as { boot?: BootPhases }).boot ?? null;
        this.#onFirstFrame?.();
        return;
      }
      if (m.type === "text") {
        this.#text?.update((m as unknown as { target: TextTarget | null }).target);
        return;
      }
      if (m.type === "haptics") {
        playHaptics((m as unknown as { patterns: HapticPulse[][] }).patterns);
        return;
      }
      if (m.type === "rumble") {
        playRumble((m as unknown as { rumble: RumbleRequest[] }).rumble);
        return;
      }
      if (m.type === "saved") {
        this.#onScenesSaved?.((m as unknown as { scenes: { source: string; text: string }[] }).scenes);
        return;
      }
      if (m.type === "copy") {
        this.#text?.copied((m as unknown as { text: string }).text);
        return;
      }
      if (m.type === "a11y") {
        this.#a11y?.update((m as unknown as { tree: A11yTree }).tree);
        this.#text?.relabel();
        return;
      }
      if (m.type === "cursor") {
        const c = m as unknown as { cursor: string; image?: CursorImage | null };
        this.#presentation.cursor(c.cursor || "default", c.image ?? null);
        return;
      }
      if (m.type === "pointer-lock") {
        const ask = m as unknown as { wanted: boolean; raw?: boolean };
        this.#presentation.want(ask.wanted === true, ask.raw !== false);
        return;
      }
      if (m.type === "page") {
        this.#presentation.page(m as unknown as { fullscreen?: boolean; lowPower?: boolean });
        return;
      }
      if (m.type === "rpc" && m.frame !== undefined) {
        this.#requests.answer(m.frame);
        return;
      }
      if (m.type === "room") {
        this.#platform.rooms((m as unknown as { ops: Record<string, unknown>[] }).ops ?? []);
        return;
      }
      if (m.type === "platform") {
        this.#platform.calls((m as unknown as { calls: PlatformCall[] }).calls ?? []);
        return;
      }
      if (m.type !== "result" || m.id === undefined) return;
      this.#requests.result(m.id, m.error === undefined ? undefined : new Error(m.error), m.value);
    });
  }

  /** Start the engine in the Worker and wire the page's input to it. */
  static async start(options: ShellOptions): Promise<Shell> {
    const { canvas, worker } = options;
    const { ready, ring, rpc, dpr, displayKey } = await bootWorker(options);
    const shell = new Shell(worker, ready, rpc, canvas, displayKey);
    shell.#ring = ring;
    shell.#dprCap = options.dprCap ?? 2;
    shell.#onLog = options.onLog;
    shell.#onCrash = options.onCrash;
    shell.#onFirstFrame = options.onFirstFrame;
    shell.#onLoadFailure = options.onLoadFailure;
    shell.#onScenesSaved = options.onScenesSaved;
    shell.#onProgress = options.onProgress;
    shell.#onStage = options.onStage;
    shell.#playing = options.autoplay ?? true;
    shell.#viewport = { width: canvas.clientWidth || canvas.width / dpr, height: canvas.clientHeight || canvas.height / dpr, dpr, coarsePointer: globalThis.matchMedia?.("(pointer: coarse)").matches ?? false };
    shell.#layout = { width: shell.#viewport["width"] as number, height: shell.#viewport["height"] as number };
    shell.#deadline.start({
      playing: () => shell.#playing,
      stopped: () => shell.#requests.stopped !== null,
      hung: (message, error) => {
        shell.#crashed(message, error);
        shell.worker.terminate();
      },
    });
    shell.setPlatform(options.platform ?? shell.#platform.platform);
    const wired = wireInput(canvas, {
      send: (e) => shell.#send(ring, e),
      post: (message) => shell.worker.postMessage(message),
      viewport: () => shell.#viewport,
      stopped: () => shell.#requests.stopped !== null,
      resized: () => {
        const l = shell.#layout;
        if (l) shell.resize(l.width, l.height, l.safeArea, l.covered);
      },
      presentation: shell.#presentation,
      listeners: shell.#listeners,
    });
    shell.#a11y = wired.mirror;
    shell.#text = wired.text;
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
    return this.#requests.probe(method, params);
  }

  /**
   * The run's trace in the Chrome trace format, as `og run --trace` writes it natively (src/trace.ts):
   * buffered in the engine Worker, taken here. `take` (the default) answers the document so far (null
   * when no trace is recorded), `start` begins one (`ShellOptions.trace` starts one at the boot),
   * `stop` ends it.
   */
  trace(action: "take" | "start" | "stop" = "take"): Promise<TraceDocument | null | boolean> {
    return this.call("trace", action) as Promise<TraceDocument | null | boolean>;
  }

  /** Saves the trace so far as a JSON file the browser downloads (`name`, default `trace.json`): open it in Perfetto. False when none is recorded. */
  async saveTrace(name = "trace.json"): Promise<boolean> {
    const doc = await this.trace("take");
    if (!doc || typeof doc !== "object") return false;
    const url = URL.createObjectURL(new Blob([JSON.stringify(doc)], { type: "application/json" }));
    const a = Object.assign(document.createElement("a"), { href: url, download: name });
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    return true;
  }

  /** Call the engine in the Worker: `advance`, `command`, `hash`, `frame`, or `probe` with a member name. */
  call(method: string, ...args: unknown[]): Promise<unknown> {
    return this.#requests.call(method, args);
  }

  /**
   * What carries og.platform's rooms from now on (og.platform.roomInfo's `host`); null for
   * none, where roomCreate and roomJoin are refused. A host already carrying a room keeps it
   * until its `left`: the engine is told the new host only for its next room. `link` is the room
   * a Play link named (`?room=<id>`, the play page passes it): the engine joins it once, itself,
   * as soon as a host carries rooms (og.RoomJoined answers, as for roomJoin; with none it says
   * this page carries no rooms), and og.platform.roomInfo's `link` names it.
   */
  setRooms(host: RoomHost | null, link?: string): void {
    this.#platform.setRooms(host, link);
  }

  /** How the page pauses the game for its platform host (ticks and audio, as its own pause does); default: the clock alone. */
  onPlatformPause(pause: (on: boolean) => void): void {
    this.#platform.onPause(pause);
  }

  /** Unasked platform input (a Player's pushed notice): applied at the next tick's start as og.PlatformNotice. */
  notify(notice: PlatformNotice): void {
    this.#platform.notify(notice);
  }

  /** What answers og.platform's calls from now on (og.platform.info reports its host and capabilities). */
  setPlatform(host: PlatformHost): void {
    this.#platform.setPlatform(host);
  }

  /** What stopped the engine (a panic, an error, a hang), or null while it runs. */
  get error(): HostError | null {
    return this.#error;
  }

  /** The engine failed: nothing waits for an answer it will not give, and the page hears why once. */
  #crashed(message: string, error: HostError): void {
    if (this.#error) return;
    this.#error = error;
    this.#requests.stop(`the engine stopped: ${message}`);
    this.#deadline.stop();
    this.#onCrash?.(message, error);
  }

  /** Start or stop ticking with the display. */
  play(on: boolean): void {
    this.#playing = on;
    this.#deadline.restart();
    this.worker.postMessage({ type: "play", on });
  }

  /**
   * A user gesture outside the canvas (the play gate's click, Enter or
   * Space): a pointer lock the game already wants is asked for with it, so
   * the click that starts the game also takes the lock instead of waiting
   * for the next one on the canvas. A lock not wanted asks nothing.
   */
  gesture(): void {
    this.#presentation.press();
  }

  /** Whether the clock ticks with the display (the last `play`). */
  get playing(): boolean {
    return this.#playing;
  }

  /**
   * The canvas was laid out at `cssWidth`×`cssHeight`: resize the render
   * target (device pixels, with the DPR cap) and tell the UI its viewport,
   * with the page's own control over the canvas (`covered`, Viewport.covered:
   * the shell's Back) that anchored groups keep out of.
   */
  resize(cssWidth: number, cssHeight: number, safeArea?: Record<string, number>, covered?: Record<string, number>): { width: number; height: number; dpr: number } {
    this.#layout = { width: cssWidth, height: cssHeight, ...(safeArea ? { safeArea } : {}), ...(covered ? { covered } : {}) };
    const size = pixelSize(cssWidth, cssHeight, globalThis.devicePixelRatio || 1, this.#dprCap);
    const viewport = {
      width: cssWidth,
      height: cssHeight,
      dpr: size.dpr,
      ...(safeArea ? { safeArea } : {}),
      // svh and lvh: as the browser's bar shows and hides, the canvas (dvh)
      // follows it between these.
      ...viewportExtremes(cssWidth, cssHeight),
      ...(covered ? { covered } : {}),
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

  /** Remove the page listeners and stop the Worker. */
  dispose(): void {
    this.#deadline.stop();
    this.#requests.stop("the shell was disposed");
    for (const off of this.#listeners.splice(0)) off();
    this.worker.terminate();
  }
}
