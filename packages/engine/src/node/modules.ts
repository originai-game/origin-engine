// A split engine wasm's modules read from disk (docs/web-export.md §Modules):
// the web host run outside a browser (og build's probe, og explore, the
// cross-host tests) loads them from the directory beside the module, where
// scripts/build.sh and `og build --modules local` put them.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import type { ModuleHost } from "../web/split.ts";

/** The modules listed in `<dir>/modules[-lanes]/modules.json`; none (an unsplit module) when it is absent. */
export function fileModuleHost(dir: string, lanes = false): ModuleHost {
  const listing = join(dir, lanes ? "modules-lanes" : "modules", "modules.json");
  const listed = existsSync(listing) ? (JSON.parse(readFileSync(listing, "utf8")) as Record<string, { file: string }>) : {};
  const instance = (name: string, imports: WebAssembly.Imports) => {
    const entry = listed[name];
    if (!entry) throw new Error(`engine module ${name} is not in ${listing}`);
    return new WebAssembly.Instance(new WebAssembly.Module(new Uint8Array(readFileSync(join(dir, entry.file)))), imports);
  };
  // The lanes pool's Workers read the same files (scripts/og-pool-split.js).
  const pool = existsSync(listing) ? { base: pathToFileURL(join(dir, "/")).href, files: Object.fromEntries(Object.entries(listed).map(([m, e]) => [m, e.file])) } : undefined;
  return { load: async (name, imports) => instance(name, imports), loadSync: instance, ...(pool ? { pool } : {}) };
}

interface Bindings {
  splits: Readonly<Record<string, readonly string[]>>;
  components: { name: string; split?: readonly string[] }[];
  resources: { name: string; split?: readonly string[] }[];
  events: { name: string; split?: readonly string[] }[];
}
const SOURCES = /\.(ts|tsx|mts|js|mjs|svelte|html)$/;
const SKIP = new Set(["node_modules", "cooked", "dist", "og-out", ".og", ".git"]);

/**
 * The engine modules opening a cooked bundle needs: its manifest's
 * top-level `modules` line (the boot scene's, og-cook's pipeline/needs.rs);
 * none when the line is absent.
 */
export function cookedModules(cooked: string): string[] {
  const listing = join(cooked, ".cook.ogt");
  if (!existsSync(listing)) return [];
  const line = /^modules((?: [a-z0-9]+)+)[ \t]*$/m.exec(readFileSync(listing, "utf8"));
  return line ? line[1]!.trim().split(" ") : [];
}

/**
 * The physics simulation a cooked bundle's project steps: `physics2d` for
 * its manifest's `profile "2d"`, else `physics3d` (as og-cook's
 * pipeline/needs.rs names a scene's).
 */
function simulation(cooked: string): string {
  const listing = join(cooked, ".cook.ogt");
  const text = existsSync(listing) ? readFileSync(listing, "utf8") : "";
  return /^profile "2d"[ \t]*$/m.test(text) ? "physics2d" : "physics3d";
}

/**
 * The split modules a project references (og-build.json `modules`): those
 * its boot content needs as the cook recorded them ({@link cookedModules}),
 * and those whose IDL module's members its code names (`og.RigidBody`,
 * `og.physics.castRay`; bindings.json `splits` and each record's `split`),
 * with `physics` the simulation of the project's profile.
 * Code is read as text, so it errs towards more; a module named nowhere but
 * called anyway still loads on its first call, and later scenes' modules
 * load as they are prepared (og.scene.preload, og.load.scene, a pre-roll).
 */
export function referencedModules(project: string, bindings: Bindings): string[] {
  const records = new Map<string, readonly string[]>();
  for (const r of [...bindings.components, ...bindings.resources, ...bindings.events]) if (r.split) records.set(r.name, r.split);
  const found = new Set<string>(cookedModules(join(project, "cooked")));
  const walk = (dir: string) => {
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      if (SKIP.has(e.name)) continue;
      const path = join(dir, e.name);
      if (e.isDirectory()) walk(path);
      else if (e.isFile() && SOURCES.test(e.name)) {
        const text = readFileSync(path, "utf8");
        for (const m of text.matchAll(/\bog\.([a-z][A-Za-z0-9]*)\./g)) for (const o of bindings.splits[m[1]!] ?? []) found.add(o);
        for (const m of text.matchAll(/\bog\.([A-Z][A-Za-z0-9]*)\b/g)) for (const o of records.get(m[1]!) ?? []) found.add(o);
      }
    }
  };
  walk(project);
  if (found.has("physics") && !found.has("physics2d") && !found.has("physics3d")) found.add(simulation(join(project, "cooked")));
  return [...found].sort();
}
