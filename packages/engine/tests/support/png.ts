// PNG frames the probe's capture answers, decoded for pixel assertions (the
// decoder is src/node/png.ts).
import { decodePng, type Image } from "../../src/node/png.ts";
import { readFileSync } from "node:fs";

export { decodePng, type Image };

declare function atob(data: string): string;

/** The RGB of one pixel. */
export function pixel(img: Image, x: number, y: number): [number, number, number] {
  const i = (Math.round(y) * img.width + Math.round(x)) * 4;
  return [img.rgba[i]!, img.rgba[i + 1]!, img.rgba[i + 2]!];
}

/** A capture's PNG (the probe answers base64 PNG bytes) decoded. */
export function decodeCapture(frame: unknown): Image {
  const f = frame as { png?: string } | null;
  if (!f || typeof f.png !== "string") throw new Error(`no frame: ${JSON.stringify(frame)?.slice(0, 200)}`);
  return decodePng(Uint8Array.from(atob(f.png), (c) => c.charCodeAt(0)));
}

// The tolerances and the recording tier are og-tests' (crates/og-tests/src/
// compare.rs), parsed from it, so the two cannot drift apart.
const compareSource = readFileSync(
  new URL("../../../../crates/og-tests/src/compare.rs", import.meta.url),
  "utf8",
);

function parse(what: string, pattern: RegExp): string {
  const got = compareSource.match(pattern)?.[1];
  if (got === undefined) throw new Error(`png.ts could not read ${what} out of crates/og-tests/src/compare.rs`);
  return got;
}

function field(block: string, name: string): number {
  const got = block.match(new RegExp(`\\b${name}\\s*:\\s*([0-9.]+)`))?.[1];
  if (got === undefined) throw new Error(`png.ts could not read ${name} out of compare.rs`);
  return Number(got);
}

const defaultBlock = parse("Tolerance::default", /fn default\(\) -> Self \{\s*Tolerance \{([\s\S]*?)\}/);
const crossBlock = parse(
  "Tolerance::CROSS_PLATFORM",
  /pub const CROSS_PLATFORM: Tolerance = Tolerance \{([\s\S]*?)\};/,
);

/** og-tests' rule (compare.rs `Tolerance::default`), parsed from it: a channel off by more than 32/255 is an outlier; fail above 0.5 % outliers or an RGB RMSE above 0.01; no bias limit (1). */
export const TOLERANCE = {
  channel: field(defaultBlock, "channel"),
  outliers: field(defaultBlock, "max_outlier_fraction"),
  rmse: field(defaultBlock, "rmse"),
  bias: field(defaultBlock, "max_bias"),
} as const;

/** Its cross-platform twin (compare.rs `Tolerance::CROSS_PLATFORM`), parsed from it: channel 32/255, at most 1 % outliers, RMSE 0.024 and absolute channel mean bias 0.012. RMSE and bias allow about twice the measured backend spread; outliers have more headroom for edge ties. */
export const CROSS_TOLERANCE = {
  channel: field(crossBlock, "channel"),
  outliers: field(crossBlock, "max_outlier_fraction"),
  rmse: field(crossBlock, "rmse"),
  bias: field(crossBlock, "max_bias"),
} as const;

/** The tier reference frames are recorded on (compare.rs `RECORDING_TIER`), parsed from it: the GPU host's, an NVIDIA L4 on Vulkan. A tier without its own frame is judged against this one. */
export const RECORDING = parse("RECORDING_TIER", /pub const RECORDING_TIER: &str = "([^"]+)"/);

export type Tolerance = { readonly channel: number; readonly outliers: number; readonly rmse: number; readonly bias: number };

/** `tolerance_for` (compare.rs): TOLERANCE against a reference recorded on `tier` (its directory, references/<tier>/), CROSS_TOLERANCE against another tier's. */
export function toleranceFor(tier: string, reference: string): Tolerance {
  return reference.split(/[\\/]/).at(-2) === tier ? TOLERANCE : CROSS_TOLERANCE;
}

/**
 * Two frames under `limit` (og-tests' default; `toleranceFor` picks it by the reference): the outliers' share (0 to 1), the RGB
 * RMSE, the largest mean signed difference of a channel (bias, 0 to 1), whether they match, and a difference image (raw RGBA, differences
 * ×4, outliers red). Frames of different sizes never match.
 */
export function compareFrames(a: Image, b: Image, limit: Tolerance = TOLERANCE): { outliers: number; rmse: number; bias: number; ok: boolean; diff: Uint8Array } {
  if (a.width !== b.width || a.height !== b.height) return { outliers: 1, rmse: 1, bias: 1, ok: false, diff: new Uint8Array(0) };
  const n = a.width * a.height;
  const diff = new Uint8Array(n * 4);
  let outliers = 0;
  let sum = 0;
  const signed = [0, 0, 0];
  for (let p = 0; p < n; p++) {
    let max = 0;
    for (let k = 0; k < 3; k++) {
      const s = a.rgba[p * 4 + k]! - b.rgba[p * 4 + k]!;
      signed[k]! += s;
      const d = Math.abs(s);
      max = Math.max(max, d);
      sum += (d / 255) ** 2;
      diff[p * 4 + k] = Math.min(255, d * 4);
    }
    diff[p * 4 + 3] = 255;
    if (max > limit.channel) {
      outliers++;
      diff.set([255, 0, 0, 255], p * 4);
    }
  }
  const share = outliers / n;
  const rmse = Math.sqrt(sum / (n * 3));
  const bias = Math.max(...signed.map((s) => Math.abs(s) / 255 / n));
  return { outliers: share, rmse, bias, ok: share <= limit.outliers && rmse <= limit.rmse && bias <= limit.bias, diff };
}
