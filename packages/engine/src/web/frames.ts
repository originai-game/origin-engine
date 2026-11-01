// Field telemetry's frame times (docs/web-export.md §Telemetry): the engine
// Worker counts each played frame into fixed bins, one comparison chain and
// two increments a frame, and the page takes the counts at most once a
// minute (`call("telemetry")`), so a session's frame-time and hitch
// distributions cost no allocation and no message per frame. Only frames
// the clock played count: a hidden page, a paused clock and the boot do not.

/** The bins' upper edges, ms: 120, 90, 60, 50, 40, 30, 24, 20, 15 and 10 Hz, 4 Hz; the last bin is everything above. */
export const FRAME_BINS = [8.4, 11.2, 16.8, 20.1, 25.1, 33.4, 41.7, 50.1, 66.7, 100, 250] as const;

/** A display interval this long or longer is a hitch (three frames missed at 60 Hz). */
export const HITCH_MS = 50;
/** A display interval this long or longer is a long hitch (a stall the player sees as a freeze). */
export const LONG_HITCH_MS = 250;

/** The frames played since the last take: what a `sample`'s `frames` carries. */
export interface FrameStats {
  /** Frames counted. */
  readonly count: number;
  /** Their display time, ms (the sum of their intervals). */
  readonly ms: number;
  /** The bins' upper edges, ms (FRAME_BINS); each histogram has one count more, the bin above the last edge. */
  readonly bins: readonly number[];
  /** Display intervals (this frame's timestamp minus the last one's) by bin. */
  readonly interval: readonly number[];
  /** The engine's work per frame (input, ticks, render submission) by bin. */
  readonly work: readonly number[];
  /** Intervals of HITCH_MS or more, and of LONG_HITCH_MS or more. */
  readonly hitches: number;
  readonly longHitches: number;
  /** The longest interval, ms. */
  readonly max: number;
}

const binOf = (ms: number): number => {
  let i = 0;
  while (i < FRAME_BINS.length && ms > FRAME_BINS[i]!) i++;
  return i;
};

/** The Worker's counts: `add` per played frame, `take` answers them and starts again. */
export class FrameCounter {
  #interval = new Uint32Array(FRAME_BINS.length + 1);
  #work = new Uint32Array(FRAME_BINS.length + 1);
  #count = 0;
  #ms = 0;
  #hitches = 0;
  #long = 0;
  #max = 0;

  add(interval: number, work: number): void {
    this.#interval[binOf(interval)]!++;
    this.#work[binOf(work)]!++;
    this.#count++;
    this.#ms += interval;
    if (interval >= HITCH_MS) this.#hitches++;
    if (interval >= LONG_HITCH_MS) this.#long++;
    if (interval > this.#max) this.#max = interval;
  }

  take(): FrameStats {
    const out: FrameStats = {
      count: this.#count,
      ms: Math.round(this.#ms),
      bins: [...FRAME_BINS],
      interval: Array.from(this.#interval),
      work: Array.from(this.#work),
      hitches: this.#hitches,
      longHitches: this.#long,
      max: Math.round(this.#max * 10) / 10,
    };
    this.#interval.fill(0);
    this.#work.fill(0);
    this.#count = this.#ms = this.#hitches = this.#long = this.#max = 0;
    return out;
  }
}

