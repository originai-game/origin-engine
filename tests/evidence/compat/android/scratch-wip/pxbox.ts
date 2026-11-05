import { readFileSync } from "node:fs";
import { decodePng } from "../../../packages/engine/src/node/png.ts";
const a = decodePng(new Uint8Array(readFileSync(process.argv[2]!))), b = decodePng(new Uint8Array(readFileSync(process.argv[3]!)));
let x0 = 1e9, y0 = 1e9, x1 = -1, y1 = -1; const cols = new Map<string, number>();
for (let y = 0; y < a.height; y++) for (let x = 0; x < a.width; x++) {
  const i = (y * a.width + x) * 4; const d = Math.abs(a.rgba[i]! - b.rgba[i]!);
  if (d < 10) continue;
  x0 = Math.min(x0, x); y0 = Math.min(y0, y); x1 = Math.max(x1, x); y1 = Math.max(y1, y);
  const k = `${a.rgba[i]},${a.rgba[i+1]},${a.rgba[i+2]} vs ${b.rgba[i]},${b.rgba[i+1]},${b.rgba[i+2]}`; cols.set(k, (cols.get(k) ?? 0) + 1);
}
console.log(`box ${x0},${y0}..${x1},${y1}`); console.log([...cols].sort((p, q) => q[1] - p[1]).slice(0, 6));
