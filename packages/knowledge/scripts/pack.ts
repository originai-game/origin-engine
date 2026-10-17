// prepack: copies the project template's text files into scaffold/template/
// so the published package carries the scaffold overlay without the engine
// repository (binaries stay out: distribution.json names them). Ignored by Git.

import { cpSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { defaultRoots, isText, PACKAGE_ROOT, TEMPLATE_SKIP, walk } from "./lib.ts";

const source = join(PACKAGE_ROOT, "..", "..", "templates", "game");
const target = join(PACKAGE_ROOT, "scaffold", "template");
if (defaultRoots().template !== source) {
  process.stderr.write(`no ${source}: pack inside the engine repository\n`);
  process.exit(1);
}
rmSync(target, { recursive: true, force: true });
let n = 0;
for (const full of walk(source, TEMPLATE_SKIP)) {
  if (!isText(new Uint8Array(readFileSync(full)))) continue;
  const to = join(target, relative(source, full));
  mkdirSync(dirname(to), { recursive: true });
  cpSync(full, to);
  n++;
}
process.stdout.write(`copied ${n} template text files into scaffold/template/\n`);
