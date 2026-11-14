// The shell's frame deadline (shell.ts): a playing, visible page whose
// engine Worker stops saying it plays (`beat`) is stuck.

import type { HostError } from "./boot.ts";

/**
 * How long a playing, visible page may go without a frame from the engine
 * Worker before the shell gives up on it (a Worker stuck in a loop, or in
 * `Atomics.wait` for a lane that never ends): it terminates the Worker and
 * reports a `hang`. A panicking lane is caught sooner, at the lane boundary.
 */
// Fork s61: 120 s (was 15 s): a first frame that compiles and uploads a large scene's pipelines and
// textures on a slow GPU (a cold shader cache on Windows) took more than 15 s and was stopped as a hang.
export const FRAME_DEADLINE_MS = 120_000;

/** What the deadline looks at, and what it does when the Worker is stuck. */
export interface Watched {
  /** Whether the clock ticks with the display. */
  playing(): boolean;
  /** Whether the engine already stopped. */
  stopped(): boolean;
  /** The Worker sent no frame for FRAME_DEADLINE_MS. */
  hung(message: string, error: HostError): void;
}

export class FrameDeadline {
  /** When the Worker last said it plays (`beat`), and the tick it reached. */
  #beat = 0;
  #beatTick: number | null = null;
  /** When the deadline was last looked at: a late look means the page itself was blocked. */
  #watched = 0;
  #watchdog: ReturnType<typeof setInterval> | undefined;

  /** The tick the Worker last said it reached, null before it said any. */
  get tick(): number | null {
    return this.#beatTick;
  }

  /** The Worker said it plays (`beat`), at `tick` when it named one. */
  beat(tick: number | null | undefined): void {
    this.#beat = performance.now();
    this.#beatTick = tick ?? this.#beatTick;
  }

  /** The deadline starts over now (the clock started or stopped). */
  restart(): void {
    this.#beat = performance.now();
  }

  /** Looks at `watched` once a second from now on. */
  start(watched: Watched): void {
    this.#beat = this.#watched = performance.now();
    this.#watchdog = setInterval(() => this.#watch(watched), 1000);
  }

  stop(): void {
    clearInterval(this.#watchdog);
  }

  /**
   * The frame deadline: a playing page whose Worker sent no `beat` for
   * FRAME_DEADLINE_MS is stuck; the Worker is terminated and the page told.
   * A hidden page (no display frames) and a paused clock do not count.
   */
  #watch(watched: Watched): void {
    const now = performance.now();
    const late = now - this.#watched > 3000;
    this.#watched = now;
    const hidden = globalThis.document?.visibilityState === "hidden";
    // A page that was blocked or throttled itself has not been hearing the Worker: start over.
    if (!watched.playing() || hidden || watched.stopped() || late) {
      this.#beat = now;
      return;
    }
    if (now - this.#beat < FRAME_DEADLINE_MS) return;
    const tick = this.#beatTick;
    const message = `the engine Worker sent no frame for ${FRAME_DEADLINE_MS / 1000} s${tick === null ? "" : ` (last at tick ${tick})`}: it is stuck, so it was stopped`;
    watched.hung(message, { kind: "hang", message, tick, lane: null, location: null });
  }
}
