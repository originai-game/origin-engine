// A session freezes a run's project by cloning it: clones cost no space, so
// a project past 512 MiB of files (sources plus their cooked bundle) runs.
import { expect, test } from "bun:test";
import { constants, copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, truncateSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { freezeInputs } from "../src/node/session-inputs.ts";

function clones(dir: string): boolean {
  writeFileSync(join(dir, "probe"), "x");
  try {
    copyFileSync(join(dir, "probe"), join(dir, "probe-clone"), constants.COPYFILE_FICLONE_FORCE);
    return true;
  } catch {
    return false;
  } finally {
    rmSync(join(dir, "probe"), { force: true });
    rmSync(join(dir, "probe-clone"), { force: true });
  }
}

const dir = mkdtempSync(join(tmpdir(), "og-session-inputs-"));
test.skipIf(!clones(dir))("a cloned project past 512 MiB is admitted", () => {
  try {
    const project = join(dir, "project");
    mkdirSync(join(project, "cooked"), { recursive: true });
    const metadata = '{"version":1,"name":"Clone","engine":"0.9.0"}\n';
    writeFileSync(join(project, "og-project.json"), metadata);
    writeFileSync(join(project, "project.ts"), 'import {defineProject} from "@origingame/engine/author"; export default defineProject({id:"clone",entry:"src/main.ts"});\n');
    // Sparse: 600 MiB long, no blocks written.
    writeFileSync(join(project, "cooked", "big.ogb"), "");
    truncateSync(join(project, "cooked", "big.ogb"), 600 * 1024 * 1024);
    const frozen = freezeInputs(["--root", project], dir, join(dir, "runs"));
    try {
      // Beside the project, on its volume, whatever volume the runs directory is on.
      expect(frozen.inputRoot!.startsWith(join(project, ".og", "session-inputs"))).toBe(true);
      expect(readFileSync(join(frozen.inputRoot!, "og-project.json"), "utf8")).toBe(metadata);
    } finally {
      frozen.dispose();
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
