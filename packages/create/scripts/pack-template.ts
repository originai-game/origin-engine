// The skeleton is tracked in template/. Styles, fonts, and the look catalogue
// live once in @origingame/engine beside create/ (never duplicated here); only
// the interactive-film starter overlay (`templates/film`) is packed into create/.
import { cpSync, lstatSync, rmSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { TEMPLATE_SKIP } from "../../knowledge/scripts/lib.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const filter = (path: string) => !TEMPLATE_SKIP.has(basename(path)) && !lstatSync(path).isSymbolicLink();

// Remove legacy duplicated asset directories if an older checkout left them behind.
for (const legacy of ["looks", "previews", "styles", "fonts", "locales"]) {
  rmSync(join(pkg, legacy), { recursive: true, force: true });
}

// The interactive-film skeleton `genre interactive-film` adds (templates/film, its clip pinned in templates/og-assets.json).
const filmTarget = join(pkg, "film");
rmSync(filmTarget, { recursive: true, force: true });
cpSync(join(pkg, "../../templates/film"), filmTarget, { recursive: true, filter });
console.log(`pack-template: templates/film -> ${filmTarget}`);
