// Draws the UI effects' CC0 white/grey particle masks, 128 px with alpha.
//   bun templates/tools/make-fx-art.ts [repo]
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { encodePngBest } from "../../packages/engine/src/node/png.ts";

const clamp = (v: number) => Math.max(0, Math.min(1, v));
const bell = (v: number) => Math.exp(-(v ** 2));
// NumPy rounds halfway to even, not towards positive infinity.
function byte(v: number): number {
  const n = clamp(v) * 255,
    i = Math.floor(n);
  return n - i === 0.5 ? i + (i % 2) : Math.round(n);
}
const beam = (u: number, v: number) => Math.exp(-Math.abs(v) / (0.035 + 0.05 * clamp(1 - Math.abs(u)))) * clamp(1 - Math.abs(u)) ** 1.6;
const out = join(process.argv[2] ?? ".", "templates/styles/shared/fx");
mkdirSync(out, { recursive: true });
for (const name of ["sparkle", "glint", "shard", "coin", "ray", "ring"]) {
  const rgba = new Uint8Array(128 * 128 * 4);
  for (let j = 0; j < 128; j++)
    for (let i = 0; i < 128; i++) {
      const x = ((i + 0.5) / 128) * 2 - 1,
        y = ((j + 0.5) / 128) * 2 - 1;
      const r = Math.hypot(x, y),
        edge = clamp((1 - r) / 0.08);
      let grey = 1,
        alpha = 0;
      if (name === "sparkle") {
        const d1 = (x + y) / Math.sqrt(2),
          d2 = (x - y) / Math.sqrt(2);
        const star = Math.max(beam(x, y), beam(y, x), 0.45 * Math.max(beam(d1 * 1.8, d2), beam(d2 * 1.8, d1)));
        alpha = clamp(star + bell(r / 0.16) + bell(r / 0.42) * 0.35) * edge;
      } else if (name === "glint") alpha = clamp(bell(r / 0.22) + bell(r / 0.6) * 0.45) * edge;
      else if (name === "shard") {
        const q = Math.abs(x) / 0.42 + Math.abs(y) / 0.92;
        alpha = clamp((1 - q) * 30);
        grey = clamp((x < 0 ? 1 : 0.62) * (y < 0 ? 1 : 0.85) + clamp(1 - Math.abs(q - 0.93) * 18) * 0.5);
      } else if (name === "coin") {
        alpha = clamp((0.94 - r) * 40);
        grey = clamp((r - 0.74) * 40) > 0.5 ? 0.96 - (0.3 * (y + 1)) / 2 : 0.78 - 0.28 * y;
        if (clamp(1 - (Math.abs(x) / 0.28 + Math.abs(y) / 0.42)) > 0) grey = 1;
        grey += Math.exp(-(((x + 0.32) / 0.22) ** 2 + ((y + 0.38) / 0.16) ** 2)) * 0.35;
      } else if (name === "ray")
        alpha = clamp(bell(x / 0.16) * clamp(1 - Math.abs(y)) ** 0.8 + bell(x / 0.04) * clamp(1 - Math.abs(y)) * 0.6);
      else alpha = clamp(bell((r - 0.82) / 0.035) + bell((r - 0.82) / 0.14) * 0.4) * edge;
      const at = (j * 128 + i) * 4;
      rgba.set([byte(grey), byte(grey), byte(grey), byte(alpha)], at);
    }
  writeFileSync(join(out, `${name}.png`), encodePngBest({ width: 128, height: 128, rgba }));
}
