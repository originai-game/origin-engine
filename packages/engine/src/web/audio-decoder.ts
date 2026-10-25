// The web host's decoder Worker (crates/og-audio/README.md §Decoding beside
// the tick): the sounds' QOA frames are decoded off the engine Worker, so a
// decode never lengthens a frame. After each audio pump the engine Worker
// takes the bank's decode jobs (og_web_audio_decode_jobs: the frames the
// tick's decode-ahead planned from the playheads, and the frames of sounds
// just loaded), posts them here, and answers the bank with the samples
// (og_web_audio_put_decoded) when they come back. A frame the mixer reaches
// first is decoded by the engine itself, to the same samples, so the mix
// never depends on this Worker's timing: only where the work ran does.
//
// The Worker is started from a `blob:` URL of `decodeQoa`'s own source (as
// the lanes pool's Workers are, docs/web-export.md §Hosting), so the build
// needs no second bundle. Where a Worker cannot start (a CSP without
// `worker-src blob:`, whether the constructor throws or the Worker fails
// to load) no decoder runs and the engine decodes as it mixes.

/**
 * Decodes a QOA file (https://qoaformat.org; og-cook's `src/qoa.rs`, which
 * the engine decodes with) to interleaved 16-bit samples, or null when it is
 * malformed. The bank's jobs are one-frame files. Integer arithmetic only,
 * the Rust decoder's operation for operation, so both give the same samples
 * (`tests/audio-decoder.test.ts` and og-audio's `tests/decode.rs` decode one
 * fixture to one hash).
 *
 * Self-contained: it reads nothing outside its body, because its source is
 * the decoder Worker's.
 */
export function decodeQoa(file: Uint8Array): Int16Array | null {
  const DEQUANT = [
    [1, -1, 3, -3, 5, -5, 7, -7],
    [5, -5, 18, -18, 32, -32, 49, -49],
    [16, -16, 53, -53, 95, -95, 147, -147],
    [34, -34, 113, -113, 203, -203, 315, -315],
    [63, -63, 210, -210, 378, -378, 588, -588],
    [104, -104, 345, -345, 621, -621, 966, -966],
    [158, -158, 528, -528, 950, -950, 1477, -1477],
    [228, -228, 760, -760, 1368, -1368, 2128, -2128],
    [316, -316, 1053, -1053, 1895, -1895, 2947, -2947],
    [422, -422, 1405, -1405, 2529, -2529, 3934, -3934],
    [548, -548, 1828, -1828, 3290, -3290, 5117, -5117],
    [696, -696, 2320, -2320, 4176, -4176, 6496, -6496],
    [868, -868, 2893, -2893, 5207, -5207, 8099, -8099],
    [1064, -1064, 3548, -3548, 6386, -6386, 9933, -9933],
    [1286, -1286, 4288, -4288, 7718, -7718, 12005, -12005],
    [1536, -1536, 5120, -5120, 9216, -9216, 14336, -14336],
  ];
  const size = file.byteLength;
  const view = new DataView(file.buffer, file.byteOffset, size);
  if (size < 8 || view.getUint32(0) !== 0x716f6166) return null;
  const total = view.getUint32(4);
  const history = new Int32Array(32);
  const weights = new Int32Array(32);
  let out: Int16Array | null = null;
  let channels = 0;
  let rate = 0;
  let pos = 8;
  let decoded = 0;
  while (decoded < total) {
    if (pos + 8 > size) return null;
    const ch = file[pos]!;
    const fr = (file[pos + 1]! << 16) | (file[pos + 2]! << 8) | file[pos + 3]!;
    const len = view.getUint16(pos + 4);
    const bytes = view.getUint16(pos + 6);
    if (ch === 0 || ch > 8 || len === 0 || len > 5120) return null;
    if (channels === 0) {
      channels = ch;
      rate = fr;
      out = new Int16Array(total * ch);
    } else if (ch !== channels || fr !== rate) {
      return null;
    }
    const slices = Math.ceil(len / 20);
    if (bytes !== 8 + 16 * ch + 8 * slices * ch || pos + bytes > size || decoded + len > total) return null;
    pos += 8;
    for (let c = 0; c < ch; c++) {
      for (let i = 0; i < 4; i++) {
        history[c * 4 + i] = view.getInt16(pos + i * 2);
        weights[c * 4 + i] = view.getInt16(pos + 8 + i * 2);
      }
      pos += 16;
    }
    for (let start = 0; start < len; start += 20) {
      for (let c = 0; c < ch; c++) {
        const hi = view.getUint32(pos);
        const lo = view.getUint32(pos + 4);
        pos += 8;
        const row = DEQUANT[hi >>> 28]!;
        const h = c * 4;
        const end = Math.min(start + 20, len);
        for (let i = start; i < end; i++) {
          // Residual j of the slice sits at bits 57 - 3j .. 59 - 3j.
          const shift = 57 - 3 * (i - start);
          const q = shift >= 32 ? (hi >>> (shift - 32)) & 7 : shift > 29 ? ((lo >>> shift) | (hi << (32 - shift))) & 7 : (lo >>> shift) & 7;
          let p = 0;
          for (let k = 0; k < 4; k++) p = (p + Math.imul(weights[h + k]!, history[h + k]!)) | 0;
          const dequantized = row[q]!;
          let sample = (p >> 13) + dequantized;
          sample = sample < -32768 ? -32768 : sample > 32767 ? 32767 : sample;
          out![(decoded + i) * ch + c] = sample;
          const delta = dequantized >> 4;
          for (let k = 0; k < 4; k++) weights[h + k] = (weights[h + k]! + (history[h + k]! < 0 ? -delta : delta)) | 0;
          history[h] = history[h + 1]!;
          history[h + 1] = history[h + 2]!;
          history[h + 2] = history[h + 3]!;
          history[h + 3] = sample;
        }
      }
    }
    decoded += len;
  }
  return out ?? new Int16Array(0);
}

/** A batch of jobs, engine Worker to decoder: each one-frame QOA file under its key. */
export interface DecodeRequest {
  readonly keys: number[];
  readonly files: Uint8Array[];
}

/** The decoder's answer: per key, the samples (null: the file was malformed). */
export interface DecodeReply {
  readonly keys: number[];
  readonly samples: Array<Int16Array | null>;
}

/** The decoder Worker's whole source: `decodeQoa` and a loop answering `DecodeRequest`s. */
export function decoderSource(): string {
  return `const decodeQoa = ${decodeQoa.toString()};
self.onmessage = (e) => {
  const { keys, files } = e.data;
  const samples = files.map((f) => decodeQoa(f));
  self.postMessage({ keys, samples }, samples.filter((s) => s).map((s) => s.buffer));
};
`;
}

/** The glue members the decoder uses (crates/og-host-web/src/audio.rs); absent in an older engine module. */
export interface DecoderGlue {
  og_web_audio_decode_jobs?(engine: number, max: number): unknown[];
  og_web_audio_put_decoded?(engine: number, key: number, samples: Int16Array): boolean;
}

/** Jobs the engine Worker keeps with the decoder at once (a third of a second of a 48 kHz voice each). */
export const DECODE_IN_FLIGHT = 32;

/** The engine Worker's side of one engine's decoder Worker. */
export class AudioDecoder {
  readonly #glue: Required<DecoderGlue>;
  readonly #engine: number;
  readonly #worker: Worker;
  readonly #url: string | null;
  /** Jobs posted and not answered. */
  inFlight = 0;
  /** Frames answered and taken by the bank. */
  decoded = 0;
  /**
   * False once the Worker failed (a browser that refuses the `blob:` script
   * after constructing the Worker reports it as an error event): nothing is
   * fed to it again, and the engine decodes as it mixes.
   */
  alive = true;

  private constructor(glue: Required<DecoderGlue>, engine: number, worker: Worker, url: string | null) {
    this.#glue = glue;
    this.#engine = engine;
    this.#worker = worker;
    this.#url = url;
    worker.onmessage = (e: MessageEvent<DecodeReply>) => this.#answer(e.data);
    worker.onerror = () => {
      this.alive = false;
    };
  }

  /**
   * The decoder for `engine`, or null where none can run (the module has no
   * decode members, or the Worker constructor refuses a `blob:` URL): then
   * the engine decodes each frame as it mixes it.
   */
  static start(glue: DecoderGlue, engine: number, spawn?: () => Worker): AudioDecoder | null {
    const { og_web_audio_decode_jobs: jobs, og_web_audio_put_decoded: put } = glue;
    if (!jobs || !put) return null;
    let url: string | null = null;
    try {
      let worker: Worker;
      if (spawn) {
        worker = spawn();
      } else {
        url = URL.createObjectURL(new Blob([decoderSource()], { type: "text/javascript" }));
        worker = new Worker(url);
      }
      return new AudioDecoder({ og_web_audio_decode_jobs: jobs.bind(glue), og_web_audio_put_decoded: put.bind(glue) }, engine, worker, url);
    } catch {
      if (url) URL.revokeObjectURL(url);
      return null;
    }
  }

  /** After a pump: hand the decoder the bank's next jobs, up to DECODE_IN_FLIGHT outstanding. Returns how many were posted. */
  feed(): number {
    const room = DECODE_IN_FLIGHT - this.inFlight;
    if (room <= 0 || !this.alive) return 0;
    const flat = this.#glue.og_web_audio_decode_jobs(this.#engine, room);
    if (flat.length === 0) return 0;
    const keys: number[] = [];
    const files: Uint8Array[] = [];
    for (let i = 0; i + 2 < flat.length; i += 3) {
      keys.push(flat[i] as number);
      files.push(flat[i + 2] as Uint8Array);
    }
    this.inFlight += keys.length;
    this.#worker.postMessage({ keys, files } satisfies DecodeRequest, files.map((f) => f.buffer as ArrayBuffer));
    return keys.length;
  }

  #answer(reply: DecodeReply): void {
    this.inFlight = Math.max(0, this.inFlight - reply.keys.length);
    reply.keys.forEach((key, i) => {
      const samples = reply.samples[i];
      if (samples && this.#glue.og_web_audio_put_decoded(this.#engine, key, samples)) this.decoded++;
    });
  }

  close(): void {
    this.#worker.terminate();
    if (this.#url) URL.revokeObjectURL(this.#url);
  }
}
