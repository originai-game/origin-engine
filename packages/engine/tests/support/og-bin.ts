// The `og` binary a bun test runs (AGENTS.md §Build and gate discipline):
// $OG_BIN, else the one `scripts/build.sh` put in native/ for this platform.
// Tests never run `cargo`: under the gate build.sh has already run, and a
// standalone `bun test` says what to do when it has not.
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function ogBin(): string {
  const env = process.env.OG_BIN;
  if (env) return env;
  const platform = `${process.platform}-${process.arch}`;
  const bin = join(pkg, "native", `og.${platform}`);
  if (existsSync(bin)) return bin;
  throw new Error(`no og binary at ${bin}: run \`bun run build\` in packages/engine (or set OG_BIN)`);
}

/**
 * Cooks `project` (a path under `cwd`, the repository root) with that og
 * before a run reads its `cooked/`, as `og test` does. The cook is
 * incremental, about 0.1 s for a project that is up to date, and it redoes
 * what an older og cooked: a blob whose type hash moved with a format
 * change is cooked again, never reused. A run that only cooked when
 * `cooked/` was missing started on a stale cook after every merge that
 * moved a format (debt round, 2026-10-24: Ember Vale's strider.glb).
 */
export function cookProject(project: string, cwd: string): void {
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
