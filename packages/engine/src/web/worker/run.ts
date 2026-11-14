// The engine Worker's run (worker.ts): the engine and its glue once started,
// the clock the frames keep, whether it plays, and how it fails. Every other
// concern of the Worker (the frame loop, the play gate, the calls, the boot)
// reads and moves the run through this object.

import type { Engine } from "../../runtime.ts";
import type { WorkerAudio, WebAudioGlue } from "../audio.ts";
import type { FrameCounter } from "../frames.ts";
import type { WebExtras, WebGlue } from "../host.ts";
import { drainRing, stampAt, type InjectedInput } from "../input.ts";
import { PlatformCounter } from "../settle.ts";
import type { HostError } from "../shell.ts";
import type { WorkerScope } from "./messages.ts";
import { BootClock, EngineTrace, FrameRecords, PageRelay, ProbeRequests } from "./state.ts";

declare const self: WorkerScope;

/** The wasm-bindgen glue the engine runs in, with the web host's extras. */
export type Glue = WebGlue & WebExtras & WebAudioGlue;

/** A message to the shell. */
export const post = (message: Record<string, unknown>) => self.postMessage(message);

/**
 * What a failure was: a Rust panic the wasm recorded before it trapped
 * (`og_web_panic`: its message, location, tick and lane), or else `message`
 * at the last tick a frame reached.
 */
export function hostErrorOf(panic: string | undefined, message: string, tick: number | null): HostError {
  try {
    const p = JSON.parse(panic ?? "null") as { kind?: string; message?: string; location?: string; tick?: number; lane?: string | null } | null;
    if (p && p.kind === "panic") {
      return { kind: "panic", message: p.message ?? message, tick: typeof p.tick === "number" ? p.tick : tick, lane: p.lane ?? null, location: p.location || null };
    }
  } catch {
    /* not JSON: no panic report */
  }
  return { kind: "error", message, tick, lane: null, location: null };
}

/** One line for the crash card and the log: `panic at tick 41 in a lane (cull): … (file:line:col)`. */
export function describeHostError(e: HostError): string {
  if (e.kind !== "panic") return e.message;
  return `panic at tick ${e.tick ?? "?"}${e.lane ? ` in a lane (${e.lane})` : ""}: ${e.message}${e.location ? ` (${e.location})` : ""}`;
}

export class Run {
  engine: Engine | null = null;
  glue: Glue | null = null;
  /** The input ring, on a cross-origin isolated page. */
  ring: SharedArrayBuffer | undefined;
  canvas: OffscreenCanvas | undefined;
  rendering = false;
  playing = false;
  ready = false;
  crashed = false;
  firstFrame = false;
  /** The last drawn display frame's timestamp (0: none since play began). */
  last = 0;
  /** Real time not yet simulated after the last frame (its answer's `behind`), ms. */
  behind = 0;
  /** The tick the last frame reached (a crash's tick when no panic names one). */
  lastTick: number | null = null;
  /** What the trace's document records beside its events: the renderer's tier and the lanes. */
  lastTier = "none";
  lanes: unknown = null;
  audio: WorkerAudio | null = null;
  /** Field telemetry's frame counts (frames.ts), when the page's build has telemetry on; null otherwise. */
  frameCounter: FrameCounter | null = null;
  /** What the page was asked and answered: the ticks behind the gate apply its answers (settle.ts). */
  readonly platformCounter = new PlatformCounter();
  /** Where the boot went (BootPhases), sent with the first frame. */
  readonly boot = new BootClock();
  /** What the page was last told of a frame's answer. */
  readonly page = new PageRelay(post);
  /** The played frames' timings (the shell's `timings` call). */
  readonly records = new FrameRecords();
  /** The run's trace while one is recorded. */
  readonly trace = new EngineTrace();
  /** The probe's requests: answered between frames, a pending capture on a later one. */
  readonly probes: ProbeRequests;

  /** `schedule` asks for a display frame (the frame loop's). */
  constructor(schedule: () => void) {
    this.probes = new ProbeRequests({ engine: () => this.engine, rendering: () => this.rendering, crash: (message) => this.crash(message), schedule }, post);
  }

  /** A line of the console or the engine's log, to the shell (the crash card's last lines). */
  log(text: string): void {
    self.postMessage({ type: "log", lines: [text] });
  }

  flushLog(): void {
    if (!this.engine) return;
    const lines = this.engine.takeLog();
    if (lines.length > 0) self.postMessage({ type: "log", lines: lines.map((l) => `tick ${l.tick}: ${l.warning ? "warning: " : ""}${l.text}`) });
  }

  crash(message: string): void {
    if (this.crashed) return;
    this.crashed = true;
    this.playing = false;
    let panic: string | undefined;
    try {
      // A trap after a Rust panic: the panic hook recorded what it said, where and when.
      panic = this.glue?.og_web_panic?.();
    } catch {
      /* the module is past answering */
    }
    const error = hostErrorOf(panic, message, this.lastTick);
    try {
      this.flushLog();
    } catch {
      /* the engine may be the thing that broke */
    }
    const text = describeHostError(error);
    self.postMessage(this.ready ? { type: "crash", message: text, error } : { type: "error", message: text, error });
  }

  /** A frame's answer: its trace taken, what changed told to the page. */
  relay(answered: ReturnType<Engine["frame"]>): void {
    this.trace.take(this.engine);
    this.page.relay(answered);
  }

  /**
   * An event with the page's time becomes one with its `at` within the tick
   * that reads it (input.ts stampAt): `now` is this Worker's clock now (a
   * frame's timestamp, or a message's arrival), on the same timeline as `last`.
   */
  stamped(event: InjectedInput, now: number): InjectedInput {
    const engine = this.engine;
    if (!engine || !("time" in event) || event.time === undefined) return event;
    const { time, ...rest } = event;
    const ahead = this.behind + (this.last ? Math.max(0, now - this.last) : 0);
    return { ...rest, at: stampAt(time, performance.timeOrigin + now, ahead, 1000 / engine.rate) } as InjectedInput;
  }

  pumpInput(now: number): void {
    if (this.engine && this.ring) for (const event of drainRing(this.ring)) this.engine.probe.input(this.stamped(event, now));
  }
}
