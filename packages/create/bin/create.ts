#!/usr/bin/env bun
// `npm create @origingame/game` (and `bun create`): scaffolds the Origin
// Engine project template (templates/game) into a directory, naming the game.
//
//   create [dir] [--name "Name"] [--id slug] [--archetype runner|match3|...] [--force]
//
// The template comes from ./template/ in the published package (prepack:
// scripts/pack-template.ts) or ../../templates/game in the engine repository;
// an archetype (a complete small game to start from) from ./archetypes/<name>/
// or ../../templates/archetypes/<name>/.

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Directories of a template copy that are build output, never scaffolded. */
export const SKIPPED = new Set(["cooked", "dist", "node_modules", "og-out", ".og"]);

/**
 * Template files stored under another name and renamed in the scaffold: the
 * template's Git attributes route media through LFS in a game's repository,
 * but as `.gitattributes` they would do the same in the engine repository,
 * which stores the template's media as plain blobs.
 */
export const RENAMED: ReadonlyMap<string, string> = new Map([["gitattributes", ".gitattributes"]]);

export interface CreateOptions {
  readonly dir: string;
  readonly name?: string;
  readonly id?: string;
  readonly force?: boolean;
  /** Start from an archetype (`runner`, `match3`, `topdown`, `tower-defense`, `roguelike`, ...: templates/archetypes/README.md) instead of the project template. */
  readonly archetype?: string;
  /** The template directory; found next to this file when omitted. */
  readonly template?: string;
}

export interface Created {
  readonly dir: string;
  readonly name: string;
  readonly id: string;
  readonly files: readonly string[];
}

/** `My Game!` -> `my-game`. */
export function slug(name: string): string {
  return name
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/** `my-game` -> `My Game`. */
function title(id: string): string {
  return id
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((w) => w[0]!.toUpperCase() + w.slice(1))
    .join(" ");
}

/** The template this copy of the package scaffolds. */
export function templateDir(archetype?: string): string {
  const here = dirname(fileURLToPath(import.meta.url));
  if (archetype !== undefined) {
    const roots = [join(here, "..", "archetypes"), join(here, "..", "..", "..", "templates", "archetypes")];
    for (const root of roots) {
      if (existsSync(join(root, archetype, "project.ogt"))) return join(root, archetype);
    }
    const known = roots.filter(existsSync).flatMap((r) => readdirSync(r).filter((d) => existsSync(join(r, d, "project.ogt"))));
    throw new Error(`no archetype '${archetype}': choose one of ${[...new Set(known)].sort().join(", ") || "(none shipped)"}`);
  }
  for (const candidate of [join(here, "..", "template"), join(here, "..", "..", "..", "templates", "game")]) {
    if (existsSync(join(candidate, "project.ogt"))) return candidate;
  }
  throw new Error(`no project template next to ${here}: expected ../template (published package) or ../../../templates/game (engine repository)`);
}

function ogtString(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n")}"`;
}

/** Replace the value of a top-level `key "..."` line of an .ogt document. */
function setOgt(text: string, key: string, value: string): string {
  const line = new RegExp(`^${key}[ \\t]+"(?:[^"\\\\]|\\\\.)*"[ \\t]*$`, "m");
  if (!line.test(text)) throw new Error(`the template's project.ogt has no \`${key} "..."\` line`);
  return text.replace(line, `${key} ${ogtString(value)}`);
}

/** Every file under `root`, relative, skipping build output. */
function files(root: string, at = root): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(at).sort()) {
    const path = join(at, entry);
    if (statSync(path).isDirectory()) {
      if (SKIPPED.has(entry) && at === root) continue;
      out.push(...files(root, path));
    } else out.push(relative(root, path));
  }
  return out;
}

/** Scaffold the template into `options.dir`. Throws with the fix on refusal. */
export function create(options: CreateOptions): Created {
  const dir = resolve(options.dir);
  const id = options.id ?? slug(options.name ?? basename(dir));
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id)) {
    throw new Error(`'${id}' is not a game id: use lowercase letters, digits and single hyphens, as \`--id my-game\``);
  }
  const name = options.name ?? title(id);
  if (existsSync(dir)) {
    if (!statSync(dir).isDirectory()) throw new Error(`${dir} is a file; pick another directory`);
    if (readdirSync(dir).length > 0 && !options.force) {
      throw new Error(`${dir} is not empty; pick an empty directory, or pass --force to write the template over it`);
    }
  }
  const template = options.template ?? templateDir(options.archetype);
  const list = files(template);
  const written: string[] = [];
  mkdirSync(dir, { recursive: true });
  for (const file of list) {
    const to = RENAMED.get(file) ?? file;
    mkdirSync(dirname(join(dir, to)), { recursive: true });
    cpSync(join(template, file), join(dir, to));
    written.push(to);
  }
  const manifest = join(dir, "project.ogt");
  writeFileSync(manifest, setOgt(setOgt(readFileSync(manifest, "utf8"), "name", name), "id", id));
  const pkgPath = join(dir, "package.json");
  if (existsSync(pkgPath)) {
    const pkg = JSON.parse(readFileSync(pkgPath, "utf8")) as Record<string, unknown>;
    pkg.name = id;
    writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
  }
  return { dir, name, id, files: written };
}

const USAGE = `usage: create [dir] [--name "Name"] [--id slug] [--archetype name] [--force]

Scaffolds an Origin Engine game into dir (default: ./origin-game).
  --name   the game's display name (default: from the id or the directory)
  --id     the game's id, lowercase-hyphen (default: from the name)
  --archetype  start from a complete archetype game (runner, match3, topdown, tower-defense, roguelike) instead of the template
  --force  write into a directory that is not empty`;

function main(argv: string[]): number {
  let dir: string | undefined;
  let name: string | undefined;
  let id: string | undefined;
  let force = false;
  let archetype: string | undefined;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    const value = () => {
      const v = argv[++i];
      if (v === undefined) throw new Error(`${a} needs a value. ${USAGE}`);
      return v;
    };
    if (a === "--name") name = value();
    else if (a === "--id") id = value();
    else if (a === "--force") force = true;
    else if (a === "--archetype") archetype = value();
    else if (a === "-h" || a === "--help") {
      console.log(USAGE);
      return 0;
    } else if (a.startsWith("-")) throw new Error(`unknown option ${a}. ${USAGE}`);
    else if (dir === undefined) dir = a;
    else throw new Error(`one directory only, got '${dir}' and '${a}'. ${USAGE}`);
  }
  const made = create({ dir: dir ?? (id ?? (name ? slug(name) : "origin-game")), ...(name ? { name } : {}), ...(id ? { id } : {}), ...(archetype ? { archetype } : {}), force });
  const shown = relative(process.cwd(), made.dir) || ".";
  console.log(`Created ${made.name} (${made.id}) in ${shown}: ${made.files.length} files.

Next:
  cd ${shown}
  bun install
  og cook
  og run .`);
  return 0;
}

if (import.meta.main) {
  try {
    process.exit(main(process.argv.slice(2)));
  } catch (e) {
    console.error(`create: ${e instanceof Error ? e.message : String(e)}`);
    process.exit(1);
  }
}
