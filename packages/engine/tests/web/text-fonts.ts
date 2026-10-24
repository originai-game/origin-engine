// og-ui's cooked-font text frames (text-bidi, text-kerning, text-edit,
// text-mixed-runs) draw with fonts og-tests-web cannot cook in the browser:
// this cooks them first, as og-ui's native test does (tests/text_gfx.rs).
// crates/og-ui/tests/text-fonts (a project of the fonts' import settings,
// `assets/fonts/<file>.ogt`) is copied into `dir` with each settings
// file's font from templates/ui-kit/assets/fonts, and cooked with `og cook`
// (tests/support/og-bin.ts: $OG_BIN, else packages/engine/native/). The
// harness serves `<dir>/cooked/` and hands og-tests-web the manifest and
// every file (og_gfx_text_manifest, og_gfx_text_file); the module decodes
// them with og-cook's loaders (crates/og-ui/tests/support/text_scenes.rs).

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { join } from "node:path";
import { cookProject } from "../support/og-bin.ts";

declare module "node:fs" {
  export function cpSync(from: string, to: string, options?: { recursive?: boolean }): void;
  export function readdirSync(path: string): string[];
}

export interface TextFonts {
  /** `cooked/` of the scratch project. */
  readonly cooked: string;
  /** Its manifest (`cooked/.cook.ogt`). */
  readonly manifest: string;
  /** Every blob under `cooked/`, by path under it (`fonts/<file>.ogb`). */
  readonly files: readonly string[];
  /** The fonts cooked: the font files the settings name, sorted. */
  readonly fonts: readonly string[];
}

/** Cooks crates/og-ui/tests/text-fonts into `dir` (emptied first); `repo` is the repository root. */
export function cookTextFonts(repo: string, dir: string): TextFonts {
  const project = join(repo, "crates", "og-ui", "tests", "text-fonts");
  const sources = join(repo, "templates", "ui-kit", "assets", "fonts");
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  cpSync(project, dir, { recursive: true });
  const fonts = readdirSync(join(project, "assets", "fonts"))
    .filter((f) => f.endsWith(".ogt"))
    .map((f) => f.slice(0, -".ogt".length))
    .sort();
  for (const f of fonts) {
    if (!existsSync(join(sources, f))) throw new Error(`text-fonts names ${f}, which templates/ui-kit/assets/fonts does not have`);
    cpSync(join(sources, f), join(dir, "assets", "fonts", f));
  }
  cookProject(dir, repo);
  const cooked = join(dir, "cooked");
  const files: string[] = [];
  const walk = (rel: string) => {
    for (const name of readdirSync(join(cooked, rel))) {
      const path = rel ? `${rel}/${name}` : name;
      if (statSync(join(cooked, path)).isDirectory()) walk(path);
      else if (path.endsWith(".ogb")) files.push(path);
    }
  };
  walk("");
  files.sort();
  return { cooked, manifest: readFileSync(join(cooked, ".cook.ogt"), "utf8"), files, fonts };
}
