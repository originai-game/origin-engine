// Host tools consume the Rust bootstrap reader. Never import/eval project.ts
// in Bun, parse it with regexes, or cache a snapshot across commands/edits.
import { existsSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import type { ProjectInput } from "../../types/author-inputs.d.ts";
import type { ProjectStartup } from "../../types/index.d.ts";
import { METADATA } from "./project-metadata.ts";
import type { PublishSettings } from "./web-build.ts";

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
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr.trim() || `og project exited ${result.status}`);
  return JSON.parse(result.stdout) as CheckedProject;
}
