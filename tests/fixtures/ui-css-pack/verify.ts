// Compare a freshly rendered Metal sheet with the owner-approved cells.
// Run `og ui sheet assets/styles/css-laboratory --out <capture>` from here,
// then `bun verify.ts <capture>`. Reference recording is intentionally manual.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { decodePng } from "../../../packages/engine/src/node/png.ts";

const capture = process.argv[2];
assert(capture, "usage: bun verify.ts <sheet-capture-directory>");
const references = fileURLToPath(new URL("references/metal/", import.meta.url));
const expected = JSON.parse(readFileSync(resolve(references, "index.json"), "utf8"));
const actual = JSON.parse(readFileSync(resolve(capture, "index.json"), "utf8"));
assert(Array.isArray(expected.cells) && Array.isArray(actual.cells), "missing sheet cells");
assert.equal(actual.cells.length, expected.cells.length, "scroll-page coverage changed");
for (const [i, cell] of expected.cells.entries()) {
  const fresh = actual.cells[i];
  for (const key of ["screen", "breakpoint", "scale", "locale", "style"]) {
    assert.equal(fresh[key], cell[key], `cell ${i}: ${key}`);
  }
  assert.equal(typeof fresh.frame, "string", `cell ${i}: missing GPU capture`);
  const want = decodePng(readFileSync(resolve(references, cell.frame)));
  const got = decodePng(readFileSync(resolve(capture, fresh.frame)));
  assert.equal(got.width, want.width, `cell ${i}: width`);
  assert.equal(got.height, want.height, `cell ${i}: height`);
  assert(Buffer.from(got.rgba).equals(Buffer.from(want.rgba)), `cell ${i} (${cell.screen}, ${cell.breakpoint}, ${cell.scale}%): pixels differ; review before recording`);
}
console.log(`CSS pack sheet: ${actual.cells.length} Metal reference cells match`);
