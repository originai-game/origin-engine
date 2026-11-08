// The skeleton is tracked in template/. Pack only its supporting catalogues.
import { cpSync, existsSync, lstatSync, mkdirSync, readdirSync, rmSync } from "node:fs";
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

// The art brief: bin/brief.ts is the knowledge package's scripts/brief.ts,
// byte for byte (create.ts re-exports it; tests/create.test.ts fails on a
// difference), and the look presets with the built-in palettes it reads
// under ./presets/.
const brief = join(pkg, "..", "knowledge", "scripts", "brief.ts");
cpSync(brief, join(pkg, "bin", "brief.ts"));
const looksDir = join(pkg, "..", "..", "templates", "looks");
const presetsTarget = join(pkg, "presets");
rmSync(presetsTarget, { recursive: true, force: true });
mkdirSync(join(presetsTarget, "assets"), { recursive: true });
for (const name of readdirSync(looksDir)) if (name.endsWith(".ogt")) cpSync(join(looksDir, name), join(presetsTarget, name));
cpSync(join(looksDir, "assets", "palettes"), join(presetsTarget, "assets", "palettes"), { recursive: true, filter });
cpSync(join(looksDir, "moods"), join(presetsTarget, "moods"), { recursive: true, filter });
console.log(`pack-template: ${brief} -> bin/brief.ts, ${looksDir} -> presets/`);

const stylesTarget = join(pkg, "styles");
rmSync(stylesTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/styles"), stylesTarget, { recursive: true, filter });

// The font packs, installed by a project's languages (docs/ui.md §3.4).
const fontsTarget = join(pkg, "fonts");
rmSync(fontsTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/fonts"), fontsTarget, { recursive: true, filter: (path) => filter(path) && !path.endsWith(".py") });

// Locale resources remain available independently of creation choices.
const localesTarget = join(pkg, "locales");
rmSync(localesTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/locales"), localesTarget, { recursive: true, filter: (path) => filter(path) && !path.endsWith(".py") });
