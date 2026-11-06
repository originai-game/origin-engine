// prepack: copies the project template's text files into scaffold/template/
// so the published package carries the scaffold overlay without the engine
// repository (binaries stay out: distribution.json names them), and the look
// presets with their palettes into presets/ (scripts/brief.ts). Both are
// ignored by Git.

import { cpSync, mkdirSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { defaultRoots, isText, PACKAGE_ROOT, TEMPLATE_SKIP, walk } from "./lib.ts";

for (const [from, to] of [["game", "template"], ["game-2d", "template-2d"]] as const) {
  const source = join(PACKAGE_ROOT, "..", "..", "templates", from);
  const target = join(PACKAGE_ROOT, "scaffold", to);
  if (defaultRoots().template !== join(PACKAGE_ROOT, "../../templates/game")) {
    process.stderr.write(`no ${source}: pack inside the engine repository\n`);
    process.exit(1);
  }
  rmSync(target, { recursive: true, force: true });
  let n = 0;
  for (const full of walk(source, TEMPLATE_SKIP)) {
    if (!isText(new Uint8Array(readFileSync(full)))) continue;
    // npm drops .gitignore from a tarball; TEMPLATE_RENAMED maps it back at mount.
    const dest = join(target, relative(source, full) === ".gitignore" ? "gitignore" : relative(source, full));
    mkdirSync(dirname(dest), { recursive: true });
    cpSync(full, dest);
    n++;
  }
  process.stdout.write(`copied ${n} template text files into scaffold/${to}/\n`);
}

// The look presets and the built-in palettes, under presets/: what
// scripts/brief.ts reads a look's chain and palette from outside the engine
// repository (templates/looks there). Not mounted for the platform.
const looks = join(PACKAGE_ROOT, "..", "..", "templates", "looks");
const presets = join(PACKAGE_ROOT, "presets");
rmSync(presets, { recursive: true, force: true });
mkdirSync(join(presets, "assets"), { recursive: true });
let p = 0;
for (const name of readdirSync(looks)) {
  if (!name.endsWith(".ogt")) continue;
  cpSync(join(looks, name), join(presets, name));
  p++;
}
cpSync(join(looks, "assets", "palettes"), join(presets, "assets", "palettes"), { recursive: true });
cpSync(join(looks, "moods"), join(presets, "moods"), { recursive: true });
process.stdout.write(`copied ${p} look presets, the moods and the built-in palettes into presets/\n`);
