// The knowledge package as the platform receives it: `payload()` maps the
// package (and, in the engine repository, templates/game) onto the
// platform's production layout (`game/…`, `game-scaffold/…`) through
// knowledge.json, and the helpers read what the platform's own tooling
// reads (SKILL.md front matter, pack names and hashes).

import { createHash } from "node:crypto";
import { existsSync, lstatSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

export const PACKAGE_ROOT = join(import.meta.dir, "..");

export interface Roots {
  /** packages/knowledge (or a copy of it). */
  readonly pkg: string;
  /** The project template the scaffold overlay carries: templates/game in the repository, scaffold/template/ in the published package. */
  readonly template: string;
  /** packages/engine, whose version the scaffold pins. */
  readonly engine: string;
}

/** Whether `dir` is the published @origingame/engine package (the knowledge ships inside it as knowledge/). */
function isEnginePackage(dir: string): boolean {
  try {
    return (JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as { name?: string }).name === "@origingame/engine";
  } catch {
    return false;
  }
}

/** The roots in the engine repository (packages/knowledge beside packages/engine), or in the published @origingame/engine, whose knowledge/ this is. */
export function defaultRoots(pkg = PACKAGE_ROOT): Roots {
  const repoTemplate = join(pkg, "..", "..", "templates", "game");
  const parent = join(pkg, "..");
  return {
    pkg,
    template: existsSync(join(repoTemplate, "project.ogt")) ? repoTemplate : join(pkg, "scaffold", "template"),
    engine: isEnginePackage(parent) ? parent : join(parent, "engine"),
  };
}

export interface Mount {
  /** A package path (`skills/`, `wiki/`), or `@template/` for the project template. */
  readonly from: string;
  /** Its place in the platform's production tree. */
  readonly to: string;
}

export interface Manifest {
  readonly schemaVersion: 1;
  readonly overlay: string;
  readonly mounts: readonly Mount[];
}

export function readManifest(pkg: string): Manifest {
  return JSON.parse(readFileSync(join(pkg, "knowledge.json"), "utf8")) as Manifest;
}

/** One file of the payload: its production path, where it came from, its bytes. */
export interface PayloadFile {
  readonly path: string;
  readonly source: string;
  readonly bytes: Uint8Array;
  readonly symlink: boolean;
}

/** Files under `dir`, sorted, skipping build output a template never carries. */
export function walk(dir: string, skip: ReadonlySet<string> = new Set()): string[] {
  const out: string[] = [];
  const visit = (d: string) => {
    for (const name of readdirSync(d).sort()) {
      const full = join(d, name);
      const rel = relative(dir, full);
      if (skip.has(name) || skip.has(rel)) continue;
      const st = lstatSync(full);
      if (st.isDirectory()) visit(full);
      else out.push(full);
    }
  };
  if (existsSync(dir)) visit(dir);
  return out;
}

/** What never travels with a template: build and run output (the template's .gitignore) and installs. */
export const TEMPLATE_SKIP: ReadonlySet<string> = new Set(["cooked", "dist", "og-out", ".og", "node_modules", ".DS_Store"]);

/**
 * Template files stored under another name than the scaffold gives them
 * (packages/create's RENAMED): the template's `gitattributes` lands in the
 * overlay as `.gitattributes`, which in the engine repository would route
 * the template's plain-blob media through LFS.
 */
export const TEMPLATE_RENAMED: ReadonlyMap<string, string> = new Map([["gitattributes", ".gitattributes"]]);

const fatal = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });

/** Whether bytes are text the platform's payload can carry (UTF-8, no NUL). */
export function isText(bytes: Uint8Array): boolean {
  if (bytes.includes(0)) return false;
  try {
    fatal.decode(bytes);
    return true;
  } catch {
    return false;
  }
}

/**
 * The production-tree payload: every mounted file. Template binaries are
 * included here (so the validator sees them) and flagged by `isText`; the
 * published bundle carries text only and names binaries in distribution.json.
 */
export function payload(roots: Roots): PayloadFile[] {
  const manifest = readManifest(roots.pkg);
  const files: PayloadFile[] = [];
  for (const m of manifest.mounts) {
    const base = m.from === "@template/" ? roots.template : join(roots.pkg, m.from);
    const skip = m.from === "@template/" ? TEMPLATE_SKIP : new Set<string>();
    const sources = m.from.endsWith("/") ? walk(base, skip) : [base];
    for (const source of sources) {
      const found = m.from.endsWith("/") ? relative(base, source).split("\\").join("/") : "";
      const rel = m.from === "@template/" ? (TEMPLATE_RENAMED.get(found) ?? found) : found;
      const path = m.to.endsWith("/") ? m.to + rel : m.to;
      const symlink = existsSync(source) ? lstatSync(source).isSymbolicLink() : false;
      files.push({ path, source, bytes: existsSync(source) && !symlink ? new Uint8Array(readFileSync(source)) : new Uint8Array(), symlink });
    }
  }
  return files.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

export function sha256(bytes: Uint8Array | string): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/** Pack names as agent-server/src/core/knowledge.ts `packsFromFiles` groups production paths. */
export function packName(path: string): string {
  const parts = path.split("/");
  if (path === "game/instructions.md") return "instructions";
  if (parts[0] === "game" && parts[1] === "skills" && parts[2]) return `skill:${parts[2]}`;
  if (parts[0] === "game-scaffold" && parts[1]) return `scaffold:${parts[1]}`;
  return "game";
}

/** Each pack's content hash, as `packsFromFiles` computes it over the text files. */
export function packs(files: readonly PayloadFile[]): Record<string, string> {
  const groups = new Map<string, Record<string, string>>();
  for (const f of files) {
    if (!isText(f.bytes)) continue;
    const name = packName(f.path);
    const g = groups.get(name) ?? {};
    g[f.path] = fatal.decode(f.bytes);
    groups.set(name, g);
  }
  return Object.fromEntries(
    [...groups.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([name, g]) => [name, sha256(Object.keys(g).sort().map((p) => `${p}\0${sha256(g[p]!)}`).join("\n")).slice(0, 16)]),
  );
}

/** A SKILL.md's front matter as `key: value` pairs, or null without the `---` fences. */
export function frontMatter(text: string): Record<string, string> | null {
  const lines = text.replace(/^\uFEFF/, "").split(/\r?\n/);
  if (lines[0]?.trim() !== "---") return null;
  const end = lines.findIndex((l, i) => i > 0 && l.trim() === "---");
  if (end < 0) return null;
  const out: Record<string, string> = {};
  let key = "";
  for (const line of lines.slice(1, end)) {
    const m = /^([A-Za-z][\w-]*):(.*)$/.exec(line);
    if (m) {
      key = m[1]!;
      const rest = m[2]!.trim();
      out[key] = ["", ">", "|", ">-", "|-"].includes(rest) ? "" : rest.replace(/^["']|["']$/g, "");
    } else if (key && /^[ \t]/.test(line)) {
      out[key] = out[key] ? `${out[key]} ${line.trim()}` : line.trim();
    }
  }
  return out;
}

/** The top-level `key value` entries of an `.ogt` document (values as written, quotes removed). */
export function ogtTopLevel(text: string): { type: string; entries: Map<string, string> } {
  const lines = text.split(/\r?\n/);
  const sep = lines.findIndex((l) => l.trim() === "---");
  const header = lines.slice(0, sep < 0 ? 0 : sep).map((l) => l.trim()).filter((l) => l && !l.startsWith("#"));
  const type = header.find((l) => l.startsWith("type "))?.slice(5).trim() ?? "";
  const entries = new Map<string, string>();
  let depth = 0;
  for (const raw of lines.slice(sep + 1)) {
    const line = raw.replace(/(^|\s)#.*$/, "").trim();
    if (!line) continue;
    if (line === "}") {
      depth--;
      continue;
    }
    const [key, ...rest] = line.split(/\s+/);
    const opens = line.endsWith("{");
    if (depth === 0) entries.set(key!, (opens ? rest.slice(0, -1) : rest).join(" ").replace(/^"|"$/g, ""));
    if (opens) depth++;
  }
  return { type, entries };
}
