// Audio on the web (docs/design.md §2.2, §4; crates/og-audio/README.md §Host
// contract). Two halves, like input:
//
// - Main thread (`AudioShell`): owns the AudioContext. It is created and
//   resumed once, on the page's first gesture (the one audio unlock), loads
//   the worklet (audio-worklet.ts, bundled), and hands the engine Worker
//   either the SharedArrayBuffer ring the worklet reads (cross-origin
//   isolated) or a MessagePort to the worklet (otherwise), with the
//   context's sample rate, as a `{ type: "audio" }` message.
// - Engine Worker (`WorkerAudio`): on that message opens the engine's mixer
//   at the context's rate (og_web_audio_open), and after every frame's ticks
//   pumps the PCM those ticks cover (og_web_audio_pump) into the ring, or
//   posts it to the port; then hands the sounds' next QOA frames to the
//   decoder Worker (audio-decoder.ts), so decoding runs beside the tick.

import {
  PROCESSOR,
  RING_CAPACITY,
  RING_CHANNELS,
  RING_HEADER,
  RING_PRIME,
  RING_READ,
  RING_SAMPLE_RATE,
  RING_UNDERRUNS,
  RING_WRITE,
  ringAvailable,
  type WorkletOptions,
} from "./audio-worklet.ts";
import { AudioDecoder, type DecoderGlue } from "./audio-decoder.ts";

// ── the ring (producer side) ───────────────────────────────────────────────

/** Ring capacity in frames (a power of two): 8192 is 170 ms at 48 kHz. */
export const AUDIO_RING_FRAMES = 8192;
/** Frames the worklet buffers before playing: 2048 is 43 ms at 48 kHz. */
export const AUDIO_PRIME_FRAMES = 2048;

/** A ring for `frames` (rounded up to a power of two) stereo frames at `sampleRate`. */
export function createAudioRing(sampleRate: number, frames = AUDIO_RING_FRAMES, prime = AUDIO_PRIME_FRAMES): SharedArrayBuffer {
  let capacity = 1;
  while (capacity < frames) capacity *= 2;
  const ring = new SharedArrayBuffer(RING_HEADER * 4 + capacity * 2 * 4);
  const header = new Int32Array(ring, 0, RING_HEADER);
  header[RING_CAPACITY] = capacity;
  header[RING_CHANNELS] = 2;
  header[RING_SAMPLE_RATE] = sampleRate;
  header[RING_PRIME] = Math.min(prime, capacity);
  return ring;
}

/** Frames the producer may write now. */
export function audioRingFree(ring: SharedArrayBuffer): number {
  const header = new Int32Array(ring, 0, RING_HEADER);
  return header[RING_CAPACITY]! - ringAvailable(header);
}

/** Producer: appends interleaved stereo frames; returns how many fit. */
export function writeAudioRing(ring: SharedArrayBuffer, pcm: Float32Array): number {
  const header = new Int32Array(ring, 0, RING_HEADER);
  const data = new Float32Array(ring, RING_HEADER * 4);
  const capacity = header[RING_CAPACITY]!;
  const mask = capacity - 1;
  const write = Atomics.load(header, RING_WRITE);
  const frames = Math.min(pcm.length >> 1, capacity - ringAvailable(header));
  for (let i = 0; i < frames; i++) {
    const at = ((write + i) & mask) * 2;
    data[at] = pcm[i * 2]!;
    data[at + 1] = pcm[i * 2 + 1]!;
  }
  Atomics.store(header, RING_WRITE, (write + frames) | 0);
  return frames;
}

/** Underrun frames the worklet has played so far, and frames buffered. */
export function audioRingState(ring: SharedArrayBuffer): { buffered: number; underruns: number; read: number } {
  const header = new Int32Array(ring, 0, RING_HEADER);
  return { buffered: ringAvailable(header), underruns: Atomics.load(header, RING_UNDERRUNS), read: Atomics.load(header, RING_READ) };
}

// ── main thread ────────────────────────────────────────────────────────────

/** The message the shell posts to the engine Worker once audio is unlocked. */
export interface AudioMessage {
  readonly type: "audio";
  readonly sampleRate: number;
  readonly ring?: SharedArrayBuffer;
  readonly port?: MessagePort;
  /** The context's baseLatency plus outputLatency, seconds (what the device adds after the worklet). */
  readonly latency?: number;
}

/** A later report of the context's latency (outputLatency moves as the device settles). */
export interface AudioLatencyMessage {
  readonly type: "audio-latency";
  readonly latency: number;
}

/** The context's own latency after the worklet: baseLatency and outputLatency, seconds (0 where a browser reports none). */
export function contextLatency(context: { baseLatency?: number; outputLatency?: number }): number {
  const n = (v: number | undefined) => (typeof v === "number" && Number.isFinite(v) && v > 0 ? v : 0);
  return n(context.baseLatency) + n(context.outputLatency);
}

export interface AudioShellOptions {
  /** The engine Worker. */
  readonly worker: Worker;
  /** URL of the bundled audio-worklet module. */
  readonly worklet: string;
  /** Where the first gesture is listened for (default: the window). */
  readonly target?: EventTarget;
  /** Use the ring when cross-origin isolated (default true). */
  readonly ring?: boolean;
}

const GESTURES = ["pointerdown", "keydown", "touchend"] as const;

/**
 * The page's audio: nothing until the first gesture, then one AudioContext,
 * the worklet node, and the transport to the engine Worker.
 */
export class AudioShell {
  readonly #options: AudioShellOptions;
  #context: AudioContext | null = null;
  #node: AudioWorkletNode | null = null;
  #ring: SharedArrayBuffer | null = null;
  #unlocking: Promise<AudioContext> | null = null;
  readonly #off: Array<() => void> = [];

  constructor(options: AudioShellOptions) {
    this.#options = options;
  }

  /** Unlock on the first pointer, key or touch gesture (once). */
  listen(): void {
    const target = this.#options.target ?? (globalThis as unknown as EventTarget);
    const once = () => {
      for (const off of this.#off.splice(0)) off();
      void this.unlock();
    };
    for (const type of GESTURES) {
      target.addEventListener(type, once, { capture: true });
      this.#off.push(() => target.removeEventListener(type, once, { capture: true }));
    }
  }

  /** Create and resume the AudioContext (call inside a user gesture). Idempotent. */
  unlock(): Promise<AudioContext> {
    this.#unlocking ??= this.#start();
    return this.#unlocking;
  }

  async #start(): Promise<AudioContext> {
    const context = new AudioContext({ latencyHint: "interactive" });
    // resume() first: it must run inside the gesture's task.
    const resumed = context.resume();
    await context.audioWorklet.addModule(this.#options.worklet);
    const isolated = (this.#options.ring ?? true) && globalThis.crossOriginIsolated === true;
    const sampleRate = context.sampleRate;
    let processorOptions: WorkletOptions;
    let message: AudioMessage;
    let channel: MessageChannel | null = null;
    if (isolated) {
      this.#ring = createAudioRing(sampleRate);
      processorOptions = { ring: this.#ring };
      message = { type: "audio", sampleRate, ring: this.#ring };
    } else {
      channel = new MessageChannel();
      processorOptions = { prime: AUDIO_PRIME_FRAMES, maxQueued: Math.round(sampleRate / 2) };
      message = { type: "audio", sampleRate, port: channel.port2 };
    }
    this.#node = new AudioWorkletNode(context, PROCESSOR, {
      numberOfInputs: 0,
      numberOfOutputs: 1,
      outputChannelCount: [2],
      processorOptions,
    });
    if (channel) this.#node.port.postMessage({ type: "port", port: channel.port1 }, [channel.port1]);
    this.#node.connect(context.destination);
    message = { ...message, latency: contextLatency(context) };
    this.#options.worker.postMessage(message, message.port ? [message.port] : []);
    await resumed;
    this.#context = context;
    // outputLatency is known once the context runs and moves with the device: report changes.
    let told = message.latency ?? 0;
    const report = () => {
      const now = contextLatency(context);
      if (Math.abs(now - told) < 0.0005) return;
      told = now;
      this.#options.worker.postMessage({ type: "audio-latency", latency: now } satisfies AudioLatencyMessage);
    };
    report();
    const timer = setInterval(report, 2000);
    this.#off.push(() => clearInterval(timer));
    return context;
  }

  get context(): AudioContext | null {
    return this.#context;
  }

  /** `ring`, `message`, or `locked` before the first gesture. */
  get transport(): "ring" | "message" | "locked" {
    if (!this.#context) return "locked";
    return this.#ring ? "ring" : "message";
  }

  /** The ring's fill and underruns (ring transport only). */
  ringState(): { buffered: number; underruns: number } | null {
    return this.#ring ? audioRingState(this.#ring) : null;
  }

  /** Suspend or resume output (the engine keeps ticking; a pause menu uses og.Audio instead). */
  async suspend(on: boolean): Promise<void> {
    if (!this.#context) return;
    if (on) await this.#context.suspend();
    else await this.#context.resume();
  }

  async dispose(): Promise<void> {
    for (const off of this.#off.splice(0)) off();
    this.#node?.disconnect();
    await this.#context?.close();
    this.#context = null;
  }
}

// ── engine Worker ──────────────────────────────────────────────────────────

/** The wasm-bindgen exports of crates/og-host-web/src/audio.rs. */
export interface WebAudioGlue extends DecoderGlue {
  og_web_audio_open(engine: number, sampleRate: number): string;
  og_web_audio_pump(engine: number, maxFrames: number): Float32Array;
  og_web_audio_stats(engine: number): string;
  og_web_audio_close(engine: number): boolean;
  og_web_audio_insert_qoa(engine: number, id: string, qoa: Uint8Array): string;
  og_web_audio_latency(engine: number, seconds: number): string;
}

type Wire<T> = { ok: true; value: T } | { ok: false; reason: string };

/** Without a ring, the largest chunk one pump posts (a quarter second at 48 kHz). */
const MAX_CHUNK_FRAMES = 12_000;

/** The Worker side of an engine's audio output. */
export class WorkerAudio {
  readonly #glue: WebAudioGlue;
  readonly #engine: number;
  readonly #ring: SharedArrayBuffer | null;
  readonly #port: MessagePort | null;
  /** Frames the ring had no room for (the worklet is behind). */
  overflow = 0;

  readonly #sampleRate: number;
  /** The decoder Worker; null where none can start (the engine then decodes as it mixes). */
  readonly #decoder: AudioDecoder | null;

  private constructor(glue: WebAudioGlue, engine: number, message: AudioMessage) {
    this.#glue = glue;
    this.#engine = engine;
    this.#ring = message.ring ?? null;
    this.#port = message.port ?? null;
    this.#sampleRate = message.sampleRate;
    this.#decoder = AudioDecoder.start(glue, engine);
  }

  /** Open the engine's mixer at the context's rate, on the shell's `audio` message. */
  static attach(glue: WebAudioGlue, engine: number, message: AudioMessage): Wire<WorkerAudio> {
    const opened = JSON.parse(glue.og_web_audio_open(engine, message.sampleRate)) as Wire<number>;
    if (!opened.ok) return opened;
    const audio = new WorkerAudio(glue, engine, message);
    audio.latency(message.latency ?? 0);
    return { ok: true, value: audio };
  }

  /**
   * The context's latency (`contextLatency`) arrived: og.audio.outputLatency
   * is it plus the worklet's prime (the frames it buffers before it plays),
   * written before the next tick. Call between frames.
   */
  latency(context: number): void {
    const seconds = context + AUDIO_PRIME_FRAMES / Math.max(1, this.#sampleRate);
    this.#glue.og_web_audio_latency(this.#engine, seconds);
  }

  /**
   * After a frame's ticks: ship the audio they cover, then give the decoder
   * Worker the frames the ticks' decode-ahead planned. Returns the frames shipped.
   */
  pump(): number {
    const shipped = this.#ship();
    this.#decoder?.feed();
    return shipped;
  }

  #ship(): number {
    if (this.#ring) {
      const pcm = this.#glue.og_web_audio_pump(this.#engine, audioRingFree(this.#ring));
      const wrote = writeAudioRing(this.#ring, pcm);
      this.overflow += (pcm.length >> 1) - wrote;
      return wrote;
    }
    const pcm = this.#glue.og_web_audio_pump(this.#engine, MAX_CHUNK_FRAMES);
    if (pcm.length > 0 && this.#port) this.#port.postMessage(pcm, [pcm.buffer]);
    return pcm.length >> 1;
  }

  /** Whether a decoder Worker decodes beside the tick (false: the engine decodes as it mixes). */
  get decoding(): boolean {
    return this.#decoder !== null;
  }

  /** The mixer's counters. */
  stats(): Wire<Record<string, number>> {
    return JSON.parse(this.#glue.og_web_audio_stats(this.#engine)) as Wire<Record<string, number>>;
  }

  /** Add a sound (a cooked sound's QOA bytes); its handle is what AudioSource.sound takes. */
  insertSound(id: string, qoa: Uint8Array): Wire<number> {
    return JSON.parse(this.#glue.og_web_audio_insert_qoa(this.#engine, id, qoa)) as Wire<number>;
  }

  close(): void {
    this.#glue.og_web_audio_close(this.#engine);
    this.#port?.close();
    this.#decoder?.close();
  }
}
