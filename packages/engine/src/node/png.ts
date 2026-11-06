// PNG frames: the probe's captures decoded (8-bit RGB or RGBA,
// non-interlaced, any filter) and RGBA images encoded (`encodePngBest`
// writes the frames kept in git, every reference).

/** node:zlib, reached when a PNG is coded (bun and node both answer it) rather than imported: `@origingame/engine/playtest` reads this module, and a game type-checks its playtests without node's declarations. */
function zlib(): Zlib {
  return (globalThis as unknown as { process: { getBuiltinModule(id: string): Zlib } }).process.getBuiltinModule("node:zlib");
}

interface Zlib {
  deflateSync(data: Uint8Array, options?: { level?: number }): Uint8Array;
  inflateSync(data: Uint8Array): Uint8Array;
}

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
  const joined = new Uint8Array(idat.reduce((n, d) => n + d.length, 0));
  idat.reduce((at, d) => (joined.set(d, at), at + d.length), 0);
  const raw = zlib().inflateSync(joined);
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

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function chunk(type: string, data: Uint8Array): Uint8Array {
  const out = new Uint8Array(12 + data.length);
  const view = new DataView(out.buffer);
  view.setUint32(0, data.length);
  for (let i = 0; i < 4; i++) out[4 + i] = type.charCodeAt(i);
  out.set(data, 8);
  let c = 0xffffffff;
  for (let i = 4; i < 8 + data.length; i++) c = CRC[(c ^ out[i]!) & 255]! ^ (c >>> 8);
  view.setUint32(8 + data.length, (c ^ 0xffffffff) >>> 0);
  return out;
}

/** An RGB PNG of the image (alpha dropped), rows filtered with Sub. */
export function encodePng(img: Image): Uint8Array {
  const stride = img.width * 3;
  const raw = new Uint8Array((stride + 1) * img.height);
  for (let y = 0; y < img.height; y++) {
    raw[y * (stride + 1)] = 1;
    for (let x = 0; x < img.width; x++) {
      for (let k = 0; k < 3; k++) {
        const v = img.rgba[(y * img.width + x) * 4 + k]!;
        const left = x > 0 ? img.rgba[(y * img.width + x - 1) * 4 + k]! : 0;
        raw[y * (stride + 1) + 1 + x * 3 + k] = (v - left) & 255;
      }
    }
  }
  const ihdr = new Uint8Array(13);
  const v = new DataView(ihdr.buffer);
  v.setUint32(0, img.width);
  v.setUint32(4, img.height);
  ihdr.set([8, 2, 0, 0, 0], 8);
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", zlib().deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array(0))];
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  parts.reduce((at, p) => (out.set(p, at), at + p.length), 0);
  return out;
}

/**
 * The smallest PNG of the image this module can write, for frames kept in git
 * (references): RGB when every pixel is opaque, else RGBA; each row filtered
 * with whichever of PNG's five filters leaves the smallest sum of signed
 * residuals (the adaptive heuristic of libpng and the png crate); deflated
 * at zlib's level 9. The pixels decode exactly. A 1280×720 frame takes about
 * a second, so per-run screenshots keep the probe's fast encoder.
 */
export function encodePngBest(img: Image): Uint8Array {
  const { width, height, rgba } = img;
  let opaque = true;
  for (let i = 3; i < rgba.length; i += 4) if (rgba[i] !== 255) { opaque = false; break; }
  const bpp = opaque ? 3 : 4;
  const stride = width * bpp;
  const raw = new Uint8Array((stride + 1) * height);
  let prev = new Uint8Array(stride);
  let cur = new Uint8Array(stride);
  const trial = [0, 1, 2, 3, 4].map(() => new Uint8Array(stride));
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) for (let k = 0; k < bpp; k++) cur[x * bpp + k] = rgba[(y * width + x) * 4 + k]!;
    let best = 0;
    let bestCost = Infinity;
    for (let f = 0; f < 5; f++) {
      const out = trial[f]!;
      let cost = 0;
      for (let i = 0; i < stride; i++) {
        const a = i >= bpp ? cur[i - bpp]! : 0;
        const b = prev[i]!;
        const c = i >= bpp ? prev[i - bpp]! : 0;
        let pred = 0;
        if (f === 1) pred = a;
        else if (f === 2) pred = b;
        else if (f === 3) pred = (a + b) >> 1;
        else if (f === 4) {
          const p = a + b - c;
          const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
          pred = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
        }
        const v = (cur[i]! - pred) & 255;
        out[i] = v;
        cost += v < 128 ? v : 256 - v;
      }
      if (cost < bestCost) {
        bestCost = cost;
        best = f;
      }
    }
    raw[y * (stride + 1)] = best;
    raw.set(trial[best]!, y * (stride + 1) + 1);
    [prev, cur] = [cur, prev];
  }
  const ihdr = new Uint8Array(13);
  const v = new DataView(ihdr.buffer);
  v.setUint32(0, width);
  v.setUint32(4, height);
  ihdr.set([8, opaque ? 2 : 6, 0, 0, 0], 8);
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", zlib().deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array(0))];
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  parts.reduce((at, p) => (out.set(p, at), at + p.length), 0);
  return out;
}
