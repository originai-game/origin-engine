// Host tools consume the Rust bootstrap reader. Never import/eval project.ts
// in Bun, parse it with regexes, or cache a snapshot across commands/edits.
import { existsSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { ProjectInput } from "../../types/author-inputs.d.ts";
import type { ProjectStartup } from "../../types/index.d.ts";
import { METADATA } from "./project-metadata.ts";
import type { PublishSettings } from "./web-build.ts";

// Knowledge is inside the engine when packed, beside it in a checkout, as
// create/bin/knowledge.ts also resolves it. Never duplicate its stream reader.
const here = dirname(fileURLToPath(import.meta.url));
const reader = [join(here, "../../knowledge/scripts/project-output.ts"), join(here, "../../../knowledge/scripts/project-output.ts")].find((file) => existsSync(file));
if (!reader) throw new Error("no knowledge/scripts/project-output.ts; reinstall the engine distribution");
const { readProjectOutput }: typeof import("../../../knowledge/scripts/project-output.ts") = await import(pathToFileURL(reader).href);
const stringTables: typeof import("../../../knowledge/scripts/string-tables.ts") = await import(pathToFileURL(join(dirname(reader), "string-tables.ts")).href);

/** This engine's CLI, shared by cooking and checked project/content readers. */
export function ogBin(): string {
  if (process.env.OG_BIN) return process.env.OG_BIN;
  const platform = `${process.platform}-${process.arch}`;
  const local = join(here, "..", "..", "native", `og.${platform}`);
  if (existsSync(local)) return local;
  // install.sh's archive has engine/ beside bin/og, not an npm platform
  // package. Resolve that version's executable, never the PATH launcher.
  const installed = join(here, "..", "..", "..", "bin", process.platform === "win32" ? "og.exe" : "og");
  if (existsSync(installed)) return installed;
  try {
    const manifest = createRequire(import.meta.url).resolve(`@origingame/engine-${platform}/package.json`);
    return join(dirname(manifest), process.platform === "win32" ? "og.exe" : "og");
  } catch {
    throw new Error(`no og binary at ${local} or ${installed}: reinstall the engine with its CLI (or set OG_BIN)`);
  }
}

export function readStringTables(root: string, builtins = false, og = ogBin()) {
  return stringTables.readStringTables(root, builtins, og);
}

export function readStringTableIndex(root: string, og = ogBin()) {
  return stringTables.readStringTableIndex(root, og);
}

export interface CheckedProject {
  readonly name: string;
  readonly engine: string;
  readonly id: string;
  readonly input: ProjectInput;
  readonly startup: ProjectStartup;
  readonly publish: Partial<PublishSettings>;
  readonly dependencies: readonly string[];
}

export function projectRoot(from: string): string | undefined {
  let dir = resolve(from);
  if (existsSync(dir) && statSync(dir).isFile()) dir = dirname(dir);
  for (;;) {
    if (existsSync(join(dir, METADATA))) return dir;
    const parent = dirname(dir);
    if (parent === dir) return undefined;
    dir = parent;
  }
}

export function readProject(from: string | undefined): CheckedProject | undefined {
  if (from === undefined) return undefined;
  const root = projectRoot(from);
  if (!root) return undefined;
  const result = spawnSync(ogBin(), ["project", root, "--json"], {encoding: "utf8"});
  return readProjectOutput<CheckedProject>(result);
}
