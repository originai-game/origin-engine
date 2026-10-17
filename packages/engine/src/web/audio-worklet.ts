// The AudioWorklet that plays the engine's mix (docs/design.md §2.2, §2.7;
// crates/og-audio/README.md §Ring). The mixer runs in the engine Worker;
// this processor only copies its PCM to the output, from a SharedArrayBuffer
// ring when the page is cross-origin isolated, else from chunks posted on a
// MessagePort. Loaded with `audioWorklet.addModule(<this module, bundled>)`;
// the processor is `og-audio`. It imports nothing, so it can be loaded on
// its own; audio.ts imports its ring layout and functions.
//
// Ring (one SharedArrayBuffer): an Int32 header of RING_HEADER slots, then
// `capacity * 2` f32 (interleaved stereo frames, starting at byte
// RING_HEADER * 4). Header slots:
//   0 write      frames written, monotonic (int32, wraps)
//   1 read       frames read, monotonic (int32, wraps)
//   2 capacity   frames, a power of two
//   3 channels   2
//   4 underruns  frames of silence played because the ring ran dry
//   5 sampleRate the AudioContext's rate the mixer renders at
//   6 primed     1 once the reader has seen `prime` frames (0 after an underrun)
//   7 prime      frames to buffer before (re)starting
// Single producer (the Worker), single consumer (this processor): the
// producer writes data then publishes `write`; the consumer reads data then
// publishes `read` (Atomics on the header, plain access to the data).

export const RING_HEADER = 8;
export const RING_WRITE = 0;
export const RING_READ = 1;
export const RING_CAPACITY = 2;
export const RING_CHANNELS = 3;
export const RING_UNDERRUNS = 4;
export const RING_SAMPLE_RATE = 5;
export const RING_PRIMED = 6;
export const RING_PRIME = 7;

/** The processor's registered name. */
export const PROCESSOR = "og-audio";

/** Frames waiting in the ring. */
export function ringAvailable(header: Int32Array): number {
  return (Atomics.load(header, RING_WRITE) - Atomics.load(header, RING_READ)) | 0;
}

/**
 * Consumer: fills `left`/`right` (one render quantum) from the ring; returns
 * the frames that were silence because the ring was dry. Playback waits
 * until `prime` frames are buffered, at the start and after a dry spell.
 */
export function readAudioRing(header: Int32Array, data: Float32Array, left: Float32Array, right: Float32Array): number {
  const n = left.length;
  const available = ringAvailable(header);
  if (Atomics.load(header, RING_PRIMED) === 0) {
    if (available < header[RING_PRIME]!) {
      left.fill(0);
      right.fill(0);
      return 0; // still buffering: not an underrun
    }
    Atomics.store(header, RING_PRIMED, 1);
  }
  const mask = header[RING_CAPACITY]! - 1;
  const read = Atomics.load(header, RING_READ);
  const take = Math.min(n, available);
  for (let i = 0; i < take; i++) {
    const at = ((read + i) & mask) * 2;
    left[i] = data[at]!;
    right[i] = data[at + 1]!;
  }
  for (let i = take; i < n; i++) {
    left[i] = 0;
    right[i] = 0;
  }
  Atomics.store(header, RING_READ, (read + take) | 0);
  const dry = n - take;
  if (dry > 0) {
    Atomics.add(header, RING_UNDERRUNS, dry);
    Atomics.store(header, RING_PRIMED, 0);
  }
  return dry;
}

// ── the processor (only where AudioWorkletGlobalScope defines it) ─────────

interface ProcessorPort {
  onmessage: ((e: { data: unknown }) => void) | null;
  postMessage(message: unknown): void;
}

interface WorkletScope {
  registerProcessor?(name: string, ctor: unknown): void;
  AudioWorkletProcessor?: new () => { readonly port: ProcessorPort };
}

/** What the main thread passes as `processorOptions`. */
export interface WorkletOptions {
  /** The ring, when the page is cross-origin isolated. */
  readonly ring?: SharedArrayBuffer;
  /** Without a ring: frames to queue before playing, and the most to keep. */
  readonly prime?: number;
  readonly maxQueued?: number;
}

const scope = globalThis as unknown as WorkletScope;

if (typeof scope.registerProcessor === "function" && scope.AudioWorkletProcessor) {
  const Base = scope.AudioWorkletProcessor;

  class OgAudioProcessor extends Base {
    readonly #header: Int32Array | null;
    readonly #data: Float32Array | null;
    /** Posted chunks (interleaved stereo), oldest first, and the read offset into the first. */
    readonly #queue: Float32Array[] = [];
    #offset = 0;
    #queued = 0;
    #primed = false;
    readonly #prime: number;
    readonly #maxQueued: number;
    #underruns = 0;
    #dropped = 0;
    #quanta = 0;

    constructor(options: { processorOptions?: WorkletOptions }) {
      super();
      const o = options.processorOptions ?? {};
      this.#header = o.ring ? new Int32Array(o.ring, 0, RING_HEADER) : null;
      this.#data = o.ring ? new Float32Array(o.ring, RING_HEADER * 4) : null;
      this.#prime = o.prime ?? 2048;
      this.#maxQueued = o.maxQueued ?? 24_000;
      // The Worker's chunks arrive on a port the main thread hands over.
      this.port.onmessage = (e) => {
        const m = e.data as { type?: string; port?: ProcessorPort };
        if (m.type === "port" && m.port) m.port.onmessage = (c) => this.#push(c.data as Float32Array);
      };
    }

    #push(chunk: Float32Array): void {
      this.#queue.push(chunk);
      this.#queued += chunk.length / 2;
      // A producer far ahead (no backpressure on a port): drop the oldest.
      while (this.#queued > this.#maxQueued && this.#queue.length > 1) {
        const old = this.#queue.shift()!;
        const frames = (old.length - this.#offset) / 2;
        this.#queued -= frames;
        this.#dropped += frames;
        this.#offset = 0;
      }
    }

    #readQueue(left: Float32Array, right: Float32Array): void {
      const n = left.length;
      if (!this.#primed && this.#queued < this.#prime) {
        left.fill(0);
        right.fill(0);
        return;
      }
      this.#primed = true;
      let i = 0;
      while (i < n && this.#queue.length > 0) {
        const chunk = this.#queue[0]!;
        while (i < n && this.#offset < chunk.length) {
          left[i] = chunk[this.#offset]!;
          right[i] = chunk[this.#offset + 1]!;
          this.#offset += 2;
          i++;
        }
        if (this.#offset >= chunk.length) {
          this.#queue.shift();
          this.#offset = 0;
        }
      }
      this.#queued -= i;
      if (i < n) {
        left.fill(0, i);
        right.fill(0, i);
        this.#underruns += n - i;
        this.#primed = false;
      }
    }

    process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
      const out = outputs[0];
      if (!out || out.length === 0) return true;
      const left = out[0]!;
      const right = out[1] ?? left;
      if (this.#header && this.#data) readAudioRing(this.#header, this.#data, left, right);
      else this.#readQueue(left, right);
      // About once a second without a ring: report what the queue did.
      if (!this.#header && ++this.#quanta % 375 === 0) {
        this.port.postMessage({ type: "stats", queued: this.#queued, underruns: this.#underruns, dropped: this.#dropped });
      }
      return true;
    }
  }

  scope.registerProcessor(PROCESSOR, OgAudioProcessor);
}
