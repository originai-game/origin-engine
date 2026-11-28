// Regenerates the template's CC0 stone textures and mono blip.wav.
//   bun templates/tools/make-template-assets.ts [templates/game/assets]
// Seeded MT19937, including integer-seed initialization and 53-bit sampling,
// preserves the original generator's lattice (seed 20261017).
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { encodePngBest } from "../../packages/engine/src/node/png.ts";
import { Random } from "../../scripts/random.ts";

const out = process.argv[2] ?? "templates/game/assets";
const rng = new Random(20261017);
const random = () => rng.random();
const N = 512;
const octs = Array.from({ length: 6 }, (_, o) => {
  const n = 4 * 2 ** o;
  return { n, g: Array.from({ length: n + 1 }, () => Array.from({ length: n + 1 }, random)) };
});
function noise(x: number, y: number): number {
  let v = 0,
    a = 0.5,
    total = 0;
  for (const { n, g } of octs) {
    const fx = x * n,
      fy = y * n,
      ix = Math.floor(fx) % n,
      iy = Math.floor(fy) % n;
    const tx = fx - Math.floor(fx),
      ty = fy - Math.floor(fy);
    const sx = tx * tx * (3 - 2 * tx),
      sy = ty * ty * (3 - 2 * ty);
    const a0 = g[iy]![ix]! + (g[iy]![ix + 1]! - g[iy]![ix]!) * sx;
    const a1 = g[iy + 1]![ix]! + (g[iy + 1]![ix + 1]! - g[iy + 1]![ix]!) * sx;
    v += a * (a0 + (a1 - a0) * sy);
    total += a;
    a *= 0.5;
  }
  return v / total;
}
const h = Array.from({ length: N }, (_, y) =>
  Float64Array.from({ length: N }, (_, x) => {
    const u = x / N,
      v = y / N;
    const grout = Math.max(Math.abs(((u * 4) % 1) - 0.5), Math.abs(((v * 4) % 1) - 0.5)) > 0.46 ? 1 : 0;
    return noise(u, v) * 0.8 + 0.2 - grout * 0.5;
  }),
);
const color = new Uint8Array(N * N * 4),
  normal = new Uint8Array(N * N * 4);
for (let y = 0; y < N; y++)
  for (let x = 0; x < N; x++) {
    const tile = Math.floor((x / N) * 4) + Math.floor((y / N) * 4) * 4;
    const hue = [
      [0.62, 0.55, 0.48],
      [0.55, 0.57, 0.6],
      [0.66, 0.6, 0.5],
    ][tile % 3]!;
    const k = 0.55 + 0.6 * h[y]![x]!;
    const dx = (h[y]![(x + 1) % N]! - h[y]![(x + N - 1) % N]!) * 4;
    const dy = (h[(y + 1) % N]![x]! - h[(y + N - 1) % N]![x]!) * 4;
    const l = Math.sqrt(dx * dx + dy * dy + 1),
      i = (y * N + x) * 4;
    color.set([...hue.map((q) => Math.max(0, Math.min(255, Math.trunc(255 * q * k)))), 255], i);
    normal.set(
      [Math.trunc(((-dx / l) * 0.5 + 0.5) * 255), Math.trunc(((-dy / l) * 0.5 + 0.5) * 255), Math.trunc(((1 / l) * 0.5 + 0.5) * 255), 255],
      i,
    );
  }
mkdirSync(join(out, "textures"), { recursive: true });
mkdirSync(join(out, "audio"), { recursive: true });
for (const [name, rgba] of [
  ["stone", color],
  ["stone_n", normal],
] as const) {
  writeFileSync(join(out, `textures/${name}.png`), encodePngBest({ width: N, height: N, rgba }));
}

const sr = 44100,
  count = Math.floor(sr * 0.5),
  wav = Buffer.alloc(44 + count * 2);
wav.write("RIFF");
wav.writeUInt32LE(wav.length - 8, 4);
wav.write("WAVEfmt ", 8);
wav.writeUInt32LE(16, 16);
wav.writeUInt16LE(1, 20);
wav.writeUInt16LE(1, 22);
wav.writeUInt32LE(sr, 24);
wav.writeUInt32LE(sr * 2, 28);
wav.writeUInt16LE(2, 32);
wav.writeUInt16LE(16, 34);
wav.write("data", 36);
wav.writeUInt32LE(count * 2, 40);
let ph = 0;
for (let i = 0; i < count; i++) {
  const t = i / sr,
    f = t < 0.25 ? 660 * 2 ** (t / 0.25) : 1320;
  ph += (2 * Math.PI * f) / sr;
  const env = Math.exp(-t * 7) * Math.min(1, t * 400);
  const s = env * (0.7 * Math.sin(ph) + 0.2 * Math.sin(2 * ph) + 0.1 * Math.sin(3 * ph));
  wav.writeInt16LE(Math.trunc(Math.max(-1, Math.min(1, s)) * 30000), 44 + i * 2);
}
writeFileSync(join(out, "audio/blip.wav"), wav);
