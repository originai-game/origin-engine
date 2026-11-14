// What the engine Worker (worker.ts) keeps between frames, one object per
// concern, each with its own state and nothing of another's: where the boot
// went (BootClock), what the page was last told (PageRelay), the played
// frames' timings (FrameRecords), the run's trace (EngineTrace) and the probe
// requests waiting for an answer (ProbeRequests). startWorker holds the run
// itself: the engine, the clock and the frame loop.

import { boundHost, type CursorImage, type Engine } from "../runtime.ts";
import { Trace, unixMs, type TraceDocument } from "../trace.ts";
import * as pipelines from "./pipelines.ts";
import type { PipelineChoice, PipelineStats } from "./pipelines.ts";
import { readFrames, writeFrame, type RpcRings } from "./rpc-ring.ts";

/** A message to the shell. */
type Post = (message: Record<string, unknown>) => void;

/**
 * Where the boot went, ms on the Worker's clock: each phase's duration, in
 * order (`wasm`: the engine module fetched and instantiated, the lanes
 * pool started; `bundle`: the first group fetched; `create`: the engine and
 * the game's declarations; `attach`: the renderer created on the canvas;
 * `transcode`: the Basis textures; `prime`: the tick behind the gate;
 * `settle`: the ticks that applied the platform's answers behind the gate;
 * `present`: the first frame rendered, its pipelines compiled), and `at`,
 * the Worker's time at the first frame from its own start.
 */
export type BootPhases = Partial<Record<"wasm" | "bundle" | "create" | "attach" | "transcode" | "prime" | "settle" | "present", number>> & {
  at?: number;
  /** Fork s60: the render pipelines made each way (pipelines.ts), at the first frame and again when the warm-up ends or play stops it, with the mode chosen and why (`auto`'s pick) and `bypassed` when the frame pipelines lost createRenderPipelineAsync. */
  pipelines?: PipelineStats & Partial<PipelineChoice> & { bypassed?: string };
};

/** Where the boot went (BootPhases): each phase ends where the next begins. */
export class BootClock {
  readonly phases: BootPhases = {};
  #at = performance.now();

  /** The first phase begins now. */
  restart(): void {
    this.#at = performance.now();
  }

  /** Phase `name` ends now (a phase entered twice adds up). */
  phase(name: Exclude<keyof BootPhases, "at" | "pipelines">): void {
    const now = performance.now();
    this.phases[name] = Math.round((this.phases[name] ?? 0) + now - this.#at);
    this.#at = now;
    console.info(`[og] boot stage "${name}" done in ${this.phases[name]} ms`);
  }

  /** The first frame's message: the phases, when it was drawn and the pipelines made for it. */
  firstFrame(): Record<string, unknown> {
    const at = Math.round(performance.now());
    console.info(`[og] first frame at ${at} ms`);
    return { type: "frame", first: true, boot: { ...this.phases, at, pipelines: pipelines.stats() } };
  }
}

/**
 * What a frame's answer tells the shell, each only when it changed: the UI's
 * focused text control (text-input.ts), the accessibility tree (a11y.ts), the
 * pointer's shape, og.PointerLock.wanted, and the page asks (fullscreen, the
 * power preference for the next load); a copy, saved scenes, haptics and
 * rumble whenever the frame has them.
 */
export class PageRelay {
  readonly #post: Post;
  /** The text target last sent, as JSON. */
  #text = "null";
  /** og.PointerLock.wanted and og.input's raw mouse as last sent. */
  #lockWanted = false;
  #lockRaw = true;
  /** The canvas's CSS cursor as the last frame answer set it, and its `url()` image. */
  #cursor = "default";
  #cursorImage: CursorImage | null = null;
  /** The page asks last sent ([fullscreen, lowPower]). */
  #asks = "";

  constructor(post: Post) {
    this.#post = post;
  }

  relay(answered: ReturnType<Engine["frame"]>): void {
    if (!answered.ok) return;
    const post = this.#post;
    const target = JSON.stringify(answered.text ?? null);
    if (target !== this.#text) {
      this.#text = target;
      post({ type: "text", target: answered.text ?? null });
    }
    if (answered.copy !== undefined) post({ type: "copy", text: answered.copy });
    if (answered.saved?.scenes.length) post({ type: "saved", scenes: answered.saved.scenes });
    if (answered.a11y !== undefined) post({ type: "a11y", tree: answered.a11y });
    if (answered.haptics?.length) post({ type: "haptics", patterns: answered.haptics });
    if (answered.rumble?.length) post({ type: "rumble", rumble: answered.rumble });
    const shape = typeof answered.cursor === "string" ? answered.cursor : "default";
    const image = answered.cursorImage !== undefined && answered.cursorImage !== this.#cursorImage;
    if (shape !== this.#cursor || image) {
      this.#cursor = shape;
      if (image) this.#cursorImage = answered.cursorImage ?? null;
      post({ type: "cursor", cursor: this.#cursor, image: this.#cursorImage });
    }
    const wanted = answered.pointerLock === true;
    const raw = answered.rawMouse !== false;
    if (wanted !== this.#lockWanted || raw !== this.#lockRaw) {
      this.#lockWanted = wanted;
      this.#lockRaw = raw;
      post({ type: "pointer-lock", wanted, raw });
    }
    // The player's graphics settings that are the page's to carry out: fullscreen, and the power preference for the next load.
    const asks = JSON.stringify([answered.fullscreen === true, answered.lowPower === true]);
    if (asks !== this.#asks) {
      this.#asks = asks;
      post({ type: "page", fullscreen: answered.fullscreen === true, lowPower: answered.lowPower === true });
    }
  }
}

/** Played frames whose timings the Worker keeps (`timings`). */
const FRAME_RECORDS = 4096;

/**
 * The played frames' timings, the last FRAME_RECORDS of them (the shell's
 * `timings` call, which `og run --device` reads): the display interval
 * (this frame's timestamp minus the last one's) and the engine's work
 * (`engine.frame`: input, ticks, render submission), ms.
 */
export class FrameRecords {
  readonly #intervals = new Float32Array(FRAME_RECORDS);
  readonly #work = new Float32Array(FRAME_RECORDS);
  #recorded = 0;

  add(interval: number, work: number): void {
    this.#intervals[this.#recorded % FRAME_RECORDS] = interval;
    this.#work[this.#recorded % FRAME_RECORDS] = work;
    this.#recorded++;
  }

  /** The frames played since the last reset (at most FRAME_RECORDS, oldest first); `reset` starts again. */
  take(reset: boolean): { frames: number; intervals: number[]; work: number[] } {
    const recorded = this.#recorded;
    const n = Math.min(recorded, FRAME_RECORDS);
    const at = (a: Float32Array) => Array.from({ length: n }, (_, i) => Math.round(a[(recorded - n + i) % FRAME_RECORDS]! * 1000) / 1000);
    const answer = { frames: recorded, intervals: at(this.#intervals), work: at(this.#work) };
    if (reset) this.#recorded = 0;
    return answer;
  }
}

/** The run's trace while one is recorded (src/trace.ts): taken after every frame, so the engine holds a frame's spans at most. */
export class EngineTrace {
  #trace: Trace | null = null;

  /** Starts a trace of `engine` (keeping one already recorded) or ends it; false when the module cannot trace. */
  turn(engine: Engine | null, on: boolean): boolean {
    const abi = boundHost()?.abi;
    if (!engine || !abi) return false;
    if (on && !this.#trace) this.#trace = new Trace({ producer: "og web trace", process: "og web" });
    if (!on) this.#trace = null;
    abi.og_engine_trace(engine.id, on);
    return true;
  }

  /** What `engine` recorded since the last take joins the trace (`last`: the engine closes its open spans). */
  take(engine: Engine | null, last = false): void {
    const abi = boundHost()?.abi;
    if (!this.#trace || !engine || !abi) return;
    this.#trace.append(abi.og_engine_trace_take(engine.id, last), unixMs());
  }

  /** The document so far with `extra` beside its events; null when none is recorded. */
  document(engine: Engine | null, extra: Record<string, unknown>): TraceDocument | null {
    if (!this.#trace) return null;
    this.take(engine, true);
    return this.#trace.document(extra);
  }
}

/**
 * A `capture`, `marks` or `sweep` answered null: on the web the readback finishes on
 * a later call, and a sweep goes on from the frames it has each time it is
 * asked again. (A sweep refused outright, an unknown entity, answers null
 * too: it is asked again until its budget runs out.)
 */
function isPendingCapture(frame: string, answer: string): boolean {
  // Every null result is written `"result":null` (the probe's compact JSON);
  // a frame's answer is megabytes of base64 not worth parsing to learn it is not null.
  if (!answer.includes('"result":null')) return false;
  try {
    const request = JSON.parse(frame) as { method?: string };
    if (request.method !== "capture" && request.method !== "marks" && request.method !== "sweep") return false;
    const response = JSON.parse(answer) as { result?: unknown; error?: unknown };
    return response.error === undefined && response.result === null;
  } catch {
    return false;
  }
}

/**
 * How long a pending `capture` is asked again, once a display frame, before it is given up on (ms).
 * Wall-clock, not frames: at 60 frames a playing page gave up after 1 s, while a GPU process with
 * a cold shader cache (Edge on D3D12) was still compiling the pipelines queued before the readback.
 */
const CAPTURE_MS = 20_000;

/** How long a request may wait for its readbacks: a `sweep` 250 ms more for each value (at most 64). */
function waitOf(frame: string): number {
  try {
    const request = JSON.parse(frame) as { method?: string; params?: { values?: unknown[] } };
    if (request.method === "sweep") return 250 * (request.params?.values?.length ?? 0) + CAPTURE_MS;
  } catch {
    /* not a sweep */
  }
  return CAPTURE_MS;
}

/** What the probe requests need of the run. */
export interface ProbeRun {
  /** The engine once it started. */
  engine(): Engine | null;
  /** Whether a renderer is attached (a capture's readback then lands on a later frame). */
  rendering(): boolean;
  /** The engine failed serving a request. */
  crash(message: string): void;
  /** A display frame is wanted: a capture waits for its readback. */
  schedule(): void;
}

/**
 * The probe's JSON-RPC requests: each answered at once through the ring it
 * came by or a message, or kept and asked again on the following frames
 * while it is a capture whose readback has not landed.
 */
export class ProbeRequests {
  /** The request and answer rings, on a cross-origin isolated page. */
  rings: RpcRings | undefined;
  readonly #run: ProbeRun;
  readonly #post: Post;
  /** Captures waiting for their readback (requested on tick N, answered on N + 1). */
  readonly #retries: Array<{ frame: string; until: number; viaRing: boolean }> = [];

  constructor(run: ProbeRun, post: Post) {
    this.#run = run;
    this.#post = post;
  }

  /** Requests kept for a later frame. */
  get waiting(): number {
    return this.#retries.length;
  }

  #answer(frame: string, viaRing: boolean): void {
    if (!viaRing || !this.rings || !writeFrame(this.rings.answers, frame)) this.#post({ type: "rpc", frame });
  }

  /** A JSON-RPC error answer to a request frame (its id, or null when it has none). */
  refuse(frame: string, message: string, viaRing: boolean): void {
    const id = (() => {
      try {
        return (JSON.parse(frame) as { id?: unknown }).id ?? null;
      } catch {
        return null;
      }
    })();
    this.#answer(JSON.stringify({ jsonrpc: "2.0", id, error: { code: -32603, message } }), viaRing);
  }

  /** One request frame: answered now, or kept for a retry when it is a capture whose readback has not landed. */
  serve(frame: string, viaRing: boolean): void {
    const engine = this.#run.engine();
    if (!engine) return this.refuse(frame, "the engine has not started", viaRing);
    let out: string;
    try {
      // Fork s60: a probe call that renders (capture, sweep) draws its frame whole, never one with draws skipped.
      out = pipelines.drawnWhole(() => engine.call(frame));
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      this.refuse(frame, message, viaRing);
      this.#run.crash(message);
      return;
    }
    if (this.#run.rendering() && isPendingCapture(frame, out)) {
      this.#retries.push({ frame, until: performance.now() + waitOf(frame), viaRing });
      this.#run.schedule();
    } else this.#answer(out, viaRing);
  }

  /** The kept requests asked again: each answered once its readback landed or its wait ran out. */
  retry(): void {
    const engine = this.#run.engine();
    if (!engine) return;
    for (let i = this.#retries.length - 1; i >= 0; i--) {
      const r = this.#retries[i]!;
      const out = pipelines.drawnWhole(() => engine.call(r.frame));
      if (!isPendingCapture(r.frame, out) || performance.now() >= r.until) {
        this.#retries.splice(i, 1);
        this.#answer(out, r.viaRing);
      }
    }
  }

  /** Every request waiting on the ring, served. */
  drain(): void {
    if (this.#run.engine() && this.rings) for (const frame of readFrames(this.rings.requests)) this.serve(frame, true);
  }
}
