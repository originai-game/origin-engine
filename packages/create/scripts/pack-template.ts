// The skeleton is tracked in template/. Pack only its supporting catalogues.
import { cpSync, existsSync, lstatSync, mkdirSync, rmSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { TEMPLATE_SKIP } from "../../knowledge/scripts/lib.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const filter = (path: string) => !TEMPLATE_SKIP.has(basename(path)) && !lstatSync(path).isSymbolicLink();
// The style catalogue the look is chosen from, under ./looks/, and each
// style's preview under ./previews/: the catalogue's `preview`
// (previews/<slug>.webp) resolves against the package root here as in the
// knowledge package.
const catalogue = join(pkg, "..", "knowledge", "looks", "catalogue.json");
const looksTarget = join(pkg, "looks");
rmSync(looksTarget, { recursive: true, force: true });
if (!existsSync(catalogue)) {
  console.error(`pack-template: no style catalogue at ${catalogue}; run inside the origin-engine repository`);
  process.exit(1);
}
mkdirSync(looksTarget, { recursive: true });
cpSync(catalogue, join(looksTarget, "catalogue.json"));
rmSync(join(pkg, "previews"), { recursive: true, force: true });
cpSync(join(pkg, "..", "knowledge", "previews"), join(pkg, "previews"), { recursive: true, filter });
console.log(`pack-template: ${catalogue} -> ${looksTarget}/catalogue.json, previews/`);

// The art brief is the knowledge package's own (bin/knowledge.ts reads it
// beside create/), with the presets it carries: nothing of it is copied here.

const stylesTarget = join(pkg, "styles");
rmSync(stylesTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/styles"), stylesTarget, { recursive: true, filter });

// The font packs, installed by a project's languages (docs/ui.md §3.4).
const fontsTarget = join(pkg, "fonts");
rmSync(fontsTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/fonts"), fontsTarget, { recursive: true, filter: (path) => filter(path) && !path.endsWith(".py") });

// The interactive-film skeleton `genre interactive-film` adds (templates/film, its clip pinned in templates/og-assets.json).
const filmTarget = join(pkg, "film");
rmSync(filmTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/film"), filmTarget, { recursive: true, filter });

// Locale resources remain available independently of creation choices.
const localesTarget = join(pkg, "locales");
rmSync(localesTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/locales"), localesTarget, { recursive: true, filter: (path) => filter(path) && !path.endsWith(".py") });
