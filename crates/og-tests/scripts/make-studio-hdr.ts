// Synthetic equirectangular studio: warm floor, blue sky, key and rim softboxes.
//   bun crates/og-tests/scripts/make-studio-hdr.ts <out.hdr> [width]
import { writeFileSync } from "node:fs";
const lerp = (a: number[], b: number[], t: number) => a.map((x, i) => x + (b[i]! - x) * t);
const norm = (v: number[]) => {
  const n = Math.sqrt(v.reduce((s, c) => s + c * c, 0));
  return v.map((c) => c / n);
};
const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(Math.max((x - a) / (b - a), 0), 1);
  return t * t * (3 - 2 * t);
};
const KEY = norm([-0.5, 0.55, 0.65]),
  RIM = norm([0.75, 0.25, -0.6]);
function radiance(d: number[]): number[] {
  const sky = lerp([0.95, 0.88, 0.78], [0.3, 0.45, 0.85], Math.sqrt(Math.max(d[1]!, 0)));
  const c = lerp([0.16, 0.13, 0.1], sky, smooth(-0.04, 0.04, d[1]!));
  const dot = (v: number[]) => d.reduce((s, a, i) => s + a * v[i]!, 0),
    cos = (deg: number) => Math.cos((deg * Math.PI) / 180);
  const key = smooth(cos(14), cos(9), dot(KEY)),
    rim = smooth(cos(11), cos(6), dot(RIM));
  return c.map((v, i) => v + key * [36, 30, 22][i]! + rim * [6, 9, 14][i]!);
}
const w = Number(process.argv[3] ?? 64),
  h = Math.floor(w / 2),
  data = Buffer.alloc(w * h * 4);
for (let row = 0; row < h; row++)
  for (let col = 0; col < w; col++) {
    const acc = [0, 0, 0];
    for (let sy = 0; sy < 4; sy++)
      for (let sx = 0; sx < 4; sx++) {
        const la = (0.5 - (row + (sy + 0.5) / 4) / h) * Math.PI,
          lo = ((col + (sx + 0.5) / 4) / w - 0.5) * 2 * Math.PI;
        const r = radiance([Math.cos(la) * Math.cos(lo), Math.sin(la), Math.cos(la) * Math.sin(lo)]);
        for (let i = 0; i < 3; i++) acc[i]! += r[i]! / 16;
      }
    const v = Math.max(...acc),
      at = (row * w + col) * 4;
    if (v >= 1e-32) {
      const e = Math.floor(Math.log2(v)) + 1,
        m = v / 2 ** e,
        s = (m * 256) / v;
      data.set([...acc.map((c) => Math.min(255, Math.trunc(c * s))), e + 128], at);
    }
  }
writeFileSync(
  process.argv[2]!,
  Buffer.concat([Buffer.from(`#?RADIANCE\n# og-tests make-studio-hdr.ts\nFORMAT=32-bit_rle_rgbe\n\n-Y ${h} +X ${w}\n`), data]),
);
