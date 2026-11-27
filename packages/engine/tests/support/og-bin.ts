// Tests use the host tools' CLI resolution and never run cargo.
import { ogBin } from "../../src/node/project.ts";
export { ogBin };

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
};

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
