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
