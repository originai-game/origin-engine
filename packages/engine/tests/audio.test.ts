// Audio through `og run` on the native host: the topdown archetype's bite
// and sword hits rendered by `--audio` into a WAV whose SHA-256 is the same
// for two runs of the script (the mixer's output is a function of the
// command stream), and the `listen` action writing the last second the
// mixer rendered, which holds the hit.
import { expect, test } from "bun:test";
import { mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "archetypes", "topdown");
const work = join(repo, "target", "evidence", "audio-run");

const SCRIPT = [
  "0.1:place:12.3:6.5",
  "0.1:slime:12.8:6.5",
  "0.6:place:12.3:6.5",
  "0.6:slime:13.4:6.5",
  "0.6:face:right",
  "0.65:key_down:Space",
  "0.7:key_up:Space",
  "1.1:listen:hit.wav",
  "1.2:quit",
].join(";");

function run(name: string): { wavSha256: string; out: string } {
  const out = join(work, name);
  const wav = join(work, `${name}.wav`);
  const r = Bun.spawnSync([ogBin(), "run", project, "--commands", SCRIPT, "--audio", wav, "--out", out, "--json"], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const text = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(text + new TextDecoder().decode(r.stderr));
  const summary = JSON.parse(text.trim().split("\n").pop()!) as { audio: { mode: string; wavSha256: string } };
  expect(summary.audio.mode).toBe("capture");
  return { wavSha256: summary.audio.wavSha256, out };
}

function level(bytes: Uint8Array): { rate: number; peak: number; frames: number } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let peak = 0;
  for (let at = 44; at + 1 < bytes.length; at += 2) peak = Math.max(peak, Math.abs(view.getInt16(at, true)) / 32767);
  return { rate: view.getUint32(24, true), peak, frames: (bytes.length - 44) / 4 };
}

test("og run --audio renders the same WAV twice, and listen writes the last second with the hit in it", () => {
  rmSync(work, { recursive: true, force: true });
  mkdirSync(work, { recursive: true });
  const a = run("a");
  const b = run("b");
  expect(a.wavSha256).toMatch(/^[0-9a-f]{64}$/);
  expect(b.wavSha256).toBe(a.wavSha256);
  const whole = level(readFileSync(join(work, "a.wav")));
  expect(whole.rate).toBe(48000);
  expect(whole.frames).toBeGreaterThan(48000);
  expect(whole.peak).toBeGreaterThan(0.05);
  const heard = level(readFileSync(join(a.out, "hit.wav")));
  expect(heard.frames).toBe(48000);
  expect(heard.peak).toBeGreaterThan(0.05);
}, 120_000);
