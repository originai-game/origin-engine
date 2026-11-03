// PNG encoding for the asset tools (as examples/lantern-vale/tools/make-assets.ts).

import { deflateSync } from "node:zlib";

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const b of bytes) c = CRC[(c ^ b) & 255]! ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type: string, data: Uint8Array): Uint8Array {
  const out = new Uint8Array(12 + data.length);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, data.length);
  for (let i = 0; i < 4; i++) out[4 + i] = type.charCodeAt(i);
  out.set(data, 8);
  dv.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)));
  return out;
}

/** A PNG: RGBA8 (`channels` 4) or 16-bit grey (`channels` 1, values 0..65535). */
export function png(w: number, h: number, px: ArrayLike<number>, channels: 4 | 1): Uint8Array {
  const bpp = channels === 4 ? 4 : 2;
  const raw = new Uint8Array(h * (w * bpp + 1));
  for (let y = 0; y < h; y++) {
    const row = y * (w * bpp + 1);
    raw[row] = 0;
    for (let x = 0; x < w; x++) {
      if (channels === 4) {
        for (let k = 0; k < 4; k++) raw[row + 1 + x * 4 + k] = clamp(Math.round(px[(y * w + x) * 4 + k]!), 0, 255);
      } else {
        const v = clamp(Math.round(px[y * w + x]!), 0, 65535);
        raw[row + 1 + x * 2] = v >> 8;
        raw[row + 2 + x * 2] = v & 255;
      }
    }
  }
  const ihdr = new Uint8Array(13);
  const dv = new DataView(ihdr.buffer);
  dv.setUint32(0, w);
  dv.setUint32(4, h);
  ihdr[8] = channels === 4 ? 8 : 16;
  ihdr[9] = channels === 4 ? 6 : 0;
  const sig = Uint8Array.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const parts = [sig, chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array())];
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

