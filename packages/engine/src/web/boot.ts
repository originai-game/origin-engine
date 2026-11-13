// What the engine Worker's boot loads before the engine runs (worker.ts): the
// bundle's files fetched against a deadline and counted for the progress bar,
// its Basis textures transcoded for the renderer's formats (in a pool of
// Workers when there are many), and the fetcher, inflater, image decoder and
// video streams the wasm asks for what it loads later.

import { type BasisTranscoder, TEXTURE_FORMATS, type TextureFormat, loadBasisTranscoder, transcodeKtx2 } from "./basis.ts";
import type { WebExtras } from "./host.ts";
import { FALLBACK_URL, VideoStreams, chooseDecoder } from "./video.ts";

/** The trace's key for loads made before the engine exists (og-runtime's `BEFORE_ENGINE`). */
export const TRACE_BEFORE_ENGINE = 0xffffffff;

/** Where the runtime asks for a bundle's files: `<root>/cooked/<file>` (crates/og-runtime/src/world/assets.rs). */
export function cookedDir(root: string): string {
  return `${root.replace(/\/+$/, "")}/cooked`;
}

/** Bytes received so far by the boot, against what it expects. */
export class Received {
  loaded = 0;
  total: number;
  constructor(total = 0) {
    this.total = total;
  }
  /** Fraction in [0, 1]; grows the total when more arrived than expected. */
  get value(): number {
    if (this.loaded > this.total) this.total = this.loaded;
    return this.total > 0 ? this.loaded / this.total : 0;
  }
}

/**
 * How long one file's fetch may take, its body included, before it fails:
 * generous, since a pack can be large on a slow network; only a fetch that
 * hangs reaches it.
 */
export const FETCH_DEADLINE_MS = 300_000;

/** How many of the first group's files the boot fetches at once. */
export const FETCH_CONCURRENCY = 24;

/**
 * The device the engine renders with, for field telemetry: its limits,
 * features and fallback flag from the wasm (`og_web_gpu`), and its adapter's
 * vendor and architecture from the device itself (the canvas's WebGPU
 * context's configured device, `GPUDevice.adapterInfo`), which wgpu does not
 * carry. Null without a renderer; vendor and architecture empty where the
 * browser has no `getConfiguration` or `adapterInfo`.
 */
export function deviceReport(glue: { og_web_gpu?(engine: number): string }, engine: number, canvas: OffscreenCanvas | undefined): Record<string, unknown> | null {
  let report: Record<string, unknown> | null = null;
  try {
    report = JSON.parse(glue.og_web_gpu?.(engine) ?? "null") as Record<string, unknown> | null;
  } catch {
    return null;
  }
  if (!report) return null;
  let info: { vendor?: unknown; architecture?: unknown; isFallbackAdapter?: unknown } = {};
  try {
    const context = (canvas as unknown as { getContext(kind: string): { getConfiguration?(): { device?: { adapterInfo?: typeof info } } | null } | null } | undefined)?.getContext("webgpu");
    info = context?.getConfiguration?.()?.device?.adapterInfo ?? {};
  } catch {
    /* an older browser: limits and features only */
  }
  return { info: { vendor: info.vendor, architecture: info.architecture, isFallbackAdapter: report["fallback"] === true || info.isFallbackAdapter === true }, limits: report["limits"], features: report["features"] };
}

/**
 * A file of the bundle that did not arrive, told to the page (`{ type:
 * "load-failure", file, reason }`, field telemetry's load failures): `file`
 * is its path under `cooked/` (no origin), `reason` one of `http_<status>`,
 * `timeout` or `network` (no message text).
 */
export function loadFailure(url: string, e: unknown): { type: "load-failure"; file: string; reason: string } {
  const text = e instanceof Error ? `${e.name} ${e.message}` : String(e);
  const status = /HTTP (\d{3})/.exec(text)?.[1];
  const cooked = url.replace(/[?#].*$/, "").split("/cooked/");
  return {
    type: "load-failure",
    file: (cooked.length > 1 ? `cooked/${cooked[cooked.length - 1]}` : url.slice(url.lastIndexOf("/") + 1)).slice(0, 160),
    reason: status ? `http_${status}` : /timeout|abort/i.test(text) ? "timeout" : "network",
  };
}
const tellLoadFailure = (url: string, e: unknown) => {
  try {
    (globalThis as { postMessage?: (m: unknown) => void }).postMessage?.(loadFailure(url, e));
  } catch {
    /* no page to tell */
  }
};

/** Fork s63: the texture bytes one frame uploads at most (4 MB: a 32 MB frame was one 1.3-2.5 s GPU-process task on an RTX 3090). */
export const UPLOAD_BUDGET = 4 * 1024 * 1024;

/**
 * Fork s61: each boot stage's share of the progress bar, so it does not sit at 100 % once the
 * bytes are in: the download (wasm, bundle, fetch) 0-70 %, the Basis transcode 70-85 %, the first
 * tick 85-92 %, the first frame (its pipelines) 92-100 %.
 */
export const BOOT_SPANS: Readonly<Record<string, readonly [number, number]>> = {
  wasm: [0, 0.7],
  bundle: [0, 0.7],
  fetch: [0, 0.7],
  transcode: [0.7, 0.85],
  ready: [0.85, 0.85],
  prime: [0.85, 0.92],
  present: [0.92, 1],
};

/** A stage's fraction (0 to 1) as a place on the whole bar. */
export function onBar(stage: string, value: number): number {
  const [a, b] = BOOT_SPANS[stage] ?? [0, 1];
  return a + (b - a) * Math.max(0, Math.min(1, value));
}

async function bytesOf(url: string, received?: Received, onChunk?: () => void): Promise<Uint8Array> {
  // Fork s61: a fetch that failed (not refused: HTTP 4xx) is tried again, up to 5 times, backing off;
  // its bytes already counted come off the progress first.
  for (let attempt = 0; ; attempt++) {
    const before = received ? received.loaded : 0;
    try {
      return await bytesOnce(url, received, onChunk);
    } catch (e) {
      if (received) received.loaded = before;
      if (attempt >= 5 || /HTTP 4\d\d/.test(String(e))) throw e;
      console.warn(`fetch ${url} failed (${e instanceof Error ? e.message : String(e)}), retry ${attempt + 1}`);
      await new Promise((ok) => setTimeout(ok, 500 * 2 ** attempt));
    }
  }
}

async function bytesOnce(url: string, received?: Received, onChunk?: () => void): Promise<Uint8Array> {
  // A cooked file's name may hold a `#` (`m.gltf.tex0#mips.ogb`, a flipbook's
  // `#array#layer<n>`): sent as is, the browser takes the rest as a fragment.
  const r = await fetch(url.replaceAll("#", "%23"), typeof AbortSignal?.timeout === "function" ? { signal: AbortSignal.timeout(FETCH_DEADLINE_MS) } : {});
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  if (!received || !r.body) {
    const bytes = new Uint8Array(await r.arrayBuffer());
    if (received) received.loaded += bytes.length;
    onChunk?.();
    return bytes;
  }
  const reader = r.body.getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    length += value.length;
    received.loaded += value.length;
    onChunk?.();
  }
  const out = new Uint8Array(length);
  let at = 0;
  for (const c of chunks) {
    out.set(c, at);
    at += c.length;
  }
  return out;
}

export interface Preloaded {
  readonly files: number;
  readonly failed: number;
  /** Basis textures transcoded and handed to the wasm. */
  readonly textures: number;
  /**
   * The encodings they reached the renderer in, joined by `+` in the order
   * first used (`bc7+bc5` on desktop, `astc4x4` on Apple and most Android,
   * `etc2+eacrg11` elsewhere, `rgba8` without a renderer or compression).
   */
  readonly textureFormat: string;
}

/**
 * Streamed video's decoders for this Worker (video.ts): the decoder this
 * browser plays AV1 with, reported by og.video.capabilities; the rav1d
 * fallback module (`av1`, dist/av1/og-av1.wasm by default) is fetched only
 * when WebCodecs cannot decode AV1, by its own Worker, when a stream first
 * plays.
 */
export async function installVideo(glue: WebExtras, av1: string | undefined): Promise<VideoStreams | null> {
  if (!glue.og_web_install_video) return null;
  const choice = await chooseDecoder();
  const streams = new VideoStreams(choice, av1 ?? FALLBACK_URL);
  glue.og_web_install_video(
    (key, width, height, table, data) => streams.stream(key, width, height, 0, table, data),
    (stream, player, frame, looped) => streams.want(player, stream, frame, looped),
    choice.name,
    choice.hardware,
    choice.maxWidth,
    choice.maxHeight,
  );
  return streams;
}

/** A fetched bundle whose Basis textures wait for the renderer's formats. */
export interface Fetched {
  readonly files: number;
  readonly failed: number;
  readonly manifest: string | null;
  /** Texture blobs by asset id. */
  readonly textures: ReadonlyArray<{ readonly id: string; readonly bytes: Uint8Array }>;
  /** Outputs of streaming groups, left for their packs (fetched when a group loads). */
  readonly streamed: number;
}

/**
 * Fetch the bundle under `root` into the wasm: the manifest and every blob
 * of the first group (everything no streaming group holds, the proxies
 * included). Its textures are transcoded by `transcodeBundle` once the
 * renderer is attached, to the encoding the renderer takes. A streaming
 * group's outputs are not fetched: the engine reads its pack
 * (`cooked/stream/<name>.ogb`) through the fetcher when the group loads;
 * nor are the chunks of a sound cooked in chunks, which the engine reads
 * through the fetcher ahead of the playheads, nor a streamed video's units
 * (og.VideoStream), which og.load.video asks the fetcher for, nor a
 * font pack's faces, which the locale's chain asks the fetcher for, nor a
 * streamed texture's whole chain (`<id>#mips`, `texture { stream true }`):
 * the bundle carries its mip tail as `<id>`, and the engine reads `#mips`
 * through the fetcher when the texture is big on screen. The engine asks for at most
 * four such files at a time (og-runtime's `stream::fetch`): the chunks a
 * playhead needs soonest first, then the packs by their group's priority.
 */
export async function preloadBundle(
  glue: Pick<WebExtras, "og_web_put_file" | "og_web_fail_file" | "og_web_bundle_files">,
  root: string,
  progress: (value: number, stage: string) => void,
  received?: Received,
): Promise<Fetched> {
  const dir = cookedDir(root);
  const manifestPath = `${dir}/.cook.ogt`;
  let manifest: string;
  try {
    manifest = new TextDecoder().decode(await bytesOf(manifestPath));
  } catch (e) {
    tellLoadFailure(manifestPath, e);
    glue.og_web_fail_file(manifestPath, `cannot fetch ${manifestPath} (${e instanceof Error ? e.message : String(e)}); run og cook and serve cooked/ next to the page`);
    return { files: 0, failed: 1, manifest: null, textures: [], streamed: 0 };
  }
  glue.og_web_put_file(manifestPath, new TextEncoder().encode(manifest));
  const all = JSON.parse(glue.og_web_bundle_files(manifest)) as Array<{ id: string; file: string; type: string; group?: string }> | { error: string };
  if (!Array.isArray(all)) throw new Error(`${manifestPath}: ${all.error}`);
  // A long sound's chunks (og.SoundChunk) are fetched as its playheads near
  // them, a streamed texture's whole chain (`#mips`) when it is big on screen.
  // A font pack's faces (`<id>.face.ogb`, docs/ui.md §3.4) when the
  // locale's chain or a frame's text first needs them.
  const listed = all.filter((o) => o.group === undefined && o.type !== "og.SoundChunk" && o.type !== "og.VideoStream" && !o.id.endsWith("#mips") && !o.file.endsWith(".face.ogb"));
  const files = [...new Set(listed.map((o) => o.file))];
  let fetched = 0;
  let failed = 0;
  const blobs = new Map<string, Uint8Array>();
  // At most FETCH_CONCURRENCY in flight: a bundle of thousands of files
  // fetched all at once fails some with "Failed to fetch" (Chrome runs out
  // of sockets and buffers).
  let next = 0;
  const one = async (file: string) => {
    const path = `${dir}/${file}`;
    try {
      const bytes = await bytesOf(path, received, received ? () => progress(received.value, "fetch") : undefined);
      glue.og_web_put_file(path, bytes);
      blobs.set(file, bytes);
    } catch (e) {
      failed++;
      tellLoadFailure(path, e);
      glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`);
    }
    ++fetched;
    if (!received) progress(0.1 + 0.7 * (fetched / Math.max(1, files.length)), "fetch");
  };
  await Promise.all(
    Array.from({ length: Math.min(FETCH_CONCURRENCY, files.length) }, async () => {
      while (next < files.length) await one(files[next++]!);
    }),
  );
  const textures = listed.filter((o) => o.type === "og.Texture" && blobs.has(o.file)).map((o) => ({ id: o.id, bytes: blobs.get(o.file)! }));
  return { files: files.length - failed, failed, manifest, textures, streamed: all.length - listed.length };
}

/**
 * Transcode a fetched bundle's Basis textures for `engine`'s renderer
 * (`null`: none attached, RGBA8): each to the encoding the wasm names for
 * its kind on this device (`og_web_texture_target`: BC7 and BC5 on desktop
 * GPUs, ASTC 4x4 on Apple and most Android, ETC2 and EAC RG11 elsewhere,
 * RGBA8 last), every mip level handed over; RGBA8 hands level 0 and the GPU
 * builds the mips.
 */
export async function transcodeBundle(
  glue: WebExtras,
  engine: number | null,
  fetched: Fetched,
  basis: { js: string; wasm: string } | undefined,
  progress: (value: number, stage: string) => void,
  received?: Received,
): Promise<Preloaded> {
  void received;
  const used: string[] = [];
  let transcoder: BasisTranscoder | null = null;
  let transcoded = 0;
  const total = fetched.textures.length;
  const put = (id: string, out: { format: string; width: number; height: number; srgb: boolean; levels: Uint8Array }) => {
    if (out.format === "rgba8") {
      glue.og_web_put_texture(id, out.width, out.height, out.levels, out.srgb);
    } else {
      const refused = glue.og_web_put_texture_encoded(id, out.format, out.width, out.height, out.srgb, out.levels);
      if (refused) throw new Error(refused);
    }
    if (!used.includes(out.format)) used.push(out.format);
    transcoded++;
    // Fork s61: by texture, as a fraction of the stage (the Worker places it on the bar).
    if (transcoded % 8 === 0 || transcoded === total) progress(transcoded / Math.max(1, total), "transcode");
  };
  // The wasm keeps the bundle it parsed last (og-host-web `bundle_of`, s58): the manifest's text
  // crosses once, and "" names it in every later call.
  let manifest = fetched.manifest;
  const jobs: BasisJob[] = [];
  for (const t of fetched.textures) {
    const ktx2 = manifest === null ? new Uint8Array() : glue.og_web_texture_ktx2(manifest, t.id, t.bytes);
    if (manifest !== null) manifest = "";
    if (ktx2.length > 0) jobs.push({ id: t.id, ktx2, target: engine === null ? "rgba8" : glue.og_web_texture_target(engine, "", t.id, t.bytes) });
  }
  // Fork s62: a bundle of many textures is transcoded by a pool of Workers (copies of this one)
  // in parallel; a pool that does not start leaves the rest to this Worker.
  const left = jobs.length >= POOL_MIN_JOBS ? await transcodeInPool(jobs, basis, put) : jobs;
  for (const job of left) {
    transcoder ??= basis ? await loadBasisTranscoder(basis.js, basis.wasm) : await loadBasisTranscoder();
    const out = transcodeKtx2(transcoder, job.ktx2, isTextureFormat(job.target) ? job.target : "rgba8");
    put(job.id, { ...out, levels: out.format === "rgba8" ? out.levels[0]! : concat(out.levels) });
  }
  return { files: fetched.files, failed: fetched.failed, textures: transcoded, textureFormat: used.length ? used.join("+") : "rgba8" };
}

interface BasisJob {
  readonly id: string;
  readonly ktx2: Uint8Array;
  readonly target: string;
}
type BasisAnswer = { type: "og-basis"; id: number; error?: string; format: string; width: number; height: number; srgb: boolean; levels: Uint8Array };

/** Fork s62: below this many Basis textures the boot transcodes them itself. */
const POOL_MIN_JOBS = 24;

/** Fork s62: the transcoding Workers: up to 4, leaving two cores to the page and this Worker. */
function poolSize(): number {
  const cores = (globalThis as { navigator?: { hardwareConcurrency?: number } }).navigator?.hardwareConcurrency ?? 1;
  return Math.max(0, Math.min(4, cores - 2));
}

/**
 * Fork s62: transcode `jobs` on a pool of Workers running this module (its `og-basis` message,
 * `basisJob`), each answer handed to `put` as it lands. Answers the jobs no Worker did (all of
 * them when the pool cannot start), for this Worker to do.
 */
async function transcodeInPool(
  jobs: readonly BasisJob[],
  basis: { js: string; wasm: string } | undefined,
  put: (id: string, out: { format: string; width: number; height: number; srgb: boolean; levels: Uint8Array }) => void,
): Promise<BasisJob[]> {
  const n = poolSize();
  const url = (globalThis as { location?: { href: string } }).location?.href;
  if (n < 2 || !url || typeof Worker !== "function") return [...jobs];
  const pool: Worker[] = [];
  const left: BasisJob[] = [];
  let next = 0;
  let failure: unknown = null;
  try {
    await Promise.all(
      Array.from({ length: n }, async () => {
        const w = new Worker(url, { type: "module" });
        pool.push(w);
        // Each Worker takes the next job when it answered the last; one that fails gives its job back.
        for (;;) {
          if (failure !== null || next >= jobs.length) return;
          const k = next++;
          const job = jobs[k]!;
          const answer = await new Promise<BasisAnswer | null>((resolve) => {
            w.onmessage = (e: MessageEvent) => resolve(e.data as BasisAnswer);
            w.onerror = () => resolve(null);
            const ktx2 = job.ktx2.slice();
            w.postMessage({ type: "og-basis", id: k, ktx2, target: job.target, basis }, [ktx2.buffer]);
          });
          if (!answer || answer.error) {
            left.push(job);
            if (!answer) return; // this Worker did not load: the others go on
            continue;
          }
          try {
            put(job.id, answer);
          } catch (e) {
            failure = e;
          }
        }
      }),
    );
  } finally {
    for (const w of pool) w.terminate();
  }
  if (failure !== null) throw failure;
  // The jobs no Worker reached (every one failed to load) are this Worker's.
  return [...left, ...jobs.slice(next)];
}

/** Fork s62: this Worker as one of the pool: a Basis transcode, answered with its levels (transferred). */
let poolTranscoder: Promise<BasisTranscoder> | null = null;
export async function basisJob(data: { id: number; ktx2: Uint8Array; target: string; basis?: { js: string; wasm: string } }): Promise<void> {
  try {
    poolTranscoder ??= data.basis ? loadBasisTranscoder(data.basis.js, data.basis.wasm) : loadBasisTranscoder();
    const out = transcodeKtx2(await poolTranscoder, data.ktx2, isTextureFormat(data.target) ? data.target : "rgba8");
    const levels = out.format === "rgba8" ? out.levels[0]! : concat(out.levels);
    (self as unknown as { postMessage(m: unknown, transfer: Transferable[]): void }).postMessage({ type: "og-basis", id: data.id, format: out.format, width: out.width, height: out.height, srgb: out.srgb, levels }, [levels.buffer as ArrayBuffer]);
  } catch (e) {
    self.postMessage({ type: "og-basis", id: data.id, error: e instanceof Error ? e.message : String(e) });
  }
}

function isTextureFormat(name: string): name is TextureFormat {
  return (TEXTURE_FORMATS as readonly string[]).includes(name);
}

function concat(parts: readonly Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

/**
 * What the engine's fetcher does with a file it asks for after the boot: a
 * streaming group's pack has its Basis textures transcoded, to the encoding
 * `engine()`'s renderer takes for each (RGBA8 without one), and handed over
 * first, so that every texture in it is ready when the engine reads the
 * pack; any other file is handed over as fetched.
 */
export function installFetcher(glue: WebExtras, manifest: string, basis: { js: string; wasm: string } | undefined, engine: () => number | null): void {
  let transcoder: Promise<BasisTranscoder> | null = null;
  const loaded = (): Promise<BasisTranscoder> => {
    let loading = transcoder;
    if (!loading) {
      const started = basis ? loadBasisTranscoder(basis.js, basis.wasm) : loadBasisTranscoder();
      transcoder = loading = started;
      // A load that failed is not kept: the next pack tries again.
      started.catch(() => {
        if (transcoder === started) transcoder = null;
      });
    }
    return loading;
  };
  const transcode = async (id: string, ktx2: Uint8Array, target: string) => {
    const out = transcodeKtx2(await loaded(), ktx2, isTextureFormat(target) ? target : "rgba8");
    if (out.format === "rgba8") {
      glue.og_web_put_texture(id, out.width, out.height, out.levels[0]!, out.srgb);
    } else {
      const refused = glue.og_web_put_texture_encoded(id, out.format, out.width, out.height, out.srgb, concat(out.levels));
      if (refused) throw new Error(refused);
    }
  };
  const put = async (path: string, bytes: Uint8Array) => {
    const triples = /\/stream\/[^/]+\.ogb$/.test(path) && glue.og_web_pack_ktx2 ? glue.og_web_pack_ktx2(engine() ?? undefined, manifest, bytes) : [];
    for (let i = 0; i + 2 < triples.length; i += 3) {
      await transcode(triples[i] as string, triples[i + 1] as Uint8Array, triples[i + 2] as string);
    }
    glue.og_web_put_file(path, bytes);
  };
  // A device made again after a loss may take other encodings (og-host-web's
  // texture hook): the engine hands back each texture transcoded for the old one.
  glue.og_web_set_transcoder?.((id, ktx2, target) => {
    transcode(id, ktx2, target).catch((e: unknown) => glue.og_web_fail_texture?.(id, e instanceof Error ? e.message : String(e)));
  });
  glue.og_web_set_fetcher((path: string) => {
    bytesOf(path)
      .then((bytes) => put(path, bytes))
      .catch((e: unknown) => {
        tellLoadFailure(path, e);
        glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`);
      });
  });
}

/**
 * Gives the engine the browser's DecompressionStream for zlib bytes (terrain
 * tiles): it inflates natively off the engine's work and answers by key; an
 * engine without it inflates in wasm within a frame budget.
 */
export function installInflater(glue: Pick<WebExtras, "og_web_set_inflater" | "og_web_put_inflated" | "og_web_fail_inflated">): boolean {
  const Stream = (globalThis as { DecompressionStream?: new (format: string) => TransformStream<Uint8Array, Uint8Array> }).DecompressionStream;
  const { og_web_set_inflater: set, og_web_put_inflated: put, og_web_fail_inflated: fail } = glue;
  if (!Stream || !set || !put || !fail) return false;
  set((key: number, bytes: Uint8Array) => {
    new Response(new Blob([new Uint8Array(bytes)]).stream().pipeThrough(new Stream("deflate")))
      .arrayBuffer()
      .then(
        (b) => put(key, new Uint8Array(b)),
        (e: unknown) => fail(key, e instanceof Error ? e.message : String(e)),
      );
  });
  return true;
}

/**
 * Gives the engine the browser's image decoders for og.load.imageData: a
 * PNG or JPEG becomes an ImageBitmap off the engine's work
 * (`createImageBitmap`, no colour conversion, alpha not premultiplied) and
 * its RGBA8 texels are read back through an OffscreenCanvas, answered by
 * key. Without it the engine has no decoder and such a load fails.
 */
export function installImageDecoder(glue: Pick<WebExtras, "og_web_set_image_decoder" | "og_web_put_image" | "og_web_fail_image">): boolean {
  const g = globalThis as { createImageBitmap?: (b: Blob, o?: Record<string, string>) => Promise<ImageBitmap>; OffscreenCanvas?: new (w: number, h: number) => OffscreenCanvas };
  const { og_web_set_image_decoder: set, og_web_put_image: put, og_web_fail_image: fail } = glue;
  if (!g.createImageBitmap || !g.OffscreenCanvas || !set || !put || !fail) return false;
  const decode = g.createImageBitmap;
  const Canvas = g.OffscreenCanvas;
  set((key: number, bytes: Uint8Array) => {
    decode(new Blob([new Uint8Array(bytes)]), { premultiplyAlpha: "none", colorSpaceConversion: "none" })
      .then((bitmap) => {
        const { width, height } = bitmap;
        const ctx = new Canvas(width, height).getContext("2d", { willReadFrequently: true }) as OffscreenCanvasRenderingContext2D | null;
        if (!ctx) throw new Error("no 2D context to read the image's texels");
        ctx.drawImage(bitmap, 0, 0);
        bitmap.close();
        put(key, width, height, new Uint8Array(ctx.getImageData(0, 0, width, height).data.buffer));
      })
      .catch((e: unknown) => fail(key, `the browser could not decode it (${e instanceof Error ? e.message : String(e)})`));
  });
  return true;
}
