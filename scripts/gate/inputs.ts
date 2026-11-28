/** Cargo dependency closures and conservative content-snapshot selection. */
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";
import { execFileSync } from "node:child_process";

export const ROOT = process.cwd();
export const LOGS = process.env.GATE_LOGS!;
export const GREEN = join(process.env.OG_TARGET_DIR!, "og-gate/green");
const TREE = process.env.OG_GATE_TREE ?? "";
export interface Metadata {
  packages: { id: string; name: string; manifest_path: string; targets: { kind: string[] }[] }[];
  workspace_members: string[];
  resolve: { nodes: { id: string; dependencies: string[] }[] };
}
let meta: Metadata;
export function metadata(): Metadata {
  return meta ??= JSON.parse(readFileSync(join(LOGS, "cargo.json"), "utf8"));
}

// Cargo gives CARGO_BIN_EXE_og to og-cli alone; og loads the built addon.
const RUNS: Record<string, string[]> = { "og-cli": ["og-host-node"] };
const reads = new Map<string, Set<string>>();
function pathReads(directory: string): Set<string> {
  if (reads.has(directory)) return reads.get(directory)!;
  const found = new Set<string>();
  const root = join(ROOT, directory);
  for (const path of new Bun.Glob("**/*.rs").scanSync({ cwd: root, dot: true, onlyFiles: false, followSymlinks: false })) {
    if (!statSync(join(root, path)).isFile()) continue;
    const source = readFileSync(join(root, path), "utf8");
    for (const match of source.matchAll(/(?:\.\.\/|crates\/)(og-[a-z0-9-]+(?:\/[A-Za-z0-9_.-]+)*)/g)) {
      if (existsSync(join(ROOT, "crates", match[1]))) found.add("crates/" + match[1]);
    }
  }
  reads.set(directory, found);
  return found;
}

export function crateInputs(names: string[]): Set<string> {
  const meta = metadata();
  const packages = new Map(meta.packages.map(p => [p.id, p]));
  const nodes = new Map(meta.resolve.nodes.map(n => [n.id, n.dependencies]));
  const ids = new Map(meta.packages.filter(p => meta.workspace_members.includes(p.id)).map(p => [p.name, p.id]));
  const pending = meta.packages.filter(p => names.includes(p.name)).map(p => p.id);
  if (pending.length < new Set(names).size) return new Set(["."]);
  const seen = new Set<string>(), paths = new Set<string>();
  while (pending.length) {
    const id = pending.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    const pkg = packages.get(id)!;
    pending.push(...nodes.get(id)!, ...(RUNS[pkg.name] ?? []).map(n => ids.get(n)!));
    const directory = relative(ROOT, dirname(pkg.manifest_path));
    if (directory !== ".." && !directory.startsWith("../") && !directory.startsWith("..\\") && !isAbsolute(directory)) {
      paths.add(directory || ".");
      for (const path of pathReads(directory)) paths.add(path);
    }
  }
  return paths;
}

function changed(key: string): Set<string> | null {
  try {
    if (process.env.FORCE_ALL === "1" || !TREE) return null;
    const base = readFileSync(join(GREEN, key), "utf8").trim();
    return new Set(execFileSync("git", ["diff", "--no-renames", "--name-only", "-z", base, TREE], { encoding: "utf8" }).split("\0").filter(Boolean));
  } catch { return null; }
}
function within(file: string, paths: Set<string>): boolean {
  return [...paths].some(p => p === "." || file === p || file.startsWith(p + "/"));
}
function inert(file: string): boolean {
  const parts = file.split("/");
  if (["AGENTS.md", "README.md"].includes(file)) return true;
  if (["docs", "tasks"].includes(parts[0])) return file.endsWith(".md");
  if (parts[0] === "changelog" && parts[1] === "unreleased") return true;
  if (parts.length >= 4 && parts[0] === "packages" && parts[2] === "tests") {
    return parts[3] === "support" || (parts.length === 4 && file.endsWith(".test.ts"));
  }
  return false;
}
export function unaffected(key: string, paths: Set<string>): boolean {
  const delta = changed(key);
  if (delta === null) return false;
  if (paths.has(".")) return delta.size === 0;
  // Only existing Rust sources have proven reachability through Cargo.
  const meta = metadata();
  const known = crateInputs(meta.packages.filter(p => meta.workspace_members.includes(p.id)).map(p => p.name));
  return [...delta].every(f => inert(f) || (f.endsWith(".rs") && existsSync(f) && statSync(f).isFile() && within(f, known) && !within(f, paths)));
}
export function mark(key: string): void {
  if (!TREE) return;
  mkdirSync(GREEN, { recursive: true });
  writeFileSync(join(GREEN, key), TREE + "\n");
}
if (import.meta.main) {
  const [action, ...args] = process.argv.slice(2);
  if (action === "crates") console.log([...crateInputs(args)].sort().join(" "));
  else if (action === "fixture") {
    const paths = crateInputs(["og-cli", "og-host-node"]);
    for (const p of [args[0], "packages/engine", "packages/create"]) paths.add(p);
    const inputs = join(args[0], "inputs");
    if (existsSync(inputs)) for (const line of readFileSync(inputs, "utf8").split(/\r?\n/)) {
      const path = line.split("#", 1)[0].trim();
      if (path) paths.add(path);
    }
    console.log([...paths].sort().join(" "));
  } else if (action === "unchanged") process.exit(unaffected(args[0], new Set(args.slice(1).join(" ").split(/\s+/).filter(Boolean))) ? 0 : 1);
  else throw new Error(action);
}
