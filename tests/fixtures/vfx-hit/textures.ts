// Writes the fixture's only textures: spark, tiling noise, puff and decal atlas.
//   bun tests/fixtures/vfx-hit/textures.ts [fixture directory]
import { join } from "node:path";
import { Random } from "../../../scripts/random.ts";
import { PixelArt } from "../../../scripts/pixel-art.ts";
const here = join(process.argv[2] ?? import.meta.dir, "assets/textures");
const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(Math.max((x - a) / (b - a), 0), 1);
  return t * t * (3 - 2 * t);
};
const octaves = Array.from({ length: 4 }, (_, k) => {
  const p = 4 << k,
    r = new Random(7 + k);
  return { p, g: Array.from({ length: p }, () => Array.from({ length: p }, () => r.random())), w: 0.5 ** k };
});
function fbm(u: number, v: number): number {
  let sum = 0,
    total = 0;
  for (const { p, g, w } of octaves) {
    const x = u * p,
      y = v * p,
      i = Math.floor(x),
      j = Math.floor(y);
    let fx = x - i,
      fy = y - j;
    fx = fx * fx * (3 - 2 * fx);
    fy = fy * fy * (3 - 2 * fy);
    const a = g[j % p]![i % p]!,
      b = g[j % p]![(i + 1) % p]!,
      c = g[(j + 1) % p]![i % p]!,
      d = g[(j + 1) % p]![(i + 1) % p]!;
    sum += w * ((a + (b - a) * fx) * (1 - fy) + (c + (d - c) * fx) * fy);
    total += w;
  }
  return sum / total;
}
const white = (a: number) => [255, 255, 255, Math.trunc(255 * a)];
function save(name: string, size: number, f: (u: number, v: number) => number[], h = size): void {
  const image = new PixelArt(size, h);
  for (let y = 0; y < h; y++) for (let x = 0; x < size; x++) image.put(x, y, f((x + 0.5) / size, (y + 0.5) / h));
  image.save(join(here, name));
}
save("spark.png", 64, (u, v) => {
  const d = Math.hypot(u - 0.5, v - 0.5) * 2;
  return white(0.75 * (1 - smooth(0, 0.25, d)) ** 2 + 0.25 * (1 - smooth(0.1, 1, d)) ** 2);
});
save("noise.png", 128, (u, v) => {
  const g = Math.trunc(255 * Math.min(Math.max((fbm(u, v) - 0.2) / 0.6, 0), 1));
  return [g, g, g];
});
save("puff.png", 64, (u, v) => white((1 - smooth(0.1, 1, Math.hypot(u - 0.5, v - 0.5) * 2)) ** 1.5 * (0.6 + 0.4 * fbm(u, v))));
save(
  "atlas.png",
  512,
  (u, v) => {
    if (u < 0.5) {
      const x = (u - 0.25) * 4,
        y = (v - 0.5) * 2,
        d = Math.hypot(x, y),
        a = Math.atan2(y, x);
      const rays = 0.5 + 0.5 * Math.cos(a * 9 + 3 * fbm(u, v)),
        reach = 0.55 + 0.4 * rays * fbm(u * 2, v * 2);
      const alpha = (1 - smooth(reach * 0.6, reach, d)) * 0.95,
        burn = 0.03 + 0.05 * fbm(u * 3, v * 3);
      return [Math.trunc(255 * burn), Math.trunc(255 * burn * 0.8), Math.trunc(255 * burn * 0.7), Math.trunc(255 * alpha)];
    }
    const x = (u - 0.75) * 4,
      y = (v - 0.5) * 2,
      d = Math.hypot(x, y),
      a = Math.atan2(y, x);
    const ring = Math.max(1 - Math.abs(d - 0.85) / 0.05, 0) + Math.max(1 - Math.abs(d - 0.62) / 0.03, 0);
    const spokes = Math.max(1 - Math.abs(Math.sin(a * 3)) * d * 12, 0) * (d < 0.62 ? 1 : 0),
      g = Math.min(ring + spokes, 1) * (1 - smooth(0.9, 0.98, d));
    return [Math.trunc(255 * 0.55), 255, Math.trunc(255 * 0.7), Math.trunc(255 * g)];
  },
  256,
);
