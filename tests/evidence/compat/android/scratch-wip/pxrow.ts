import { readFileSync } from "node:fs";
import { decodePng } from "../../../packages/engine/src/node/png.ts";
const a = decodePng(new Uint8Array(readFileSync(process.argv[2]!))), b = decodePng(new Uint8Array(readFileSync(process.argv[3]!)));
const row = (img: typeof a, y: number) => Array.from({ length: 40 }, (_, x) => img.rgba[(y * img.width + x) * 4]).join(" ");
console.log("phone y10:", row(a, 10)); console.log("metal y10:", row(b, 10));
const xs = new Set<number>(); let blackMetal = 0;
for (let y = 0; y < a.height; y++) for (let x = 0; x < a.width; x++) { const i = (y * a.width + x) * 4; if (b.rgba[i] === 0) { blackMetal++; if (a.rgba[i] === 13) xs.add(x); } }
console.log("metal black px", blackMetal, "affected columns", [...xs].sort((p, q) => p - q).slice(0, 40).join(","));
