// Compare a fresh capture of the rpg review pages with the approved
// references: the five pages at desktopHd, phonePortrait and phoneLandscape,
// 100 %, game locale, in the fantasy pack. On demand, not a gate step; the
// README says how to capture. Recording is manual: review every changed cell
// with art before replacing one.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { decodePng } from "../../../src/node/png.ts";

function assert(ok: unknown, message: string): asserts ok {
  if (!ok) throw new Error(message);
}
const same = (a: Uint8Array, b: Uint8Array): boolean => a.length === b.length && a.every((x, i) => x === b[i]);

const capture = process.argv[2];
assert(capture, "usage: bun verify.ts <preview-capture-directory>");
const references = fileURLToPath(new URL("references/vulkan/", import.meta.url));
const expected = JSON.parse(readFileSync(resolve(references, "index.json"), "utf8")) as { cells: Record<string, string | number>[] };
const actual = JSON.parse(readFileSync(resolve(capture, "index.json"), "utf8")) as { cells: Record<string, string | number>[] };
assert(actual.cells.length === expected.cells.length, "cell coverage changed");
for (const [i, cell] of expected.cells.entries()) {
  const fresh = actual.cells[i]!;
  for (const key of ["screen", "breakpoint", "scale", "locale"]) assert(fresh[key] === cell[key], `cell ${i}: ${key}`);
  const want = decodePng(readFileSync(resolve(references, String(cell.frame))));
  const got = decodePng(readFileSync(resolve(capture, String(fresh.frame))));
  assert(got.width === want.width, `cell ${i}: width`);
  assert(got.height === want.height, `cell ${i}: height`);
  assert(same(got.rgba, want.rgba), `cell ${i} (${cell.screen}, ${cell.breakpoint}): pixels differ; review before recording`);
}
console.log(`rpg sheet: ${actual.cells.length} L4 reference cells match`);
