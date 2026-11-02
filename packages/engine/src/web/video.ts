// Streamed video on the web host (G3-2, docs/design.md §2.9): the engine
// Worker's side of every stream's decoders. The runtime hands each stream's
// AV1 units over once (`og_web_video_stream`'s hook) and asks, for every
// player each frame, for the frame its player is at (`want`); the answer is
// a VideoFrame the renderer copies into the player's stream texture on the
// GPU (copyExternalImageToTexture) and closes, or null while nothing newer
// is decoded (the texture keeps its frame). Nothing here is waited on, and
// nothing it answers reaches the world: the player's time is the tick's.
//
// Two decoders, one plan (`planStep`, og-video's `stream::plan`):
//
// - WebCodecs' VideoDecoder, in this Worker, where the browser decodes AV1
//   (`VideoDecoder.isConfigSupported`): asynchronous, never blocks a frame.
// - Otherwise rav1d built for wasm32-wasip1 (crates/og-video-wasm,
//   dist/av1/og-av1.wasm), fetched only then, in a Worker of its own made
//   from `av1WorkerSource()`; it answers packed I420 planes, which become a
//   VideoFrame the browser converts on the GPU.
//
// Each decodes ahead of its player by up to AHEAD frames and, for a frame
// behind it or past the next key frame, starts again at the key frame at
// or before it; a looping player decodes on into frame 0.

/** Frames decoded ahead of the one wanted (VideoFrames hold decoder memory). */
export const AHEAD = 4;

/**
 * The decode plan, og-video's `stream::plan`: `"decode"` unit `pos` next,
 * `"idle"`, or a key frame to start again at. `has(f)`: frame f is decoded
 * or being decoded; `count`: how many are. Self-contained: the fallback
 * decoder Worker's source includes it.
 */
export function planStep(n: number, keys: ArrayLike<number>, pos: number, has: (f: number) => boolean, count: number, want: number, looped: boolean, ahead: number): "decode" | "idle" | number {
  want = Math.min(want, n - 1);
  if (has(want)) {
    if (count >= ahead || (pos >= n && !looped)) return "idle";
    if (pos >= n) return 0;
    return "decode";
  }
  let key = keys[0]!;
  for (let i = 0; i < keys.length && keys[i]! <= want; i++) key = keys[i]!;
  if (want < pos || key > pos) return key;
  return "decode";
}

/**
 * The rav1d wasm module (`og-av1.wasm`, a WASI reactor) instantiated from
 * its bytes, with the few WASI calls rav1d's std makes answered here: a
 * monotonic clock, its log to the console, no environment. Self-contained:
 * the fallback decoder Worker's source includes it (and `bun` runs it in
 * tests).
 */
export async function av1Module(bytes: BufferSource) {
  let memory: WebAssembly.Memory | null = null;
  const view = () => new DataView(memory!.buffer);
  const wasi = {
    environ_sizes_get: (count: number, size: number) => {
      view().setUint32(count, 0, true);
      view().setUint32(size, 0, true);
      return 0;
    },
    environ_get: () => 0,
    clock_time_get: (_id: number, _precision: bigint, out: number) => {
      view().setBigUint64(out, BigInt(Math.round(performance.now() * 1e6)), true);
      return 0;
    },
    fd_write: (_fd: number, iovs: number, len: number, written: number) => {
      let n = 0;
      let text = "";
      for (let i = 0; i < len; i++) {
        const ptr = view().getUint32(iovs + i * 8, true);
        const l = view().getUint32(iovs + i * 8 + 4, true);
        text += new TextDecoder().decode(new Uint8Array(memory!.buffer, ptr, l));
        n += l;
      }
      if (text.trim()) console.warn(`og-av1: ${text.trim()}`);
      view().setUint32(written, n, true);
      return 0;
    },
    proc_exit: (code: number) => {
      throw new Error(`og-av1 exited with ${code}`);
    },
    sched_yield: () => 0,
  };
  const { instance } = (await WebAssembly.instantiate(bytes, { wasi_snapshot_preview1: wasi })) as unknown as WebAssembly.WebAssemblyInstantiatedSource;
  const x = instance.exports as Record<string, (...a: number[]) => number> & { memory: WebAssembly.Memory };
  memory = x.memory;
  return {
    open: (): number => x.og_av1_open!(),
    close: (h: number) => void x.og_av1_close!(h),
    flush: (h: number) => void x.og_av1_flush!(h),
    /** Decodes a unit: the picture it shows as packed I420 (a copy), or null for none; throws on a decoding error. */
    decode(h: number, unit: Uint8Array): { width: number; height: number; i420: Uint8Array } | null {
      const ptr = x.og_av1_alloc!(unit.length);
      new Uint8Array(memory!.buffer, ptr, unit.length).set(unit);
      const r = x.og_av1_decode!(h, ptr, unit.length);
      x.og_av1_free!(ptr, unit.length);
      if (r < 0) throw new Error("rav1d could not decode a unit");
      if (r === 0) return null;
      const width = x.og_av1_width!(h);
      const height = x.og_av1_height!(h);
      const cw = (width + 1) >> 1;
      const ch = (height + 1) >> 1;
      const len = width * height + 2 * cw * ch;
      return { width, height, i420: new Uint8Array(memory!.buffer, x.og_av1_picture!(h), len).slice() };
    },
  };
}

/** A stream's units: frame n is `data[offset[n] .. offset[n] + size[n]]`, key frames ascending in `keys`. */
export interface StreamUnits {
  readonly width: number;
  readonly height: number;
  readonly fps: number;
  readonly frames: number;
  /** offset, size, key (1 or 0) per frame. */
  readonly table: Uint32Array;
  readonly data: Uint8Array;
  readonly keys: Uint32Array;
}

/** A stream's units from its table and bytes (as `og_web_video_stream`'s hook hands them over). */
export function streamUnits(width: number, height: number, fps: number, table: Uint32Array, data: Uint8Array): StreamUnits {
  const frames = table.length / 3;
  const keys: number[] = [];
  for (let i = 0; i < frames; i++) if (table[i * 3 + 2]) keys.push(i);
  return { width, height, fps, frames, table, data, keys: Uint32Array.from(keys) };
}

function unitOf(s: StreamUnits, n: number): { data: Uint8Array; key: boolean } {
  const o = s.table[n * 3]!;
  return { data: s.data.subarray(o, o + s.table[n * 3 + 1]!), key: s.table[n * 3 + 2] === 1 };
}

/** WebCodecs' codec string for a stream: AV1 main profile, 8-bit, a level its size fits. */
export function av1Codec(width: number, height: number): string {
  const level = width * height <= 2048 * 1152 ? "08" : width * height <= 4096 * 2304 ? "12" : "16";
  return `av01.0.${level}M.08`;
}

/** The decoder this browser plays streams with, as og.video.capabilities reports it. */
export interface VideoDecoderChoice {
  readonly name: "webcodecs" | "rav1d-wasm" | "none";
  readonly hardware: boolean;
  /** The largest stream it plays in real time at 30 fps. */
  readonly maxWidth: number;
  readonly maxHeight: number;
}

/**
 * The largest stream the rav1d wasm fallback decodes in real time at
 * 30 fps on one thread (og-video's `FALLBACK_MAX`; measured in Safari 27 on
 * Apple silicon: 720p 46 fps on worst-case footage, 1080p 19). A larger
 * stream plays its 720p rendition (`<id>#stream720`, which the cook adds).
 */
export const FALLBACK_MAX = { width: 1280, height: 720 } as const;

/** The fallback module in the package (a built page passes `av1/og-av1.wasm` beside it). */
export const FALLBACK_URL = new URL("../../dist/av1/og-av1.wasm", import.meta.url).href;

/** Which decoder this browser plays AV1 with. */
export async function chooseDecoder(): Promise<VideoDecoderChoice> {
  const g = globalThis as { VideoDecoder?: typeof VideoDecoder; VideoFrame?: typeof VideoFrame };
  if (g.VideoDecoder) {
    try {
      const base = { codec: av1Codec(1920, 1080), codedWidth: 1920, codedHeight: 1080 };
      if ((await g.VideoDecoder.isConfigSupported(base)).supported) {
        const hw = await g.VideoDecoder.isConfigSupported({ ...base, hardwareAcceleration: "prefer-hardware" });
        return { name: "webcodecs", hardware: hw.supported === true, maxWidth: 3840, maxHeight: 2160 };
      }
    } catch {
      /* fall back */
    }
  }
  // The fallback hands I420 planes to the browser as a VideoFrame.
  if (g.VideoFrame && typeof WebAssembly === "object") return { name: "rav1d-wasm", hardware: false, maxWidth: FALLBACK_MAX.width, maxHeight: FALLBACK_MAX.height };
  return { name: "none", hardware: false, maxWidth: 0, maxHeight: 0 };
}

/** One player's decoder, as the runtime asks it each frame. */
interface Player {
  /** The frame to show for `frame` (the player is at it), or null when nothing newer is decoded. */
  want(frame: number, looped: boolean): VideoFrame | null;
  close(): void;
}

/** Frames decoded in order, each with its index: what a player keeps until it shows them. */
class Ready<T> {
  readonly items: Array<{ frame: number; value: T }> = [];
  readonly drop: (v: T) => void;
  constructor(drop: (v: T) => void) {
    this.drop = drop;
  }
  has(f: number): boolean {
    return this.items.some((i) => i.frame === f);
  }
  clear(): void {
    for (const i of this.items) this.drop(i.value);
    this.items.length = 0;
  }
  /** For a new wanted frame: the frames before it dropped, or all of them when it is not here. */
  moveTo(f: number): boolean {
    const at = this.items.findIndex((i) => i.frame === f);
    if (at < 0) {
      this.clear();
      return false;
    }
    for (const i of this.items.splice(0, at)) this.drop(i.value);
    return true;
  }
  /**
   * The frame to show for `want`: the newest one at or before it, in decode
   * order up to a loop's wrap; it and the ones before it leave the queue.
   */
  take(want: number): { frame: number; value: T } | null {
    let pick = -1;
    for (let i = 0; i < this.items.length; i++) {
      const f = this.items[i]!.frame;
      if (f > want || (i > 0 && f < this.items[i - 1]!.frame)) break;
      pick = i;
    }
    if (pick < 0) return null;
    const taken = this.items.splice(0, pick + 1);
    for (const i of taken.slice(0, -1)) this.drop(i.value);
    return taken[taken.length - 1]!;
  }
}

/** A player decoding with WebCodecs in this Worker. */
class WebCodecsPlayer implements Player {
  readonly #s: StreamUnits;
  readonly #decoder: VideoDecoder;
  readonly #ready = new Ready<VideoFrame>((f) => f.close());
  /** Units sent and not answered yet. */
  readonly #pending = new Set<number>();
  #pos = 0;
  #keepFrom = 0;
  #want = -1;
  #looped = false;
  #shown = -1;
  #broken = false;

  constructor(s: StreamUnits) {
    this.#s = s;
    this.#decoder = new VideoDecoder({
      output: (f) => this.#output(f),
      error: () => {
        this.#broken = true;
      },
    });
    this.#configure();
  }

  #configure(): void {
    this.#decoder.configure({ codec: av1Codec(this.#s.width, this.#s.height), codedWidth: this.#s.width, codedHeight: this.#s.height, optimizeForLatency: true });
  }

  #output(f: VideoFrame): void {
    const n = f.timestamp;
    this.#pending.delete(n);
    if (n < this.#keepFrom && !this.#looped) f.close();
    else this.#ready.items.push({ frame: n, value: f });
    this.#pump();
  }

  #pump(): void {
    const s = this.#s;
    for (;;) {
      if (this.#broken || this.#want < 0) return;
      const has = (f: number) => this.#ready.has(f) || this.#pending.has(f);
      const step = planStep(s.frames, s.keys, this.#pos, has, this.#ready.items.length + this.#pending.size, this.#want, this.#looped, AHEAD);
      if (step === "idle") return;
      if (typeof step === "number") {
        // A wrap goes on from frame 0 with what is ready; a seek starts over.
        if (this.#pos < s.frames || step !== 0) {
          this.#decoder.reset();
          this.#configure();
          this.#pending.clear();
          this.#ready.clear();
          this.#keepFrom = Math.min(this.#want, s.frames - 1);
        } else {
          this.#keepFrom = 0;
        }
        this.#pos = step;
      }
      const n = this.#pos++;
      const u = unitOf(s, n);
      this.#pending.add(n);
      try {
        this.#decoder.decode(new EncodedVideoChunk({ type: u.key ? "key" : "delta", timestamp: n, data: u.data }));
      } catch {
        this.#broken = true;
        return;
      }
    }
  }

  want(frame: number, looped: boolean): VideoFrame | null {
    frame = Math.min(frame, this.#s.frames - 1);
    if (frame !== this.#want || looped !== this.#looped) {
      if (frame !== this.#want && !this.#ready.moveTo(frame) && !this.#pending.has(frame)) this.#keepFrom = frame;
      this.#want = frame;
      this.#looped = looped;
    }
    this.#pump();
    const t = this.#ready.take(frame);
    if (!t) return null;
    if (t.frame === this.#shown) {
      t.value.close();
      return null;
    }
    this.#shown = t.frame;
    return t.value;
  }

  close(): void {
    this.#ready.clear();
    if (this.#decoder.state !== "closed") this.#decoder.close();
  }
}

/** The fallback decoder Worker's loop (self-contained; its source is av1WorkerSource()). */
function av1WorkerMain(): void {
  type Ctx = { postMessage(m: unknown, t?: Transferable[]): void; onmessage: ((e: MessageEvent) => void) | null };
  const ctx = self as unknown as Ctx;
  const g = globalThis as unknown as { av1Module: typeof av1Module; planStep: typeof planStep; AHEAD: number };
  let mod: Awaited<ReturnType<typeof av1Module>> | null = null;
  const streams = new Map<number, { frames: number; table: Uint32Array; data: Uint8Array; keys: Uint32Array }>();
  interface P {
    stream: number;
    h: number;
    pos: number;
    keepFrom: number;
    want: number;
    looped: boolean;
    /** Frames posted to the engine Worker and not yet passed by its player, in order. */
    sent: number[];
  }
  const players = new Map<number, P>();
  let scheduled = false;
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    setTimeout(step, 0);
  };
  // One unit per turn, so a new `want` is read between units.
  function step(): void {
    scheduled = false;
    for (const [id, p] of players) {
      const s = streams.get(p.stream);
      if (!s || !mod || p.want < 0) continue;
      const st = g.planStep(s.frames, s.keys, p.pos, (f) => p.sent.includes(f), p.sent.length, p.want, p.looped, g.AHEAD);
      if (st === "idle") continue;
      if (typeof st === "number") {
        if (p.pos < s.frames || st !== 0) {
          mod.flush(p.h);
          p.sent.length = 0;
          p.keepFrom = Math.min(p.want, s.frames - 1);
        } else {
          p.keepFrom = 0;
        }
        p.pos = st;
      }
      const n = p.pos++;
      const o = s.table[n * 3]!;
      try {
        const pic = mod.decode(p.h, s.data.subarray(o, o + s.table[n * 3 + 1]!));
        if (pic && n >= p.keepFrom) {
          p.sent.push(n);
          ctx.postMessage({ type: "frame", player: id, frame: n, width: pic.width, height: pic.height, i420: pic.i420.buffer }, [pic.i420.buffer]);
        }
      } catch (e) {
        ctx.postMessage({ type: "error", player: id, reason: String(e) });
        players.delete(id);
      }
      schedule();
      return;
    }
  }
  ctx.onmessage = async (e: MessageEvent) => {
    const m = e.data;
    if (m.type === "load") {
      try {
        const r = await fetch(m.url);
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        mod = await g.av1Module(await r.arrayBuffer());
        ctx.postMessage({ type: "ready" });
      } catch (err) {
        ctx.postMessage({ type: "failed", reason: `cannot load the AV1 decoder ${m.url}: ${String(err)}` });
      }
    } else if (m.type === "stream") {
      const table = new Uint32Array(m.table);
      const keys: number[] = [];
      for (let i = 0; i < table.length / 3; i++) if (table[i * 3 + 2]) keys.push(i);
      streams.set(m.stream, { frames: table.length / 3, table, data: new Uint8Array(m.data), keys: Uint32Array.from(keys) });
    } else if (m.type === "want") {
      let p = players.get(m.player);
      if (!p && mod) {
        p = { stream: m.stream, h: mod.open(), pos: 0, keepFrom: 0, want: -1, looped: false, sent: [] };
        players.set(m.player, p);
      }
      if (!p) return;
      const at = p.sent.indexOf(m.frame);
      if (at >= 0) p.sent.splice(0, at);
      else if (m.frame !== p.want) {
        p.sent.length = 0;
        p.keepFrom = m.frame;
      }
      p.want = m.frame;
      p.looped = m.looped;
      schedule();
    } else if (m.type === "close") {
      const p = players.get(m.player);
      if (p && mod) mod.close(p.h);
      players.delete(m.player);
    }
  };
}

/** The fallback decoder Worker's whole source. */
export function av1WorkerSource(): string {
  return `const AHEAD = ${AHEAD};
const planStep = ${planStep.toString()};
const av1Module = ${av1Module.toString()};
globalThis.AHEAD = AHEAD; globalThis.planStep = planStep; globalThis.av1Module = av1Module;
(${av1WorkerMain.toString()})();
`;
}

/** I420 planes as a VideoFrame the browser converts (BT.709, limited range: what the cook encodes). */
function i420Frame(width: number, height: number, frame: number, i420: ArrayBuffer): VideoFrame {
  return new VideoFrame(new Uint8Array(i420), {
    format: "I420",
    codedWidth: width,
    codedHeight: height,
    timestamp: frame,
    colorSpace: { primaries: "bt709", transfer: "bt709", matrix: "bt709", fullRange: false },
  });
}

/** A player decoding with rav1d in the fallback Worker. */
class WasmPlayer implements Player {
  readonly #ready = new Ready<{ width: number; height: number; i420: ArrayBuffer }>(() => {});
  #want = -1;
  #looped = false;
  #shown = -1;

  readonly id: number;
  readonly stream: number;
  readonly frames: number;
  readonly worker: Worker;

  constructor(id: number, stream: number, frames: number, worker: Worker) {
    this.id = id;
    this.stream = stream;
    this.frames = frames;
    this.worker = worker;
  }

  /** A frame the Worker decoded. */
  put(frame: number, value: { width: number; height: number; i420: ArrayBuffer }): void {
    this.#ready.items.push({ frame, value });
  }

  want(frame: number, looped: boolean): VideoFrame | null {
    frame = Math.min(frame, this.frames - 1);
    if (frame !== this.#want || looped !== this.#looped) {
      if (frame !== this.#want) this.#ready.moveTo(frame);
      this.#want = frame;
      this.#looped = looped;
      this.worker.postMessage({ type: "want", player: this.id, stream: this.stream, frame, looped });
    }
    const t = this.#ready.take(frame);
    if (!t || t.frame === this.#shown) return null;
    this.#shown = t.frame;
    return i420Frame(t.value.width, t.value.height, t.frame, t.value.i420);
  }

  close(): void {
    this.worker.postMessage({ type: "close", player: this.id });
  }
}

/** Every stream and player of one engine Worker. */
export class VideoStreams {
  readonly #streams = new Map<number, StreamUnits>();
  readonly #players = new Map<number, Player>();
  #worker: Worker | null = null;
  #workerReady: Promise<boolean> | null = null;
  /** Why streams cannot play here, once (reported in the console). */
  failure: string | null = null;

  readonly choice: VideoDecoderChoice;
  /** The fallback module's URL (dist/av1/og-av1.wasm, or av1/og-av1.wasm beside a built page). */
  readonly fallbackUrl: string;

  constructor(choice: VideoDecoderChoice, fallbackUrl: string) {
    this.choice = choice;
    this.fallbackUrl = fallbackUrl;
  }

  /** A stream's units, handed over once by the runtime. */
  stream(key: number, width: number, height: number, fps: number, table: Uint32Array, data: Uint8Array): boolean {
    if (this.choice.name === "none") return false;
    const s = streamUnits(width, height, fps, table.slice(), data.slice());
    this.#streams.set(key, s);
    if (this.choice.name === "rav1d-wasm") {
      this.#startWorker();
      this.#worker?.postMessage({ type: "stream", stream: key, table: s.table.buffer, data: s.data.buffer }, [s.table.buffer, s.data.buffer]);
      // The engine Worker keeps the shape only; the fallback Worker has the units.
      this.#streams.set(key, { ...s, table: new Uint32Array(0), data: new Uint8Array(0) });
    }
    return true;
  }

  #startWorker(): void {
    if (this.#workerReady) return;
    let url: string | null = null;
    try {
      url = URL.createObjectURL(new Blob([av1WorkerSource()], { type: "text/javascript" }));
      const w = new Worker(url);
      this.#worker = w;
      this.#workerReady = new Promise((resolve) => {
        w.onmessage = (e: MessageEvent) => {
          const m = e.data;
          if (m.type === "ready") resolve(true);
          else if (m.type === "failed") {
            this.failure = m.reason;
            console.warn(`og.video: ${m.reason}`);
            resolve(false);
          } else if (m.type === "frame") {
            const p = this.#players.get(m.player);
            if (p instanceof WasmPlayer) p.put(m.frame, { width: m.width, height: m.height, i420: m.i420 });
          } else if (m.type === "error") {
            console.warn(`og.video: player ${m.player}: ${m.reason}`);
          }
        };
      });
      w.postMessage({ type: "load", url: new URL(this.fallbackUrl, location.href).href });
    } catch (e) {
      if (url) URL.revokeObjectURL(url);
      this.failure = `cannot start the AV1 decoder Worker: ${String(e)}`;
      this.#workerReady = Promise.resolve(false);
    }
  }

  /** The frame to show for player `player` of stream `stream`, at `frame`; null while nothing newer is decoded. */
  want(player: number, stream: number, frame: number, looped: boolean): VideoFrame | null {
    let p = this.#players.get(player);
    if (!p) {
      const s = this.#streams.get(stream);
      if (!s) return null;
      if (this.choice.name === "webcodecs") p = new WebCodecsPlayer(s);
      else if (this.choice.name === "rav1d-wasm" && this.#worker) p = new WasmPlayer(player, stream, s.frames, this.#worker);
      else return null;
      this.#players.set(player, p);
    }
    try {
      return p.want(frame, looped);
    } catch (e) {
      if (!this.failure) {
        this.failure = String(e);
        console.warn(`og.video: ${this.failure}`);
      }
      return null;
    }
  }
}
