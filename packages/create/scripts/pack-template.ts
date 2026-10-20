// prepack: copies the engine repository's templates/game into ./template/,
// so the published package carries the template it scaffolds. Build output
// (cooked/, dist/, node_modules/) is left out.
import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { SKIPPED } from "../bin/create.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const source = join(pkg, "..", "..", "templates", "game");
const target = join(pkg, "template");
if (!existsSync(join(source, "project.ogt"))) {
  console.error(`pack-template: no template at ${source}; run inside the origin-engine repository`);
  process.exit(1);
}
rmSync(target, { recursive: true, force: true });
cpSync(source, target, {
  recursive: true,
  filter: (path) => !(dirname(path) === source && SKIPPED.has(basename(path))),
});
console.log(`pack-template: ${source} -> ${target}`);
// The archetypes, each a complete project, under ./archetypes/<name>/.
const archetypes = join(pkg, "..", "..", "templates", "archetypes");
const archTarget = join(pkg, "archetypes");
rmSync(archTarget, { recursive: true, force: true });
if (existsSync(archetypes)) {
  cpSync(archetypes, archTarget, {
    recursive: true,
    filter: (path) => !SKIPPED.has(basename(path)),
  });
  console.log(`pack-template: ${archetypes} -> ${archTarget}`);
}
// The style catalogue the look is chosen from, under ./looks/.
const catalogue = join(pkg, "..", "knowledge", "looks", "catalogue.json");
const looksTarget = join(pkg, "looks");
rmSync(looksTarget, { recursive: true, force: true });
if (!existsSync(catalogue)) {
  console.error(`pack-template: no style catalogue at ${catalogue}; run inside the origin-engine repository`);
  process.exit(1);
}
mkdirSync(looksTarget, { recursive: true });
cpSync(catalogue, join(looksTarget, "catalogue.json"));
console.log(`pack-template: ${catalogue} -> ${looksTarget}/catalogue.json`);
