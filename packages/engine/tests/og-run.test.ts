// `og run --headless` loads the hello world on the native host and prints
// the same world hash the web host computes.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

const run = (cmd: string[], cwd: string) => {
  const r = Bun.spawnSync(cmd, { cwd });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${cmd.join(" ")}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!);
};

test("og run --headless prints the hello world's hash after 600 ticks", () => {
  const og = run(
    [ogBin(), "run", "--headless", "packages/engine/examples/hello.ts", "--ticks", "600", "--setup", "spawn_wave:10000", "--json"],
    repo,
  );
  const web = run(["bun", "tests/support/hello-hash.ts", "--host", "web"], pkg);
  expect(og.tick).toBe(600);
  expect(og.entities).toBe(10000);
  expect(og.hash).toBe(web.hash);
}, 300_000);
