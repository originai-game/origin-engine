// PNG frames the probe's capture answers, decoded for pixel assertions (the
// same decoder templates/ui-kit/tests/frames.ts uses for its references).
import { inflateSync } from "node:zlib";

export interface Image {
  width: number;
  height: number;
  /** RGBA8 rows. */
  rgba: Uint8Array;
}

/** An 8-bit RGB or RGBA, non-interlaced PNG, unfiltered. */
export function decodePng(bytes: Uint8Array): Image {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let at = 8;
  let width = 0;
  let height = 0;
  let channels = 4;
  const idat: Uint8Array[] = [];
  while (at < bytes.length) {
    const len = view.getUint32(at);
    const type = String.fromCharCode(...bytes.subarray(at + 4, at + 8));
    const data = bytes.subarray(at + 8, at + 8 + len);
    if (type === "IHDR") {
      width = view.getUint32(at + 8);
      height = view.getUint32(at + 12);
      const depth = data[8]!;
      const color = data[9]!;
      if (depth !== 8 || (color !== 6 && color !== 2) || data[12] !== 0) throw new Error(`unsupported PNG (depth ${depth}, colour ${color}, interlace ${data[12]})`);
      channels = color === 6 ? 4 : 3;
    } else if (type === "IDAT") idat.push(data);
    else if (type === "IEND") break;
    at += 12 + len;
  }
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const out = new Uint8Array(width * height * 4);
  const prev = new Uint8Array(stride);
  const cur = new Uint8Array(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]!;
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let i = 0; i < stride; i++) {
      const a = i >= channels ? cur[i - channels]! : 0;
      const b = prev[i]!;
      const c = i >= channels ? prev[i - channels]! : 0;
      let v = line[i]!;
      if (filter === 1) v += a;
      else if (filter === 2) v += b;
      else if (filter === 3) v += (a + b) >> 1;
      else if (filter === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      cur[i] = v & 255;
    }
    for (let x = 0; x < width; x++) {
      for (let k = 0; k < 4; k++) out[(y * width + x) * 4 + k] = k < channels ? cur[x * channels + k]! : 255;
    }
    prev.set(cur);
  }
  return { width, height, rgba: out };
}

/** The RGB of one pixel. */
export function pixel(img: Image, x: number, y: number): [number, number, number] {
  const i = (Math.round(y) * img.width + Math.round(x)) * 4;
  return [img.rgba[i]!, img.rgba[i + 1]!, img.rgba[i + 2]!];
}

/** A capture's PNG (the probe answers base64 PNG bytes) decoded. */
export function decodeCapture(frame: unknown): Image {
  const f = frame as { png?: string } | null;
  if (!f || typeof f.png !== "string") throw new Error(`no frame: ${JSON.stringify(frame)?.slice(0, 200)}`);
  return decodePng(new Uint8Array(Buffer.from(f.png, "base64")));
}
