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

/** Whether a cooked bundle's project profile is 2D (`profile "2d"` in `.cook.ogt`). */
function is2dProfile(cooked: string): boolean {
  const listing = join(cooked, ".cook.ogt");
  const text = existsSync(listing) ? readFileSync(listing, "utf8") : "";
  return /^profile "2d"[ \t]*$/m.test(text);
}

/** The words after which a `/` starts a regular expression, not a division. */
const BEFORE_REGEX = new Set(["return", "typeof", "instanceof", "in", "of", "new", "delete", "void", "throw", "case", "do", "else", "yield", "await"]);

/**
 * JS/TS source without its comments and regular expression literals (each
 * a space), its strings and template literals kept (a template's `${…}` is
 * code again): a `"` or a `//` inside a regular expression opens no string
 * or comment. A `/` starts a regular expression where an operand is due
 * (after an operator, an opening bracket, a keyword, or nothing; never
 * after `<`, which is a JSX or HTML closing tag's); one that does not close
 * on its line was a division.
 */
function stripComments(text: string): string {
  let out = "";
  let at = 0;
  /** Code from `at` to the end, or to the `}` closing a template's `${` (`nested`). */
  const code = (nested: boolean): void => {
    let depth = 0;
    let last = "";
    while (at < text.length) {
      const c = text[at]!;
      const next = text[at + 1];
      if (c === "/" && next === "/") {
        while (at < text.length && text[at] !== "\n") at++;
        out += " ";
      } else if (c === "/" && next === "*") {
        const end = text.indexOf("*/", at + 2);
        at = end < 0 ? text.length : end + 2;
        out += " ";
      } else if (c === '"' || c === "'") {
        const from = at++;
        while (at < text.length && text[at] !== c && text[at] !== "\n") at += text[at] === "\\" ? 2 : 1;
        at++;
        out += text.slice(from, at);
        last = c;
      } else if (c === "`") {
        out += text[at++];
        while (at < text.length && text[at] !== "`") {
          if (text[at] === "\\") {
            out += text.slice(at, at + 2);
            at += 2;
          } else if (text[at] === "$" && text[at + 1] === "{") {
            out += "${";
            at += 2;
            code(true);
          } else out += text[at++];
        }
        if (at < text.length) out += text[at++];
        last = "`";
      } else if (c === "/" && (last === "" || /[(,=:[!&|?{};+\-*%>~^]/.test(last) || BEFORE_REGEX.has(last))) {
        let i = at + 1;
        let inClass = false;
        while (i < text.length && text[i] !== "\n" && (inClass || text[i] !== "/")) {
          if (text[i] === "\\") i++;
          else if (text[i] === "[") inClass = true;
          else if (text[i] === "]") inClass = false;
          i++;
        }
        if (text[i] === "/") {
          at = i + 1;
          while (at < text.length && /[a-z]/.test(text[at]!)) at++;
          out += " ";
          last = ")";
        } else {
          out += c;
          at++;
          last = c;
        }
      } else {
        if (nested && c === "{") depth++;
        if (nested && c === "}" && depth-- === 0) {
          out += c;
          at++;
          return;
        }
        out += c;
        at++;
        // The last token: a word (a keyword, or an operand), or one punctuation character.
        if (/[A-Za-z0-9_$]/.test(c)) last = /[A-Za-z0-9_$]/.test(text[at - 2] ?? "") ? last + c : c;
        else if (!/\s/.test(c)) last = c;
      }
    }
  };
  code(false);
  return out;
}

/** Procedural modelling calls on `og.render` that live in the `shape` split module. */
const SHAPE_RENDER_CALL = /\bog\.render\.(?:(?:shape|extrude|lathe|polyhedron|icosahedron|dodecahedron|octahedron|tetrahedron|sweep|csg)Geometry|mergeGeometries)\b/;

/**
 * The split modules a project references (og-build.json `modules`): those
 * its boot content needs as the cook recorded them ({@link cookedModules}),
 * and those whose IDL module's members its code names (`og.RigidBody`,
 * `og.physics.castRay`; bindings.json `splits` and each record's `split`),
 * resolving `physics` to the simulation of the project's profile and
 * `destruction` to `destruction2d` or `destruction3d` by the calls made (a
 * 2D call, `og.destruction.carve2d`, is destruction2d's), or, named by
 * neither a call nor a record, by the profile. When `reachedFiles` (the
 * bundler's import graph) is given, only those project files are scanned;
 * otherwise the project's source tree is walked. Comments and regular
 * expressions are left out ({@link stripComments}), so a commented-out
 * reference prefetches nothing.
 */
export function referencedModules(project: string, bindings: Bindings, reachedFiles?: readonly string[]): string[] {
  const records = new Map<string, readonly string[]>();
  for (const r of [...bindings.components, ...bindings.resources, ...bindings.events]) if (r.split) records.set(r.name, r.split);
  const cooked = join(project, "cooked");
  const found = new Set<string>(cookedModules(cooked));
  const scan = (path: string) => {
    const text = stripComments(readFileSync(path, "utf8"));
    for (const m of text.matchAll(/\bog\.([a-z][A-Za-z0-9]*)\.([A-Za-z0-9_]*)/g)) {
      for (const o of bindings.splits[m[1]!] ?? []) found.add(o);
      // og.destruction's 2D calls (carve2d, cell2d, …) are destruction2d's, the rest destruction3d's.
      if (m[1] === "destruction") found.add(m[2]!.endsWith("2d") ? "destruction2d" : "destruction3d");
    }
    for (const m of text.matchAll(/\bog\.([A-Z][A-Za-z0-9]*)\b/g)) for (const o of records.get(m[1]!) ?? []) found.add(o);
    if (SHAPE_RENDER_CALL.test(text)) found.add("shape");
  };
  if (reachedFiles) {
    const prefix = project.endsWith("/") || project.endsWith("\\") ? project : `${project}/`;
    const prefixWin = `${project}\\`;
    for (const path of reachedFiles) {
      if (!(path.startsWith(prefix) || path.startsWith(prefixWin))) continue;
      if (!SOURCES.test(path) || !existsSync(path)) continue;
      const parts = path.slice(prefix.length).split(/[\\/]/);
      if (parts.some((p) => SKIP.has(p))) continue;
      scan(path);
    }
  } else {
    const walk = (dir: string) => {
      for (const e of readdirSync(dir, { withFileTypes: true })) {
        if (SKIP.has(e.name)) continue;
        const path = join(dir, e.name);
        if (e.isDirectory()) walk(path);
        else if (e.isFile() && SOURCES.test(e.name)) scan(path);
      }
    };
    walk(project);
  }
  const is2d = is2dProfile(cooked);
  if (found.has("physics") && !found.has("physics2d") && !found.has("physics3d")) found.add(is2d ? "physics2d" : "physics3d");
  if (found.has("destruction")) {
    found.delete("destruction");
    if (!found.has("destruction2d") && !found.has("destruction3d")) found.add(is2d ? "destruction2d" : "destruction3d");
  }
  return [...found].sort();
}
