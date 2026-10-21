// PNG frames the probe's capture answers, decoded for pixel assertions (the
// decoder is src/node/png.ts, shared with og polish).
import { decodePng, type Image } from "../../src/node/png.ts";

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

/** og-tests' rule (crates/og-tests/src/compare.rs `Tolerance::default`): a channel off by more than 32/255 is an outlier; fail above 0.5 % outliers or an RGB RMSE above 0.01. */
export const TOLERANCE = { channel: 32, outliers: 0.005, rmse: 0.01 } as const;

/**
 * Two frames under og-tests' rule: the outliers' share (0 to 1), the RGB
 * RMSE, whether they match, and a difference image (raw RGBA, differences
 * ×4, outliers red). Frames of different sizes never match.
 */
export function compareFrames(a: Image, b: Image): { outliers: number; rmse: number; ok: boolean; diff: Uint8Array } {
  if (a.width !== b.width || a.height !== b.height) return { outliers: 1, rmse: 1, ok: false, diff: new Uint8Array(0) };
  const n = a.width * a.height;
  const diff = new Uint8Array(n * 4);
  let outliers = 0;
  let sum = 0;
  for (let p = 0; p < n; p++) {
    let max = 0;
    for (let k = 0; k < 3; k++) {
      const d = Math.abs(a.rgba[p * 4 + k]! - b.rgba[p * 4 + k]!);
      max = Math.max(max, d);
      sum += (d / 255) ** 2;
      diff[p * 4 + k] = Math.min(255, d * 4);
    }
    diff[p * 4 + 3] = 255;
    if (max > TOLERANCE.channel) {
      outliers++;
      diff.set([255, 0, 0, 255], p * 4);
    }
  }
  const share = outliers / n;
  const rmse = Math.sqrt(sum / (n * 3));
  return { outliers: share, rmse, ok: share <= TOLERANCE.outliers && rmse <= TOLERANCE.rmse, diff };
}
