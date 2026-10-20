// A long sound cooked in chunks (og-cook's `sound { stream true }`) through
// `og run` on the native host: the idle archetype's music (orbit.wav, a
// 26.7 s loop) cooked whole and cooked in 2-second chunks, each run for
// 35 s so the loop wraps, renders the same WAV to the sample (`--audio`'s
// SHA-256), and the chunked cook ships the chunks as files of their own.
import { expect, test } from "bun:test";
import { cpSync, mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const template = join(repo, "templates", "archetypes", "idle");
const work = join(repo, "target", "evidence", "sound-chunks");

function stage(name: string, settings: string | null): string {
  const dir = join(work, name);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  cpSync(template, dir, { recursive: true });
  for (const d of ["cooked", "node_modules", "out"]) rmSync(join(dir, d), { recursive: true, force: true });
  if (settings !== null) writeFileSync(join(dir, "assets", "audio", "orbit.wav.ogt"), settings);
  return dir;
}

function run(project: string): string {
  const r = Bun.spawnSync([ogBin(), "run", project, "--commands", "35:quit", "--audio", join(project, "run.wav"), "--out", join(project, "out"), "--json"], {
    cwd: repo,
    env: { ...process.env, OG_ENGINE_PACKAGE: pkg },
  });
  const text = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(text + new TextDecoder().decode(r.stderr));
  const summary = JSON.parse(text.trim().split("\n").pop()!) as { audio: { mode: string; wavSha256: string } };
  expect(summary.audio.mode).toBe("capture");
  return summary.audio.wavSha256;
}

test("music cooked in chunks plays sample-identical to the whole file through og run", () => {
  const whole = stage("whole", null);
  const chunked = stage("chunked", "type import\nversion 1\n---\nsound {\n  stream true\n  chunk 2\n}\n");
  const a = run(whole);
  const b = run(chunked);
  expect(a).toMatch(/^[0-9a-f]{64}$/);
  expect(b).toBe(a);
  const files = (dir: string) => readdirSync(join(dir, "cooked", "audio")).filter((f) => f.startsWith("orbit.wav"));
  expect(files(whole)).toEqual(["orbit.wav.ogb"]);
  // 26.7 s at 22,050 Hz is 115 QOA frames; 2 s rounds up to 9 a chunk: 13 chunks, 12 files beside the head.
  const chunks = files(chunked).filter((f) => /\.chunk\d+\.ogb$/.test(f));
  expect(chunks.length).toBe(12);
}, 240_000);
