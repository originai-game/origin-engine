// Host tools consume the Rust bootstrap reader. Never import/eval project.ts
// in Bun, parse it with regexes, or cache a snapshot across commands/edits.
import { existsSync, statSync } from "node:fs";
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
export const { readStringTables, readStringTableIndex }: typeof import("../../../knowledge/scripts/string-tables.ts") = await import(pathToFileURL(join(dirname(reader), "string-tables.ts")).href);

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
  const result = spawnSync(process.env.OG_BIN || "og", ["project", root, "--json"], {encoding: "utf8"});
  return readProjectOutput<CheckedProject>(result);
}
