// Frame pacing in the engine Worker (og.render.setDisplay's frame cap,
// crates/og-runtime/src/display.rs): requestAnimationFrame fires at the
// display's rate (120 Hz on ProMotion, 144 and more on gaming monitors),
// and a frame cap of 60 or 30 draws only some of those frames. A skipped
// display frame still reads input (the Worker pumps the ring on every
// display frame), and the engine's fixed tick is unchanged: the next drawn
// frame is handed the whole time since the last drawn one, so the
// accumulator runs the ticks it owes. The pacer also measures the display's
// interval, which the engine's automatic quality takes as its budget when
// the cap is the display's rate.

/** Display intervals the measurement keeps: about half a second at 120 Hz. */
const WINDOW = 64;

/** An interval this long is not the display's (a hidden page, a stall). */
const LONGEST_MS = 1000 / 20;

export class FramePacer {
  #intervals: number[] = [];
  #lastDisplay = 0;
  #lastDrawn = 0;
  /** The display's interval as measured (the window's median), ms; 0 before enough frames. */
  displayMs = 0;

  /**
   * One display frame at `now` (its requestAnimationFrame timestamp): whether
   * to draw it under a cap of `maxFps` frames a second (0: every frame). A
   * frame is drawn once the cap's interval has passed since the last drawn
   * one, less half a display interval, so a 60 cap on a 120 Hz display draws
   * every other frame without drifting into a skipped pair by jitter, and a
   * cap at or above the display's rate draws every frame.
   */
  frame(now: number, maxFps: number): boolean {
    if (this.#lastDisplay > 0) {
      const d = now - this.#lastDisplay;
      if (d > 0 && d < LONGEST_MS) {
        this.#intervals.push(d);
        if (this.#intervals.length > WINDOW) this.#intervals.shift();
        if (this.#intervals.length >= 8) {
          const sorted = [...this.#intervals].sort((a, b) => a - b);
          this.displayMs = sorted[sorted.length >> 1]!;
        }
      }
    }
    this.#lastDisplay = now;
    if (!(maxFps > 0) || this.#lastDrawn === 0) {
      this.#lastDrawn = now;
      return true;
    }
    // Before the display is measured, a slack below any display's interval.
    const slack = this.displayMs > 0 ? this.displayMs * 0.5 : 2;
    if (now - this.#lastDrawn + slack < 1000 / maxFps) return false;
    this.#lastDrawn = now;
    return true;
  }

  /** Forget the last drawn frame (play stopped or resumed): the next one draws. */
  reset(): void {
    this.#lastDrawn = 0;
    this.#lastDisplay = 0;
  }
}
