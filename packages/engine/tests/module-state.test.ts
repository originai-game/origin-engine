// The dev check for state outside the world (docs/design.md §2.4,
// src/node/module-state.ts): a dev run's warnings name each module-scope
// variable the game writes, and leave alone what it only reads, a local of
// the same shape and the text of a string or comment.
import { expect, test } from "bun:test";
import { cpSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject, ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

test("a dev run names the module-scope variables the game writes, by file, line and name", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-module-state-"));
  try {
    cpSync(join(repo, "packages", "create", "template"), dir, { recursive: true });
    writeFileSync(join(dir, "src", "main.ts"), [
      `import { og, defineSystem } from "@origingame/engine";`,
      `let ticks = 0;`,
      `const seen = new Set<number>();`,
      `const LIMIT = 3, NAMES = ["a", "b"];`,
      `// let fake = 0; fake = 1;`,
      `const note = "let quoted = 1; quoted++";`,
      `const count = defineSystem("Count", {}, (_q, time) => {`,
      `  let local = 0;`,
      `  local += LIMIT + NAMES.length + note.length;`,
      `  ticks++;`,
      `  seen.add(time.tick + local);`,
      `});`,
      `export default og.game({ systems: [count] });`,
      ``,
    ].join("\n"));
    cookProject(dir, repo);
    const r = Bun.spawnSync([ogBin(), "run", "--headless", dir, "--no-check", "--commands", "0.1:quit", "--json"], { cwd: repo });
    const out = new TextDecoder().decode(r.stdout);
    if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr) + out);
    const { warnings } = JSON.parse(out.trim().split("\n").pop()!) as { warnings: string[] };
    const found = warnings.filter((w) => w.includes("module-scope")).map((w) => /(src\/main\.ts:\d+): module-scope (\w+) '(\w+)'/.exec(w)?.slice(1).join(" "));
    expect(found).toEqual(["src/main.ts:2 let ticks", "src/main.ts:3 const seen"]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);
