// Re-encodes reference frames at maximum compression, pixel for pixel:
//
//   bun scripts/recompress-references.ts [FILE.png ...]
//
// Without arguments, every PNG git tracks under a `references/` directory.
// Each file is decoded, re-encoded with `encodePngBest` (packages/engine's
// src/node/png.ts, the encoder every recorder now uses: RGB when opaque,
// adaptive row filters, zlib level 9) and decoded again; it is replaced only
// when the decoded RGBA is identical and the new file is smaller. A PNG the
// decoder does not read (not 8-bit RGB or RGBA, interlaced) is left alone and
// listed. Prints each change and the totals; exits 1 when a re-encode did
// not decode to the same pixels.
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { decodePng, encodePngBest } from "../packages/engine/src/node/png.ts";

const repo = resolve(import.meta.dir, "..");
let files = process.argv.slice(2);
if (files.length === 0) {
  const ls = spawnSync("git", ["ls-files", "-z", "--", "*references/*.png"], { cwd: repo, encoding: "utf8" });
  files = ls.stdout.split("\0").filter((f) => /(^|\/)references\/.*\.png$/.test(f));
}
let before = 0;
let after = 0;
let changed = 0;
let mismatched = 0;
const unread: string[] = [];
for (const f of files) {
  const path = resolve(repo, f);
  const bytes = new Uint8Array(readFileSync(path));
  before += bytes.length;
  let img;
  try {
    img = decodePng(bytes);
  } catch (e) {
    unread.push(`${f} (${(e as Error).message})`);
    after += bytes.length;
    continue;
  }
  const out = encodePngBest(img);
  const back = decodePng(out);
  const same = back.width === img.width && back.height === img.height && Buffer.compare(Buffer.from(back.rgba), Buffer.from(img.rgba)) === 0;
  if (!same) {
    mismatched++;
    console.log(`MISMATCH ${f}: the re-encode decodes to other pixels; left alone`);
    after += bytes.length;
    continue;
  }
  if (out.length >= bytes.length) {
    after += bytes.length;
    continue;
  }
  writeFileSync(path, out);
  after += out.length;
  changed++;
  console.log(`${f}: ${bytes.length} -> ${out.length} (${(out.length / bytes.length).toFixed(2)}x)`);
}
for (const u of unread) console.log(`not read: ${u}`);
const mb = (n: number) => (n / 1e6).toFixed(1);
console.log(`${files.length} files, ${changed} re-encoded: ${mb(before)} MB -> ${mb(after)} MB (${(after / Math.max(1, before)).toFixed(3)}x)${mismatched ? `, ${mismatched} MISMATCHED` : ""}`);
process.exit(mismatched ? 1 : 0);
