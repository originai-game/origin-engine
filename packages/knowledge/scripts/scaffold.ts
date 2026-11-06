// Writes or checks scaffold/distribution.json: the project template's binary
// files (models, textures, audio, fonts) with their sizes, digests and
// licences. The platform's scaffold payload is UTF-8 text only, so binaries
// travel from the engine distribution (the published @origingame/create-game
// template) and are verified against these digests, as the platform's
// game-scaffold/origin-engine/distribution.json does for its overlay.
//
//   bun scripts/scaffold.ts --write | --check

import { readFileSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { defaultRoots, isText, sha256, template2d, TEMPLATE_SKIP, walk } from "./lib.ts";

/** Licence per template file, from the template's LICENSES.md table. */
function licences(template: string): Map<string, string> {
  const out = new Map<string, string>();
  let text = "";
  try {
    text = readFileSync(join(template, "LICENSES.md"), "utf8");
  } catch {
    return out;
  }
  for (const line of text.split("\n")) {
    if (!line.startsWith("| `")) continue;
    const cells = line.split("|").map((c) => c.trim());
    for (const m of cells[1]!.matchAll(/`([^`]+)`/g)) out.set(m[1]!, `${cells[2]} (LICENSES.md)`);
  }
  return out;
}

export function distribution(template: string, profile: "3d" | "2d" = "3d"): string {
  const lic = licences(template);
  const files = walk(template, TEMPLATE_SKIP)
    .map((full) => ({ full, rel: relative(template, full).split("\\").join("/"), bytes: new Uint8Array(readFileSync(full)) }))
    .filter((f) => !isText(f.bytes))
    .map((f) => ({
      path: f.rel,
      bytes: f.bytes.length,
      sha256: sha256(f.bytes),
      licence: lic.get(f.rel) ?? "unrecorded: add it to the template's LICENSES.md",
      source: `templates/${profile === "2d" ? "game-2d" : "game"}/${f.rel} of origingame/origin-engine; shipped in @origingame/create-game's ${profile === "2d" ? "template-2d" : "template"}/`,
    }));
  return `${JSON.stringify(
    {
      schema: 1,
      about: "Binary files of the Origin Engine project template that a scaffolded Game needs but that are not knowledge text. The scaffolder copies them from the published template (@origingame/create-game) and verifies each digest.",
      files,
    },
    null,
    2,
  )}\n`;
}

if (import.meta.main) {
  const roots = defaultRoots();
  for (const profile of ["3d", "2d"] as const) {
    const name = profile === "3d" ? "scaffold/distribution.json" : "scaffold/2d/distribution.json";
    const template = profile === "3d" ? roots.template : template2d(roots);
    const path = join(roots.pkg, name);
    const want = distribution(template, profile);
    if (process.argv.includes("--write")) {
      writeFileSync(path, want);
      process.stdout.write(`wrote ${name} from ${relative(process.cwd(), template) || "."}\n`);
    } else {
      let have = "";
      try {
        have = readFileSync(path, "utf8");
      } catch {}
      if (have !== want) {
        process.stderr.write(`${name} differs from the template's binaries; run bun scripts/scaffold.ts --write\n`);
        process.exit(1);
      }
      process.stdout.write(`${name} matches the template.\n`);
    }
  }
}
