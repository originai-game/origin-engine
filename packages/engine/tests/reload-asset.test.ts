// M4's edit loop on a model: `og dev` on the scaffolded template, the
// dropship's glTF edited on disk (its hull's base colour turned red), and the
// running game shows the edit after the reload with the Model keeping its
// asset handle. A reload that re-ran only the script would keep drawing the
// model it loaded first. The frames before and after stay in
// target/evidence/reload-asset/ for inspection.
import { expect, test } from "bun:test";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { decodePng } from "../src/node/png.ts";
import { ogBin } from "./support/og-bin.ts";

// The gate's path-aware lane skips this file: it spawns og processes for
// minutes (docs/ci.md §Lanes). `scripts/check.sh --all`, CI and a plain
// `bun run test` run it (OG_GATE_ALL, or no OG_GATE at all).
const SLOW = process.env.OG_GATE === "1" && !process.env.OG_GATE_ALL;

declare const Bun: {
  spawn(
    cmd: string[],
    o: { cwd: string; env?: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe" },
  ): { stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>; kill(): void };
  spawnSync(
    cmd: string[],
    o: { cwd: string; env?: Record<string, string | undefined> },
  ): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  sleep(ms: number): Promise<void>;
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const evidence = join(repo, "target", "evidence", "reload-asset");
const game = join(evidence, "game");
const env = { ...process.env, OG_ENGINE_PACKAGE: pkg };
// The hull's baseColorFactor as the template ships it, and red, the same length.
const GREY = "[0.14000000059604645,0.14499999582767487,0.15000000596046448,1]";
const RED = "[0.94000000059604645,0.04499999582767487,0.05000000596046448,1]";

function swap(file: string, from: string, to: string): void {
  const bytes = readFileSync(file);
  const at = new TextDecoder("latin1").decode(bytes).indexOf(from);
  expect(at).toBeGreaterThan(0);
  bytes.set(new TextEncoder().encode(to), at);
  writeFileSync(file, bytes);
}

/** Pixels that read as the red hull (the grey one has none). */
function red(png: string): number {
  const { rgba } = decodePng(new Uint8Array(readFileSync(png)));
  let n = 0;
  for (let i = 0; i < rgba.length; i += 4) if (rgba[i]! > 120 && rgba[i + 1]! < 60 && rgba[i + 2]! < 60) n++;
  return n;
}

test.skipIf(SLOW)("og dev shows a glTF edit after the reload", async () => {
  rmSync(evidence, { recursive: true, force: true });
  mkdirSync(evidence, { recursive: true });
  const created = Bun.spawnSync(["bun", join(repo, "packages", "create", "bin", "create.ts"), game, "--name", "Reload Game"], { cwd: repo, env });
  expect(created.exitCode).toBe(0);
  const port = String(47_000 + (process.pid % 700));
  const dev = Bun.spawn([ogBin(), "dev", game, "--port", port, "--exit-after", "2", "--json"], { cwd: repo, env, stdout: "pipe", stderr: "pipe" });
  const lines: string[] = [];
  const reader = dev.stdout.getReader();
  const decoder = new TextDecoder();
  let buffered = "";
  const until = async (match: (l: string) => boolean) => {
    for (;;) {
      const hit = lines.find(match);
      if (hit) return hit;
      const { value, done } = await reader.read();
      if (done) throw new Error(`og dev ended first:\n${lines.join("\n")}\n${await new Response(dev.stderr).text()}`);
      buffered += decoder.decode(value, { stream: true });
      const parts = buffered.split("\n");
      buffered = parts.pop()!;
      lines.push(...parts);
    }
  };
  const capture = (name: string) => {
    const r = Bun.spawnSync([ogBin(), "probe", "capture", "--port", port, "--out", join(evidence, name)], { cwd: repo, env });
    expect(r.exitCode).toBe(0);
    return join(evidence, name);
  };
  const reloads = () => lines.filter((l) => l.startsWith("{") && l.includes('"event":"reload"')).length;
  try {
    await until((l) => l.startsWith("probe: ws://"));
    await Bun.sleep(500);
    const before = capture("before.png");
    const glb = join(game, "assets", "models", "dropship.glb");
    swap(glb, GREY, RED);
    await until(() => reloads() >= 1);
    const after = capture("after.png");
    const first = JSON.parse(lines.find((l) => l.includes('"event":"reload"'))!) as { ok: boolean };
    expect(first.ok).toBeTrue();
    expect(red(before)).toBe(0);
    // A hull-sized count, not an exact one: only the hull's pixels that
    // read as plain red count (its lit and shaded sides do not), and their
    // number moves with the template's frame (1,898 since project.ogt's
    // seed applies, 2,1xx before).
    expect(red(after)).toBeGreaterThan(1000);
    // Back to grey: the second reload, after which og dev stops.
    swap(glb, RED, GREY);
    await until(() => reloads() >= 2);
  } finally {
    // og dev stops its game itself after the second reload; on a failure
    // before that, the game it started would outlive it.
    dev.kill();
    await dev.exited;
    Bun.spawnSync(["pkill", "-f", "--", `--port ${port} --realtime`], { cwd: repo });
  }
}, 600_000);
