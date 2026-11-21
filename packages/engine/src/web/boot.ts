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
  /** One file's: its total is the response's length when the response names it (not when it is encoded). */
  readonly fromResponse: boolean;
  constructor(total = 0, fromResponse = false) {
    this.total = total;
    this.fromResponse = fromResponse;
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
 * bytes are in: the download (wasm, bundle, fetch) 0-50 %, the Basis transcode 50-60 %, the first
 * tick 60-65 %, the first frame (its pipelines, counted ready of requested) 65-100 %: on a cold
 * driver a large scene's compile is most of the wait.
 */
export const BOOT_SPANS: Readonly<Record<string, readonly [number, number]>> = {
  wasm: [0, 0.5],
  bundle: [0, 0.5],
  fetch: [0, 0.5],
  transcode: [0.5, 0.6],
  ready: [0.6, 0.6],
  prime: [0.6, 0.65],
  present: [0.65, 1],
};

/** A stage's fraction (0 to 1) as a place on the whole bar. */
export function onBar(stage: string, value: number): number {
  const [a, b] = BOOT_SPANS[stage] ?? [0, 1];
  return a + (b - a) * Math.max(0, Math.min(1, value));
}

async function bytesOf(url: string, received?: Received, onChunk?: () => void, integrity?: string): Promise<Uint8Array> {
  // Fork s61: a fetch that failed (not refused: HTTP 4xx) is tried again, up to 5 times, backing off;
  // its bytes already counted come off the progress first.
  for (let attempt = 0; ; attempt++) {
    const before = received ? received.loaded : 0;
    try {
      return await bytesOnce(url, received, onChunk, integrity);
    } catch (e) {
      if (received) received.loaded = before;
      // An integrity failure is the bytes, not the network: the same on every try.
      if (attempt >= 5 || /HTTP 4\d\d|integrity/.test(String(e))) throw e;
      console.warn(`fetch ${url} failed (${e instanceof Error ? e.message : String(e)}), retry ${attempt + 1}`);
      await new Promise((ok) => setTimeout(ok, 500 * 2 ** attempt));
    }
  }
}

/** `bytes` when their SHA-384 is `sri`'s (`sha384-<base64>`); otherwise an `integrity` error naming both. */
async function checkIntegrity(bytes: Uint8Array, sri: string): Promise<Uint8Array> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-384", bytes as Uint8Array<ArrayBuffer>));
  let bin = "";
  for (const b of digest) bin += String.fromCharCode(b);
  const got = `sha384-${btoa(bin)}`;
  if (got !== sri) throw new Error(`integrity: the bytes are ${got}, expected ${sri}`);
  return bytes;
}

async function bytesOnce(url: string, received?: Received, onChunk?: () => void, integrity?: string): Promise<Uint8Array> {
  // A cooked file's name may hold a `#` (`m.gltf.tex0#mips.ogb`, a flipbook's
  // `#array#layer<n>`): sent as is, the browser takes the rest as a fragment.
  // With `integrity` (a package from an asset base) the browser refuses
  // bytes whose hash differs: a TypeError, named here.
  const init: RequestInit = {
    ...(typeof AbortSignal?.timeout === "function" ? { signal: AbortSignal.timeout(FETCH_DEADLINE_MS) } : {}),
    ...(integrity ? { integrity } : {}),
  };
  let r: Response;
  try {
    r = await fetch(url.replaceAll("#", "%23"), init);
  } catch (e) {
    if (integrity && e instanceof TypeError) throw new Error(`refused: the network, CORS or its integrity (${integrity.slice(0, 15)}…) failed (${e.message})`);
    throw e;
  }
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  if (received?.fromResponse && !r.headers.get("content-encoding")) {
    const length = Number(r.headers.get("content-length"));
    if (length > 0) received.total = length;
  }
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
 * when WebCodecs cannot decode AV1, by its own Worker (`worker`, its script
 * av1/av1-worker.js beside a built page), when a stream first plays.
 */
export async function installVideo(glue: WebExtras, av1: string | undefined, worker?: string): Promise<VideoStreams | null> {
  if (!glue.og_web_install_video) return null;
  const choice = await chooseDecoder();
  const streams = new VideoStreams(choice, av1 ?? FALLBACK_URL, worker ?? null);
  glue.og_web_install_video(
    (key, width, height, codec, config, table, chunkFrames) => streams.stream(key, width, height, 0, table, chunkFrames, codec, config),
    (stream, player, frame, looped) => streams.want(player, stream, frame, looped),
    (stream, n, data) => streams.chunk(stream, n, data),
    choice.name,
    choice.hardware,
    choice.maxWidth,
    choice.maxHeight,
    [...choice.codecs],
  );
  return streams;
}

/** A fetched bundle whose Basis textures wait for the renderer's formats. */
export interface Fetched {
  readonly files: number;
  readonly failed: number;
  /** Outputs of streaming groups, left for their packs (fetched when a group loads). */
  readonly streamed: number;
  /**
   * Each cooked file's integrity, by its path under `cooked/`, when the
   * package came from an asset base (`manifest` integrity given): the
   * fetcher fetches every later file against it. Null otherwise.
   */
  readonly integrity: Readonly<Record<string, string>> | null;
}

/**
 * Fetch the bundle under `root` into the wasm: the manifest and every blob
 * of the first group (everything no streaming group holds, the proxies
 * included). Its textures are prepared by `prepareTextures` once the
 * renderer is attached, as the engine asks for them. A streaming
 * group's outputs are not fetched: the engine reads its pack
 * (`cooked/stream/<name>.ogb`) through the fetcher when the group loads;
 * nor are the chunks of a sound cooked in chunks, which the engine reads
 * through the fetcher ahead of the playheads, nor a streamed video's index
 * (og.VideoStream) and chunks (og.VideoChunk), which its players ask the
 * fetcher for ahead of their playheads, nor a
 * font pack's faces, which the locale's chain asks the fetcher for, nor a
 * streamed texture's whole chain (`<id>#mips`, `texture { stream true }`):
 * the bundle carries its mip tail as `<id>`, and the engine reads `#mips`
 * through the fetcher when the texture is big on screen. The engine asks for at most
 * four such files at a time (og-runtime's `stream::fetch`): the chunks a
 * playhead needs soonest first, then the packs by their group's priority.
 */
export async function preloadBundle(
  glue: Pick<WebExtras, "og_web_put_file" | "og_web_fail_file" | "og_web_bundle_files" | "og_web_bundle_integrity">,
  root: string,
  progress: (value: number, stage: string) => void,
  received?: Received,
  manifestIntegrity?: string,
): Promise<Fetched> {
  const dir = cookedDir(root);
  const manifestPath = `${dir}/.cook.ogt`;
  let manifest: string;
  try {
    manifest = new TextDecoder().decode(await bytesOf(manifestPath, undefined, undefined, manifestIntegrity));
  } catch (e) {
    tellLoadFailure(manifestPath, e);
    const hint = manifestIntegrity ? "the asset base must serve this build's cooked/ (og-build.json `assets`)" : "run og cook and serve cooked/ next to the page";
    glue.og_web_fail_file(manifestPath, `cannot fetch ${manifestPath} (${e instanceof Error ? e.message : String(e)}); ${hint}`);
    return { files: 0, failed: 1, streamed: 0, integrity: null };
  }
  glue.og_web_put_file(manifestPath, new TextEncoder().encode(manifest));
  // A package from an asset base: every file against the manifest's integrity.
  let integrity: Record<string, string> | null = null;
  if (manifestIntegrity) {
    const listed = JSON.parse(glue.og_web_bundle_integrity(manifest)) as Record<string, string>;
    if (typeof listed["error"] === "string") throw new Error(`${manifestPath}: ${listed["error"]}`);
    integrity = listed;
  }
  const all = JSON.parse(glue.og_web_bundle_files(manifest)) as Array<{ id: string; file: string; type: string; group?: string }> | { error: string };
  if (!Array.isArray(all)) throw new Error(`${manifestPath}: ${all.error}`);
  // A long sound's chunks (og.SoundChunk) are fetched as its playheads near
  // them, a streamed texture's whole chain (`#mips`) when it is big on screen.
  // A font pack's faces (`<id>.face.ogb`, docs/ui.md §3.4) when the
  // locale's chain or a frame's text first needs them.
  const listed = all.filter((o) => o.group === undefined && o.type !== "og.SoundChunk" && o.type !== "og.VideoStream" && o.type !== "og.VideoChunk" && !o.id.endsWith("#mips") && !o.file.endsWith(".face.ogb"));
  const files = [...new Set(listed.map((o) => o.file))];
  let fetched = 0;
  let failed = 0;
  // At most FETCH_CONCURRENCY in flight: a bundle of thousands of files
  // fetched all at once fails some with "Failed to fetch" (Chrome runs out
  // of sockets and buffers).
  let next = 0;
  const one = async (file: string) => {
    const path = `${dir}/${file}`;
    try {
      const bytes = await bytesOf(path, received, received ? () => progress(received.value, "fetch") : undefined, integrity?.[file]);
      glue.og_web_put_file(path, bytes);
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
  return { files: files.length - failed, failed, streamed: all.length - listed.length, integrity };
}

/**
 * The boot's cache fill, once the renderer is attached: the engine asks for
 * every Basis texture of the game's bundle through its texture hook
 * (og_web_prepare_textures), as a model or og.load.texture asks for one
 * later, and this waits until the transcoders (`installTranscoder`) have
 * answered them all, so the first frame finds them prepared. Each reaches
 * the encoding the engine names for its kind on this device (BC7 and BC5 on
 * desktop GPUs, ASTC 4x4 on Apple and most Android, ETC2 and EAC RG11
 * elsewhere, RGBA8 last or without a renderer).
 */
export async function prepareTextures(glue: Pick<WebExtras, "og_web_prepare_textures">, engine: number, fetched: Fetched, progress: (value: number, stage: string) => void): Promise<Preloaded> {
  for (;;) {
    const s = JSON.parse(glue.og_web_prepare_textures(engine)) as { total: number; pending: number; formats: string };
    progress((s.total - s.pending) / Math.max(1, s.total), "transcode");
    if (s.pending === 0) return { files: fetched.files, failed: fetched.failed, textures: s.total, textureFormat: s.formats || "rgba8" };
    await new Promise((ok) => setTimeout(ok, 4));
  }
}

type BasisAnswer = { type: "og-basis"; id: number; error?: string; format: string; width: number; height: number; srgb: boolean; levels: Uint8Array };

/** The transcoding Workers: up to 4, leaving two cores to the page and this Worker. */
function poolSize(): number {
  const cores = (globalThis as { navigator?: { hardwareConcurrency?: number } }).navigator?.hardwareConcurrency ?? 1;
  return Math.max(0, Math.min(4, cores - 2));
}

/**
 * One job on a transcoding Worker: its answer, or null when the Worker did not load or broke. The
 * Worker is a copy of the engine's, which also posts its console's lines (`log`) and what stops it
 * outside a job (`error`, `crash`): only the answer to this job is one.
 */
function askWorker(w: Worker, id: number, job: { readonly ktx2: Uint8Array; readonly target: string }, basis: { js: string; wasm: string } | undefined): Promise<BasisAnswer | null> {
  return new Promise((resolve) => {
    w.onmessage = (e: MessageEvent) => {
      const m = e.data as BasisAnswer | { type: "error" | "crash" | "log" };
      if (m.type === "og-basis" && m.id === id) resolve(m);
      else if (m.type === "error" || m.type === "crash") resolve(null);
    };
    w.onerror = () => resolve(null);
    const ktx2 = job.ktx2.slice();
    w.postMessage({ type: "og-basis", id, ktx2, target: job.target, basis }, [ktx2.buffer]);
  });
}

/** This Worker as one of the transcoders' pool: a Basis transcode, answered with its levels (transferred). */
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

/**
 * The Workers that transcode what the engine asks for: copies of this one, started when the first
 * Basis texture is asked for (another while every one is busy, up to the pool's size and at least
 * one) and kept, since a streaming game goes on asking. Done on this Worker a texture holds the
 * engine's frames for as long as it takes, and the first of them the transcoder's own start
 * besides. Answers null for a job no Worker did, for this Worker to do: where none starts, and
 * after one broke.
 */
function transcodeHelpers(basis: { js: string; wasm: string } | undefined): (ktx2: Uint8Array, target: string) => Promise<BasisAnswer | null> {
  const url = (globalThis as { location?: { href: string } }).location?.href;
  const most = Math.max(1, poolSize());
  const lanes: { worker: Worker; last: Promise<BasisAnswer | null>; jobs: number }[] = [];
  let off = !url || typeof Worker !== "function";
  let asked = 0;
  return (ktx2, target) => {
    if (off) return Promise.resolve(null);
    let lane = lanes.reduce<(typeof lanes)[number] | undefined>((least, l) => (least && least.jobs <= l.jobs ? least : l), undefined);
    if (!lane || (lane.jobs > 0 && lanes.length < most)) {
      try {
        lane = { worker: new Worker(url!, { type: "module" }), last: Promise.resolve(null), jobs: 0 };
      } catch {
        off = true;
        return Promise.resolve(null);
      }
      lanes.push(lane);
    }
    const mine = lane;
    mine.jobs++;
    mine.last = mine.last.then(async () => {
      const answer = off ? null : await askWorker(mine.worker, asked++, { ktx2, target }, basis);
      mine.jobs--;
      // One that did not load or broke: no Worker is asked again, and each stops once its jobs are answered.
      if (!answer) off = true;
      if (off && (!answer || mine.jobs === 0)) mine.worker.terminate();
      return answer && !answer.error ? answer : null;
    });
    return mine.last;
  };
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
 * The engine's Basis transcoder (og_web_set_transcoder): the one way a Basis
 * texture reaches the engine on the web, whichever bundle or pack it came
 * from, at the boot (`prepareTextures`), as a streaming group loads, for a
 * runtime bundle and after a device loss. The engine's texture hook hands
 * it a job (its key, the KTX2 bytes and the encoding the engine chose for
 * this device) and answers Pending until the job's levels come back. The
 * transcoding is other Workers' (`transcodeHelpers`), so no frame waits on
 * it; this Worker's own transcoder is only for a job none of them did.
 */
export function installTranscoder(glue: Pick<WebExtras, "og_web_set_transcoder" | "og_web_put_texture" | "og_web_put_texture_encoded" | "og_web_fail_texture">, basis: { js: string; wasm: string } | undefined): void {
  let transcoder: Promise<BasisTranscoder> | null = null;
  const loaded = (): Promise<BasisTranscoder> => {
    let loading = transcoder;
    if (!loading) {
      const started = basis ? loadBasisTranscoder(basis.js, basis.wasm) : loadBasisTranscoder();
      transcoder = loading = started;
      // A load that failed is not kept: the next texture tries again.
      started.catch(() => {
        if (transcoder === started) transcoder = null;
      });
    }
    return loading;
  };
  const helped = transcodeHelpers(basis);
  const transcode = async (key: string, ktx2: Uint8Array, target: string) => {
    let out: Pick<BasisAnswer, "format" | "width" | "height" | "srgb" | "levels"> | null = await helped(ktx2, target);
    if (!out) {
      const own = transcodeKtx2(await loaded(), ktx2, isTextureFormat(target) ? target : "rgba8");
      out = { ...own, levels: own.format === "rgba8" ? own.levels[0]! : concat(own.levels) };
    }
    const refused = out.format === "rgba8" ? glue.og_web_put_texture(key, out.width, out.height, out.levels, out.srgb) : glue.og_web_put_texture_encoded(key, out.format, out.width, out.height, out.srgb, out.levels);
    if (refused) throw new Error(refused);
  };
  glue.og_web_set_transcoder((key, ktx2, target) => {
    transcode(key, ktx2, target).catch((e: unknown) => glue.og_web_fail_texture(key, e instanceof Error ? e.message : String(e)));
  });
}

/**
 * What the engine's fetcher does with a file it asks for after the boot: it
 * is handed over as fetched, against its bundle's integrity when it has one.
 */
export function installFetcher(glue: WebExtras, integrity: Readonly<Record<string, string>> | null = null): void {
  const put = (path: string, bytes: Uint8Array) => glue.og_web_put_file(path, bytes);
  // Each bundle og.load.bundle opened, by its directory: its files'
  // integrity, from its manifest as the engine fetches it (before any of
  // its files).
  const opened = new Map<string, Readonly<Record<string, string>>>();
  const sriOf = (path: string): string | undefined => {
    for (let dir = path.slice(0, Math.max(0, path.lastIndexOf("/"))); dir; dir = dir.slice(0, Math.max(0, dir.lastIndexOf("/")))) {
      const listed = opened.get(dir);
      if (listed) return listed[path.slice(dir.length + 1)];
    }
    // A package from an asset base: its file under cooked/ against the manifest's integrity.
    return integrity ? integrity[path.slice(path.lastIndexOf("/cooked/") + "/cooked/".length)] : undefined;
  };
  glue.og_web_set_fetcher((path: string, given?: string) => {
    // Its bytes as they arrive, for og.stream.status's and og.load.progress's `received`.
    const got = glue.og_web_file_progress ? new Received(0, true) : undefined;
    const progress = got ? () => glue.og_web_file_progress!(path, got.loaded, got.total) : undefined;
    // A content catalog's file comes with its integrity (og.load.data), checked here so a
    // network or CORS failure stays one (the browser's `integrity` refuses both alike); a
    // bundle's file is checked by the browser against its bundle's manifest (sriOf).
    const fetched =
      given !== undefined
        ? bytesOf(path, got, progress).then(async (bytes) => (given ? checkIntegrity(bytes, given) : bytes))
        : bytesOf(path, got, progress, sriOf(path));
    fetched
      .then((bytes) => {
        if (path.endsWith("/.cook.ogt")) {
          const listed = JSON.parse(glue.og_web_bundle_integrity(new TextDecoder().decode(bytes))) as Record<string, string>;
          // One that does not parse fails its bundle in the engine.
          if (typeof listed["error"] !== "string") opened.set(path.slice(0, -"/.cook.ogt".length), listed);
        }
        return put(path, bytes);
      })
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
 * Where an upright image's texels go to stand as its file stores them,
 * under EXIF orientation `exif` (1 to 8; anything else is upright): the
 * stored size and the 2D context transform (`setTransform(a, b, c, d, e,
 * f)`) that draws the upright `width` × `height` image there. The inverse
 * of the turn og-draw's UiOrientation names, which createImageBitmap's
 * `from-image` has applied.
 */
export function storedTurn(exif: number, width: number, height: number): { width: number; height: number; matrix: [number, number, number, number, number, number] } {
  const [w, h] = exif >= 5 && exif <= 8 ? [height, width] : [width, height];
  const matrix: Record<number, [number, number, number, number, number, number]> = {
    2: [-1, 0, 0, 1, w, 0],
    3: [-1, 0, 0, -1, w, h],
    4: [1, 0, 0, -1, 0, h],
    5: [0, 1, 1, 0, 0, 0],
    6: [0, -1, 1, 0, 0, h],
    7: [0, -1, -1, 0, w, h],
    8: [0, 1, -1, 0, w, 0],
  };
  return { width: w, height: h, matrix: matrix[exif] ?? [1, 0, 0, 1, 0, 0] };
}

/**
 * An ImageBitmap's RGBA8 texels, straight alpha, read back through an
 * OffscreenCanvas; the bitmap is closed. With `exif`, the bitmap is
 * upright and its texels are turned back as its file stores them.
 */
function texels(bitmap: ImageBitmap, Canvas: new (w: number, h: number) => OffscreenCanvas, exif = 1): { width: number; height: number; rgba: Uint8Array } {
  const { width, height, matrix } = storedTurn(exif, bitmap.width, bitmap.height);
  const ctx = new Canvas(width, height).getContext("2d", { willReadFrequently: true }) as OffscreenCanvasRenderingContext2D | null;
  if (!ctx) throw new Error("no 2D context to read the image's texels");
  ctx.setTransform(...matrix);
  ctx.drawImage(bitmap, 0, 0);
  bitmap.close();
  return { width, height, rgba: new Uint8Array(ctx.getImageData(0, 0, width, height).data.buffer) };
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
        const t = texels(bitmap, Canvas);
        put(key, t.width, t.height, t.rgba, 1);
      })
      .catch((e: unknown) => fail(key, `the browser could not decode it (${e instanceof Error ? e.message : String(e)})`));
  });
  return true;
}

/** Whether fetched bytes are an SVG image: UTF-8 text holding an `<svg` element (og-cook's and the native host's test). */
export function isSvg(bytes: Uint8Array): boolean {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes).includes("<svg");
  } catch {
    return false;
  }
}

/**
 * The SVGs this Worker asked the page to draw (a Worker has no SVG
 * decoder): `ask` posts `{ type: "og-svg", id, bytes }`, and the page's
 * `{ type: "svg", id, bitmap | error }` settles it through `answer`.
 */
export class PageSvg {
  #next = 0;
  readonly #waiting = new Map<number, { ok: (b: ImageBitmap) => void; fail: (e: Error) => void }>();
  readonly #post: (message: unknown, transfer: Transferable[]) => void;
  constructor(post: (message: unknown, transfer: Transferable[]) => void) {
    this.#post = post;
  }
  ask(bytes: Uint8Array): Promise<ImageBitmap> {
    const id = this.#next++;
    return new Promise((ok, fail) => {
      this.#waiting.set(id, { ok, fail });
      const copy = bytes.slice();
      this.#post({ type: "og-svg", id, bytes: copy }, [copy.buffer]);
    });
  }
  answer(m: { id: number; bitmap?: ImageBitmap; error?: string }): void {
    const w = this.#waiting.get(m.id);
    this.#waiting.delete(m.id);
    if (m.bitmap) w?.ok(m.bitmap);
    else w?.fail(new Error(m.error ?? "the page did not draw it"));
  }
}

/** This Worker's SVG requests to its page (worker.ts answers them). */
export const pageSvg = new PageSvg((m, transfer) => (globalThis as unknown as { postMessage(m: unknown, t: Transferable[]): void }).postMessage(m, transfer));

/**
 * Gives the engine the Worker's `fetch()` for the http(s) images documents
 * name in url() (og-runtime's world/remote.rs, docs/ui.md §3.3 Remote
 * images): CORS mode (as a browser reads another origin's response), the
 * url()'s integrity metadata, and no credentials or referrer, as natively
 * (og-net's http.rs). A PNG, JPEG, WebP or GIF is decoded as og.load.imageData's
 * are; an SVG, which a Worker cannot draw, by the page (`svg`), at its own
 * size. Answered by key with og_web_put_image or og_web_fail_image. The
 * browser's HTTP cache keeps what it fetched.
 *
 * The texels are handed over as the file stores them, with the EXIF
 * orientation the engine's own reader finds in the bytes
 * (og_web_image_orientation): createImageBitmap always turns an image
 * upright, so it is drawn turned back, and CSS image-orientation turns it
 * at draw time as on the native host.
 */
export function installRemoteFetcher(glue: Pick<WebExtras, "og_web_set_remote_fetcher" | "og_web_put_image" | "og_web_fail_image" | "og_web_image_orientation" | "og_web_set_remote_text_fetcher" | "og_web_put_text" | "og_web_fail_text">, svg: (bytes: Uint8Array) => Promise<ImageBitmap>): boolean {
  const g = globalThis as { createImageBitmap?: (b: Blob, o?: Record<string, string>) => Promise<ImageBitmap>; OffscreenCanvas?: new (w: number, h: number) => OffscreenCanvas };
  const { og_web_set_remote_fetcher: set, og_web_put_image: put, og_web_fail_image: fail } = glue;
  if (!g.createImageBitmap || !g.OffscreenCanvas || !set || !put || !fail) return false;
  const decode = g.createImageBitmap;
  const Canvas = g.OffscreenCanvas;
  const init = (integrity: string): RequestInit => ({
    mode: "cors",
    credentials: "omit",
    referrerPolicy: "no-referrer",
    ...(integrity ? { integrity } : {}),
    ...(typeof AbortSignal?.timeout === "function" ? { signal: AbortSignal.timeout(FETCH_DEADLINE_MS) } : {}),
  });
  // The browser names no reason for a refused CORS or integrity check: a TypeError.
  const reason = (e: unknown) => (e instanceof TypeError ? `refused or unreachable (${e.message}: the network, CORS or integrity)` : e instanceof Error ? e.message : String(e));
  // A remote <iframe>'s document: its bytes, under the same request rules.
  const { og_web_set_remote_text_fetcher: setText, og_web_put_text: putText, og_web_fail_text: failText } = glue;
  if (setText && putText && failText) {
    setText((key, url, integrity) => {
      fetch(url, init(integrity))
        .then(async (r) => {
          if (!r.ok) throw new Error(`HTTP ${r.status}`);
          putText(key, new Uint8Array(await r.arrayBuffer()));
        })
        .catch((e: unknown) => failText(key, reason(e)));
    });
  }
  set((key, url, integrity) => {
    fetch(url, init(integrity))
      .then(async (r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        const bytes = new Uint8Array(await r.arrayBuffer());
        const vector = isSvg(bytes);
        const exif = vector ? 1 : (glue.og_web_image_orientation?.(bytes) ?? 1);
        const bitmap = vector ? await svg(bytes) : await decode(new Blob([bytes]), { imageOrientation: "from-image", premultiplyAlpha: "none", colorSpaceConversion: "none" });
        const t = texels(bitmap, Canvas, exif);
        put(key, t.width, t.height, t.rgba, exif);
      })
      .catch((e: unknown) => fail(key, reason(e)));
  });
  return true;
}
