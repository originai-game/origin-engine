// The engine Worker's frame loop (worker.ts): a display frame at a time
// (requestAnimationFrame in the Worker, a timer where the display sends
// none), each pumping input and the probe's requests and, while the clock
// plays, ticking and drawing at the frame cap; and the loop that answers the
// probe's requests between frames whether the clock plays or not.

import { FramePacer } from "../pacing.ts";
import { waitForFrames } from "../rpc-ring.ts";
import type { Gate } from "./gate.ts";
import type { WorkerScope } from "./messages.ts";
import { pumpRoom } from "./platform.ts";
import { post, type Run } from "./run.ts";

declare const self: WorkerScope;

/** How often, at most, a playing Worker tells the shell it is alive (`beat`), ms. */
export const BEAT_MS = 1000;

export class FrameLoop {
  readonly #run: Run;
  readonly #gate: Gate;
  #scheduled = false;
  #lastBeat = 0;
  /** The frame cap (og.render.setDisplay, each frame's `maxFps`) and the display's measured interval (pacing.ts). */
  readonly #pacer = new FramePacer();
  #maxFps = 0;
  #toldDisplayMs = 0;

  constructor(run: Run, gate: Gate) {
    this.#run = run;
    this.#gate = gate;
  }

  #frame(now: number): void {
    const run = this.#run;
    this.#scheduled = false;
    const engine = run.engine;
    if (!engine || run.crashed) return;
    const probes = run.probes;
    let drawn = true;
    try {
      // Input on every display frame, drawn or not: a capped frame's input is no later than the cap's interval.
      run.pumpInput(now);
      // Retries first: a capture asked this frame is requested now and answered on a later frame.
      probes.retry();
      probes.drain();
      if (run.playing && probes.waiting === 0) {
        drawn = this.#pacer.frame(now, this.#maxFps);
        const d = this.#pacer.displayMs;
        if (d > 0 && Math.abs(d - this.#toldDisplayMs) > this.#toldDisplayMs * 0.05) {
          this.#toldDisplayMs = d;
          run.glue?.og_web_display_interval?.(engine.id, d);
        }
      }
      // The clock holds while a capture waits for its readback; a display
      // frame the cap skips is not drawn, and the next drawn one is handed
      // the time since the last drawn one, so the fixed tick runs on.
      if (run.playing && probes.waiting === 0 && drawn) {
        const began = performance.now();
        const answered = engine.frame(run.last ? now - run.last : 0);
        if (!answered.ok) {
          // The tick a system threw in, which a catch-up frame reaches past the last frame's.
          if (typeof answered.tick === "number") run.lastTick = answered.tick;
          run.crash(answered.reason ?? "the engine refused a frame");
          return;
        }
        if (run.last) {
          // As the records keep it (single precision), so both count the same frame.
          const spent = Math.fround(performance.now() - began);
          run.records.add(now - run.last, spent);
          run.frameCounter?.add(now - run.last, spent);
        }
        run.behind = answered.behind ?? 0;
        if (typeof answered.maxFps === "number") this.#maxFps = answered.maxFps;
        run.relay(answered);
        pumpRoom(engine);
        if (typeof answered.tick === "number") run.lastTick = answered.tick;
        this.#gate.warmCovered();
        if (!run.firstFrame && answered.rendered !== false) {
          run.firstFrame = true;
          post(run.boot.firstFrame());
        }
        run.audio?.pump();
      }
      run.flushLog();
    } catch (e) {
      run.crash(e instanceof Error ? (e.stack ?? e.message) : String(e));
      return;
    }
    // Alive: the shell's frame deadline hears from a playing Worker about once a second.
    if (run.playing && now - this.#lastBeat >= BEAT_MS) {
      this.#lastBeat = now;
      self.postMessage({ type: "beat", tick: run.lastTick });
    }
    if (drawn) run.last = now;
    if (run.playing || probes.waiting > 0) this.schedule();
  }

  /**
   * The next frame on the display (requestAnimationFrame in the Worker); a
   * display that stops sending frames (a hidden page) gets a timer instead,
   * so a capture still lands.
   */
  schedule(): void {
    if (this.#scheduled) return;
    this.#scheduled = true;
    let ran = false;
    const run = (t: number) => {
      if (ran) return;
      ran = true;
      this.#frame(t);
    };
    if (self.requestAnimationFrame) self.requestAnimationFrame(run);
    if (!this.#run.playing || !self.requestAnimationFrame) setTimeout(() => run(performance.now()), self.requestAnimationFrame ? 100 : 16);
  }

  /** Start or stop the clock (the shell's `play`): the next drawn frame is handed no time from before. */
  play(on: boolean): void {
    this.#run.playing = on;
    this.#run.last = 0;
    this.#pacer.reset();
    if (on) this.schedule();
  }

  /** Requests are answered between frames, whether the clock plays or not. */
  async requests(): Promise<void> {
    const run = this.#run;
    while (run.probes.rings && !run.crashed) {
      await waitForFrames(run.probes.rings.requests, 1000);
      if (!run.engine) continue;
      try {
        run.pumpInput(performance.now());
        run.probes.drain();
      } catch (e) {
        run.crash(e instanceof Error ? e.message : String(e));
      }
      // Yield to the event loop: frames, fetches and messages run between requests.
      await new Promise((r) => setTimeout(r, 0));
    }
  }
}
