// Scratch: per-channel signed mean, histogram of |diff| and where it sits, for two same-size PNGs.
//   bun pxdiff.ts <a.png> <b.png>
import { readFileSync } from "node:fs";
import { decodePng } from "../../../packages/engine/src/node/png.ts";

const a = decodePng(new Uint8Array(readFileSync(process.argv[2]!)));
const b = decodePng(new Uint8Array(readFileSync(process.argv[3]!)));
const n = a.width * a.height;
const mean = [0, 0, 0];
const hist = new Map<number, number>();
let changed = 0;
for (let i = 0; i < n; i++) {
  let m = 0;
  for (let c = 0; c < 3; c++) {
    const d = a.rgba[i * 4 + c]! - b.rgba[i * 4 + c]!;
    mean[c]! += d / n;
    m = Math.max(m, Math.abs(d));
  }
  if (m) changed++;
  hist.set(m, (hist.get(m) ?? 0) + 1);
}
console.log(`${a.width}x${a.height} changed ${((changed / n) * 100).toFixed(1)} %, mean a-b rgb ${mean.map((v) => v.toFixed(2)).join(" ")}`);
console.log([...hist].sort((x, y) => x[0] - y[0]).filter(([k]) => k > 0).map(([k, v]) => `${k}:${v}`).join(" "));
// Signed mean along the bars' left and right edges (smaa): dark->light vs light->dark transitions on each row.
if (process.argv[4] === "edges") {
  const lum = (img: typeof a, i: number) => img.rgba[i * 4]! + img.rgba[i * 4 + 1]! + img.rgba[i * 4 + 2]!;
  const side = { fall: [0, 0], rise: [0, 0] };
  for (let y = 0; y < a.height; y++)
    for (let x = 2; x < a.width - 2; x++) {
      const i = y * a.width + x;
      const g = lum(b, i + 2) - lum(b, i - 2);
      if (Math.abs(g) < 200) continue;
      const s = g < 0 ? side.fall : side.rise;
      s[0]! += lum(a, i) - lum(b, i);
      s[1]!++;
    }
  console.log(`edge a-b luminance: light->dark ${(side.fall[0]! / side.fall[1]!).toFixed(1)} (${side.fall[1]} px), dark->light ${(side.rise[0]! / side.rise[1]!).toFixed(1)} (${side.rise[1]} px)`);
}
