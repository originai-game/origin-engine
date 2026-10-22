// prepack: copies the engine repository's templates/game into ./template/
// and templates/game-2d into ./template-2d/, so the published package
// carries the templates it scaffolds (one per profile). Build output
// (cooked/, dist/, node_modules/) is left out.
import { cpSync, existsSync, mkdirSync, readdirSync, renameSync, rmSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { SKIPPED } from "../bin/create.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
// npm drops .gitignore from a tarball: stored as `gitignore`, create writes it back as `.gitignore` (RENAMED).
function undot(project: string) {
  if (existsSync(join(project, ".gitignore"))) renameSync(join(project, ".gitignore"), join(project, "gitignore"));
}
for (const [from, to] of [
  ["game", "template"],
  ["game-2d", "template-2d"],
]) {
  const source = join(pkg, "..", "..", "templates", from!);
  const target = join(pkg, to!);
  if (!existsSync(join(source, "project.ogt"))) {
    console.error(`pack-template: no template at ${source}; run inside the origin-engine repository`);
    process.exit(1);
  }
  rmSync(target, { recursive: true, force: true });
  cpSync(source, target, {
    recursive: true,
    filter: (path) => !(dirname(path) === source && SKIPPED.has(basename(path))),
  });
  undot(target);
  console.log(`pack-template: ${source} -> ${target}`);
}
// The archetypes, each a complete project, under ./archetypes/<name>/.
const archetypes = join(pkg, "..", "..", "templates", "archetypes");
const archTarget = join(pkg, "archetypes");
rmSync(archTarget, { recursive: true, force: true });
if (existsSync(archetypes)) {
  cpSync(archetypes, archTarget, {
    recursive: true,
    filter: (path) => !SKIPPED.has(basename(path)),
  });
  for (const a of readdirSync(archTarget)) if (existsSync(join(archTarget, a, "project.ogt"))) undot(join(archTarget, a));
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
