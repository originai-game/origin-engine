// The check for state outside the world (docs/design.md §2.4,
// crates/og-cli/src/module_state.rs): og check fails on each module-scope
// variable the game writes from a function, by file, line and name, and
// leaves alone what it only reads, a local of the same name and the text
// of a string or comment; og run stops on it as on a type error.
import { expect, test } from "bun:test";
import { cpSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject, ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

test("og check fails on the module-scope variables the game writes, by file, line and name, and og run stops", () => {
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
    const check = Bun.spawnSync([ogBin(), "check", dir, "--json"], { cwd: repo });
    expect(check.exitCode).not.toBe(0);
    const event = JSON.parse(new TextDecoder().decode(check.stdout).trim().split("\n")[0]!) as { ok: boolean; diagnostics: { file: string; line: number; code: number; message: string }[] };
    const found = event.diagnostics.filter((d) => d.code === 9101).map((d) => `${d.file}:${d.line} ${/module-scope (\w+) '(\w+)'/.exec(d.message)?.slice(1).join(" ")}`);
    expect(found).toEqual(["src/main.ts:10 let ticks", "src/main.ts:11 const seen"]);
    cookProject(dir, repo);
    const run = Bun.spawnSync([ogBin(), "run", "--headless", dir, "--commands", "0.1:quit"], { cwd: repo });
    expect(run.exitCode).not.toBe(0);
    expect(new TextDecoder().decode(run.stderr)).toContain("OG9101");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);
