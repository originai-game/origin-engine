// The generators' shared pieces: a seeded random stream, value noise, and
// writers for PNG (RGBA8 or 16-bit grey), Radiance HDR and 16-bit WAV.
// Every function is a pure function of its arguments.

import { deflateSync } from "node:zlib";

/** mulberry32: a small seeded generator, the same numbers on every machine. */
export function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function hash2(x: number, y: number, s: number): number {
  let h = Math.imul(x | 0, 374761393) ^ Math.imul(y | 0, 668265263) ^ Math.imul(s | 0, 2147483647);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

export const clamp = (v: number, a: number, b: number) => Math.max(a, Math.min(b, v));
export const mix = (a: number, b: number, t: number) => a + (b - a) * t;
const smooth = (t: number) => t * t * (3 - 2 * t);
export function smoothstep(a: number, b: number, v: number): number {
  return smooth(clamp((v - a) / (b - a), 0, 1));
}

/** Value noise, wrapping every `period` cells (0 for none). */
export function vnoise(x: number, y: number, s: number, period = 0): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const fx = smooth(x - xi);
  const fy = smooth(y - yi);
  const w = (v: number) => (period > 0 ? ((v % period) + period) % period : v);
  const a = hash2(w(xi), w(yi), s);
  const b = hash2(w(xi + 1), w(yi), s);
  const c = hash2(w(xi), w(yi + 1), s);
  const d = hash2(w(xi + 1), w(yi + 1), s);
  return mix(mix(a, b, fx), mix(c, d, fx), fy);
}

export function fbm(x: number, y: number, s: number, octaves = 4, period = 0): number {
  let sum = 0;
  let amp = 0.5;
  let norm = 0;
  for (let o = 0; o < octaves; o++) {
    const f = 1 << o;
    sum += amp * vnoise(x * f, y * f, s + o * 17, period > 0 ? period * f : 0);
    norm += amp;
    amp *= 0.5;
  }
  return sum / norm;
}

export function hex(h: string): [number, number, number] {
  const s = h.replace("#", "");
  return [parseInt(s.slice(0, 2), 16), parseInt(s.slice(2, 4), 16), parseInt(s.slice(4, 6), 16)];
}

/** sRGB 0..255 to linear 0..1. */
export function linear(c: number): number {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

// ---------------------------------------------------------------------- PNG

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

function concat(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

/** A PNG: RGBA8 (`channels` 4) or 16-bit grey (`channels` 1, values 0..65535). */
export function png(w: number, h: number, px: ArrayLike<number>, channels: 4 | 1 = 4): Uint8Array {
  const bpp = channels === 4 ? 4 : 2;
  const raw = new Uint8Array(h * (w * bpp + 1));
  for (let y = 0; y < h; y++) {
    const row = y * (w * bpp + 1);
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
  return concat([Uint8Array.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array())]);
}

/** A texture drawn per pixel: `colour(u, v)` answers RGBA 0..255 for u, v in 0..1. */
export function draw(size: number, colour: (u: number, v: number, x: number, y: number) => [number, number, number, number]): Uint8Array {
  const px = new Float32Array(size * size * 4);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const c = colour((x + 0.5) / size, (y + 0.5) / size, x, y);
      px.set(c, (y * size + x) * 4);
    }
  }
  return png(size, size, px, 4);
}

// -------------------------------------------------------------------- HDR

function rgbe(r: number, g: number, b: number): [number, number, number, number] {
  const m = Math.max(r, g, b);
  if (m < 1e-32) return [0, 0, 0, 0];
  const e = Math.ceil(Math.log2(m));
  const f = 256 / 2 ** e;
  return [Math.min(255, Math.floor(r * f)), Math.min(255, Math.floor(g * f)), Math.min(255, Math.floor(b * f)), e + 128];
}

/** A Radiance .hdr (flat RGBE scanlines). */
export function hdr(w: number, h: number, px: (u: number, v: number) => [number, number, number]): Uint8Array {
  const head = new TextEncoder().encode(`#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y ${h} +X ${w}\n`);
  const body = new Uint8Array(w * h * 4);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) body.set(rgbe(...px((x + 0.5) / w, (y + 0.5) / h)), (y * w + x) * 4);
  }
  return concat([head, body]);
}

// -------------------------------------------------------------------- WAV

/** 16-bit PCM WAV of interleaved float samples in -1..1. */
export function wav(samples: Float32Array, rate: number, channels: 1 | 2): Uint8Array {
  const pcm = new Int16Array(samples.length);
  for (let i = 0; i < samples.length; i++) pcm[i] = Math.round(clamp(samples[i]!, -1, 1) * 32767);
  const out = new Uint8Array(44 + pcm.byteLength);
  const dv = new DataView(out.buffer);
  const tag = (at: number, s: string) => {
    for (let i = 0; i < 4; i++) out[at + i] = s.charCodeAt(i);
  };
  tag(0, "RIFF");
  dv.setUint32(4, 36 + pcm.byteLength, true);
  tag(8, "WAVE");
  tag(12, "fmt ");
  dv.setUint32(16, 16, true);
  dv.setUint16(20, 1, true);
  dv.setUint16(22, channels, true);
  dv.setUint32(24, rate, true);
  dv.setUint32(28, rate * 2 * channels, true);
  dv.setUint16(32, 2 * channels, true);
  dv.setUint16(34, 16, true);
  tag(36, "data");
  dv.setUint32(40, pcm.byteLength, true);
  out.set(new Uint8Array(pcm.buffer), 44);
  return out;
}
