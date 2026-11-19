// A split engine wasm's modules read from disk (docs/web-export.md §Modules):
// the web host run outside a browser (og build's probe, og explore, the
// cross-host tests) loads them from the directory beside the module, where
// scripts/build.sh and `og build --modules local` put them.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
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
  return { load: async (name, imports) => instance(name, imports), loadSync: instance };
}

/**
 * The split module each IDL module's members live in (scripts/wasm-modules.json):
 * a game naming one of its members references that module.
 */
const OWNER: Readonly<Record<string, readonly string[]>> = {
  physics: ["physics"],
  destruction: ["destruction", "physics"],
  nav: ["nav"],
  ai: ["nav"],
  sight: ["nav"],
  story: ["story"],
  film: ["story", "video"],
  motion: ["motion"],
  video: ["video"],
};
const SOURCES = /\.(ts|tsx|mts|js|mjs|svelte|html|ogt)$/;
const SKIP = new Set(["node_modules", "cooked", "dist", "og-out", ".og", ".git"]);

/**
 * The split modules a project references (og-build.json `modules`): a member
 * of their IDL modules named in its sources, scenes or documents (`og.RigidBody`,
 * `og.physics.castRay`, and a record's bare name in `.ogt` text). Read as
 * text, so it errs towards more: a module named nowhere but called anyway
 * still loads on its first call.
 */
export function referencedModules(project: string, bindings: { components: { name: string; module: string }[]; resources: { name: string; module: string }[]; events: { name: string; module: string }[] }): string[] {
  const records = new Map<string, readonly string[]>();
  for (const r of [...bindings.components, ...bindings.resources, ...bindings.events]) {
    const owner = OWNER[r.module];
    if (owner) records.set(r.name, owner);
  }
  const found = new Set<string>();
  const walk = (dir: string) => {
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      if (SKIP.has(e.name)) continue;
      const path = join(dir, e.name);
      if (e.isDirectory()) walk(path);
      else if (e.isFile() && SOURCES.test(e.name)) {
        // In .ogt text a record is a bare word; quoted strings are data (a translation's "Film").
        const raw = readFileSync(path, "utf8");
        const text = e.name.endsWith(".ogt") ? raw.replace(/"(?:[^"\\\n]|\\.)*"/g, '""') : raw;
        for (const m of text.matchAll(/\bog\.([a-z][A-Za-z0-9]*)\./g)) for (const o of OWNER[m[1]!] ?? []) found.add(o);
        const named = e.name.endsWith(".ogt") ? /\b(?:og\.)?([A-Z][A-Za-z0-9]*)\b/g : /\bog\.([A-Z][A-Za-z0-9]*)\b/g;
        for (const m of text.matchAll(named)) for (const o of records.get(m[1]!) ?? []) found.add(o);
      }
    }
  };
  walk(project);
  return [...found].sort();
}
