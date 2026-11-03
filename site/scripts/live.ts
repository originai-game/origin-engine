// Builds the scenes and games the page runs live, as publish builds, into
// public/live/<name>/ (served same-origin; the hero and the Play section
// frame them). Uses this checkout's `og` (packages/engine/native, built by
// `OG_RELEASE=1 sh packages/engine/scripts/build.sh`) and its web host.
//
//   bun scripts/live.ts                 every entry below whose project exists
//   bun scripts/live.ts meridian-loop   one
import { existsSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import { $ } from "bun";

const site = join(import.meta.dir, "..");
const root = join(site, "..");
const pkg = join(root, "packages/engine");
const og = join(pkg, "native", `og.${process.platform}-${process.arch}${process.platform === "win32" ? ".exe" : ""}`);

/** name → project directory. `make` is the project's asset step, when its assets are not all committed. */
export const builds: Record<string, { project: string; make?: string }> = {
  hero: { project: "examples/site-hero" },
  ocean: { project: "examples/site-ocean" },
  motion: { project: "examples/site-motion" },
  "meridian-loop": { project: "examples/meridian-loop", make: "tools/make-assets.ts" },
  hollowbell: { project: "examples/hollowbell", make: "tools/make-assets.ts" },
  "kestrel-stair": { project: "examples/kestrel-stair", make: "tools/make-assets.ts" },
  "lantern-vale": { project: "examples/lantern-vale", make: "tools/make-assets.ts" },
  brasswick: { project: "examples/brasswick", make: "tools/make-assets.ts" },
  "rumpus-isle": { project: "examples/rumpus-isle", make: "tools/make-assets.ts" },
};

if (!existsSync(og)) throw new Error(`live: no ${og}; build the engine first (OG_RELEASE=1 sh packages/engine/scripts/build.sh)`);
const wanted = process.argv.slice(2);
for (const [name, b] of Object.entries(builds)) {
  if (wanted.length && !wanted.includes(name)) continue;
  const dir = join(root, b.project);
  if (!existsSync(join(dir, "project.ogt"))) {
    process.stdout.write(`${name}: ${b.project} does not exist yet, skipped\n`);
    continue;
  }
  if (b.make && existsSync(join(dir, b.make))) await $`bun ${b.make}`.cwd(dir).quiet();
  const out = join(site, "public/live", name);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  await $`${og} build ${dir} --publish --out ${out}`.env({ ...process.env, OG_ENGINE_PACKAGE: pkg });
  process.stdout.write(`${name}: built into public/live/${name}\n`);
}
