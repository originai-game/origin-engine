// KTX2 / Basis Universal transcoding for the web host (og-cook README §What
// phase 2 owes: the Rust runtime has no Basis transcoder on wasm32). Cooked
// textures are KTX2 files (ETC1S colour, UASTC normal and linear maps, or
// lossless RGBA8 with zlib, which og-cook's runtime decodes itself); the engine
// Worker transcodes the Basis ones here with Binomial's transcoder
// (vendor/basis/, Apache-2.0) into the GPU format the tier supports.
//
//   const basis = await loadBasisTranscoder();
//   const format = pickFormat(compressionsFromWebGpu(adapter.features), "etc1s");
//   const tex = transcodeKtx2(basis, ktx2Bytes, format);   // { width, height, format, levels, ... }
//   device.createTexture({ format: wgpuFormatName(tex.format, srgb), ... })
//
// Works in a (module) Worker and under bun. The transcoder's script is a
// classic Emscripten script, evaluated with the Function constructor as
// three.js's KTX2Loader does, so a page whose CSP forbids 'unsafe-eval' must
// allow it for the Worker or load the transcoder another way.

/** The texture formats this module transcodes to. */
export type TextureFormat = "rgba8" | "bc7" | "astc4x4" | "etc2";

/** GPU block compressions a device may support. */
export type Compression = "bc" | "astc" | "etc2";

/** What a cooked KTX2 holds: ETC1S (colour) or UASTC (normal and linear maps). */
export type BasisKind = "etc1s" | "uastc";

/** One level of the transcoder's KTX2 file. */
interface LevelInfo {
  readonly origWidth: number;
  readonly origHeight: number;
  readonly width: number;
  readonly height: number;
}

/** The part of the transcoder's `KTX2File` this module uses. */
interface Ktx2File {
  isValid(): boolean;
  getWidth(): number;
  getHeight(): number;
  getLevels(): number;
  getFaces(): number;
  getLayers(): number;
  getHasAlpha(): boolean;
  isUASTC(): boolean;
  isETC1S(): boolean;
  getDFDTransferFunc(): number;
  startTranscoding(): boolean;
  getImageLevelInfo(level: number, layer: number, face: number): LevelInfo;
  getImageTranscodedSizeInBytes(level: number, layer: number, face: number, format: number): number;
  transcodeImage(dst: Uint8Array, level: number, layer: number, face: number, format: number, getAlphaForOpaque: number, channel0: number, channel1: number): number;
  close(): void;
  delete(): void;
}

/** The initialised transcoder module. */
export interface BasisTranscoder {
  readonly KTX2File: new (bytes: Uint8Array) => Ktx2File;
  readonly transcoder_texture_format?: Record<string, { value: number }>;
  initializeBasis(): void;
}

type Factory = (module: { wasmBinary?: ArrayBuffer | Uint8Array; locateFile?: (path: string) => string }) => Promise<BasisTranscoder>;

/** basisu `transcoder_texture_format` values, used when the module does not export the enum. */
const TRANSCODER_FORMAT: Record<TextureFormat, { name: string; value: number }> = {
  etc2: { name: "cTFETC2_RGBA", value: 1 },
  bc7: { name: "cTFBC7_RGBA", value: 6 },
  astc4x4: { name: "cTFASTC_4x4_RGBA", value: 10 },
  rgba8: { name: "cTFRGBA32", value: 13 },
};

/** The vendored transcoder's files, next to this module in the package. */
export const VENDORED = {
  js: new URL("../../vendor/basis/basis_transcoder.js", import.meta.url).href,
  wasm: new URL("../../vendor/basis/basis_transcoder.wasm", import.meta.url).href,
} as const;

async function bytesOf(source: string | URL | ArrayBuffer | Uint8Array): Promise<Uint8Array> {
  if (source instanceof Uint8Array) return source;
  if (source instanceof ArrayBuffer) return new Uint8Array(source);
  const url = typeof source === "string" ? new URL(source, import.meta.url) : source;
  if (url.protocol === "file:") {
    // bun and node: fetch does not read file: URLs everywhere.
    const { readFileSync } = await import("node:fs");
    const { fileURLToPath } = await import("node:url");
    return new Uint8Array(readFileSync(fileURLToPath(url)));
  }
  const response = await fetch(url);
  if (!response.ok) throw new Error(`cannot fetch the Basis transcoder ${url.href}: HTTP ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
}

/**
 * Load and initialise the transcoder: `js` is the script (a URL, its text
 * as bytes, or omitted for the vendored copy), `wasm` the module (a URL or
 * its bytes, or omitted for the vendored copy).
 */
export async function loadBasisTranscoder(
  js: string | URL | ArrayBuffer | Uint8Array = VENDORED.js,
  wasm: string | URL | ArrayBuffer | Uint8Array = VENDORED.wasm,
): Promise<BasisTranscoder> {
  const [script, binary] = await Promise.all([bytesOf(js), bytesOf(wasm)]);
  const text = new TextDecoder().decode(script);
  // The script defines `var BASIS` (a factory) and, under CommonJS, assigns
  // module.exports: give it no `module`, so it stays a plain factory. Under
  // bun or node it believes it is in node and asks for `fs` and `path`; it
  // gets inert stand-ins, because the wasm bytes are handed over and it never
  // reads a file.
  const inert = (name: string) => new Proxy({}, { get: (_t, key) => () => { throw new Error(`the Basis transcoder tried ${name}.${String(key)}; it is given its wasm bytes and should not`); } });
  const factory = new Function("module", "exports", "require", "__dirname", "__filename", `${text}\n;return BASIS;`)(undefined, undefined, inert, "", "") as Factory;
  if (typeof factory !== "function") throw new Error("the Basis transcoder script did not define BASIS");
  const basis = await factory({ wasmBinary: binary });
  basis.initializeBasis();
  return basis;
}

/** The block compressions a WebGPU adapter or device supports, from its features. */
export function compressionsFromWebGpu(features: { has(feature: string): boolean }): Set<Compression> {
  const out = new Set<Compression>();
  if (features.has("texture-compression-bc")) out.add("bc");
  if (features.has("texture-compression-astc")) out.add("astc");
  if (features.has("texture-compression-etc2")) out.add("etc2");
  return out;
}

/** The block compressions a WebGL2 context supports, from `getSupportedExtensions()`. */
export function compressionsFromWebGl2(extensions: Iterable<string>): Set<Compression> {
  const have = new Set(extensions);
  const out = new Set<Compression>();
  if (have.has("EXT_texture_compression_bptc")) out.add("bc");
  if (have.has("WEBGL_compressed_texture_astc")) out.add("astc");
  if (have.has("WEBGL_compressed_texture_etc")) out.add("etc2");
  return out;
}

/**
 * The GPU format to transcode to on this device: BC7 where BC is supported
 * (desktop), else ASTC 4x4 (Apple, most Android), else ETC2 (other Android),
 * else uncompressed RGBA8. UASTC normal maps and ETC1S colour maps use the
 * same order: BC7 and ASTC keep UASTC's quality, and every choice keeps
 * ETC1S's.
 */
export function pickFormat(supported: ReadonlySet<Compression> | readonly Compression[], kind: BasisKind = "etc1s"): TextureFormat {
  const has = (c: Compression) => (Array.isArray(supported) ? (supported as readonly Compression[]).includes(c) : (supported as ReadonlySet<Compression>).has(c));
  void kind; // one order serves both kinds today; the parameter keeps call sites honest when it changes
  if (has("bc")) return "bc7";
  if (has("astc")) return "astc4x4";
  if (has("etc2")) return "etc2";
  return "rgba8";
}

/** The WebGPU texture format of a transcoded texture. */
export function wgpuFormatName(format: TextureFormat, srgb: boolean): string {
  const base = { rgba8: "rgba8unorm", bc7: "bc7-rgba-unorm", astc4x4: "astc-4x4-unorm", etc2: "etc2-rgba8unorm" }[format];
  return srgb ? `${base}-srgb` : base;
}

/** A transcoded texture: every level of layer 0, face 0. */
export interface Transcoded {
  /** Level 0's size in pixels. */
  readonly width: number;
  readonly height: number;
  /** What `levels` hold: the requested format, or `rgba8` when a block format cannot hold this size. */
  readonly format: TextureFormat;
  /** Level data, level 0 first: RGBA8 rows top to bottom, or 4x4 blocks in row order. */
  readonly levels: Uint8Array[];
  /** Each level's size in pixels. */
  readonly sizes: ReadonlyArray<{ readonly width: number; readonly height: number }>;
  /** What the file held. */
  readonly kind: BasisKind;
  readonly alpha: boolean;
  /** Whether the file's transfer function is sRGB (colour maps). */
  readonly srgb: boolean;
}

function formatValue(basis: BasisTranscoder, format: TextureFormat): number {
  const f = TRANSCODER_FORMAT[format];
  return basis.transcoder_texture_format?.[f.name]?.value ?? f.value;
}

/** The 12-byte KTX2 identifier: «KTX 20»\r\n\x1a\n. */
const KTX2_MAGIC = [0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a];

/**
 * Transcode a Basis Universal KTX2 file (ETC1S or UASTC) to `format`. A
 * block format needs level 0 to be a multiple of 4 on both axes (WebGPU's
 * rule); otherwise the texture comes back as `rgba8`. Throws with the reason
 * for a file that is not a valid Basis KTX2 (a lossless RGBA8 cook is decoded
 * by og-cook's runtime, not here).
 */
export function transcodeKtx2(basis: BasisTranscoder, bytes: Uint8Array, format: TextureFormat): Transcoded {
  // The transcoder's own isValid() accepts garbage that merely parses.
  if (bytes.length < 80 || !KTX2_MAGIC.every((b, i) => bytes[i] === b)) throw new Error("not a KTX2 file: the identifier «KTX 20» is missing");
  const file = new basis.KTX2File(bytes);
  try {
    if (!file.isValid()) throw new Error("not a KTX2 file the Basis transcoder accepts");
    if (!file.isETC1S() && !file.isUASTC()) throw new Error("not a Basis Universal KTX2 (neither ETC1S nor UASTC): decode it with og-cook's runtime instead");
    const width = file.getWidth();
    const height = file.getHeight();
    const chosen: TextureFormat = format !== "rgba8" && (width % 4 !== 0 || height % 4 !== 0) ? "rgba8" : format;
    if (!file.startTranscoding()) throw new Error("the Basis transcoder could not start transcoding this file");
    const value = formatValue(basis, chosen);
    const levels: Uint8Array[] = [];
    const sizes: Array<{ width: number; height: number }> = [];
    for (let level = 0; level < file.getLevels(); level++) {
      const info = file.getImageLevelInfo(level, 0, 0);
      const size = file.getImageTranscodedSizeInBytes(level, 0, 0, value);
      const dst = new Uint8Array(size);
      if (!file.transcodeImage(dst, level, 0, 0, value, 0, -1, -1)) {
        throw new Error(`the Basis transcoder failed on level ${level} (${info.origWidth}x${info.origHeight}) to ${chosen}`);
      }
      levels.push(dst);
      sizes.push({ width: info.origWidth, height: info.origHeight });
    }
    return {
      width,
      height,
      format: chosen,
      levels,
      sizes,
      kind: file.isUASTC() ? "uastc" : "etc1s",
      alpha: file.getHasAlpha(),
      srgb: file.getDFDTransferFunc() === 2,
    };
  } finally {
    file.close();
    file.delete();
  }
}
