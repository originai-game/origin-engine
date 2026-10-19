// UI sounds start in the tick of the input that caused them (og-runtime's
// og.ui.sound runs before og.audio): `og run --audio` on the UI kit renders
// the mix to a WAV, and each blip's first sample lies inside its input's
// tick. The WAV holds every tick of the run, tick k (counting from 1) as
// frames (k - 1)·800 .. k·800 at 48 kHz and 60 Hz; a `dump_pulse` beside an
// input reports the ticks done before it, so the input is read by tick
// T + 1, frames T·800 .. (T + 1)·800.
//
// Ways it could fail: the sound starts a tick late (og.ui.sound after
// og.audio: frame (T + 1)·800, as before phase 2), the UI never queues it,
// or the blip starts at a frame that depends on anything but the tick.
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
const kit = join(repo, "templates", "ui-kit");
const work = join(repo, "target", "evidence", "ui-sound");

/** The first frame of each sound after at least 400 silent frames. */
function onsets(wav: Uint8Array): { rate: number; frames: number; at: number[] } {
  const view = new DataView(wav.buffer, wav.byteOffset, wav.byteLength);
  const frames = (wav.length - 44) / 4;
  const at: number[] = [];
  let silent = Infinity;
  for (let i = 0; i < frames; i++) {
    const loud = view.getInt16(44 + 4 * i, true) !== 0 || view.getInt16(46 + 4 * i, true) !== 0;
    if (loud && silent >= 400) at.push(i);
    silent = loud ? 0 : silent + 1;
  }
  return { rate: view.getUint32(24, true), frames, at };
}

test("a UI sound's first sample lies in the tick of its input", () => {
  rmSync(work, { recursive: true, force: true });
  mkdirSync(work, { recursive: true });
  const script = [
    "0.5:show:mainMenu",
    // The pointer reaches Play (hover), then presses it (press).
    "1.5:move_to_text:=Play",
    "1.5:dump_pulse:hover.json",
    "2.5:touch_start",
    "2.5:dump_pulse:press.json",
    "2.6:touch_end",
    "3.2:quit",
  ].join(";");
  const wav = join(work, "mix.wav");
  const r = Bun.spawnSync([ogBin(), "run", kit, "--commands", script, "--audio", wav, "--out", work, "--json"], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(out + new TextDecoder().decode(r.stderr));
  const summary = JSON.parse(out.trim().split("\n").pop()!) as { ticks: number };
  const mix = onsets(readFileSync(wav));
  expect(mix.rate).toBe(48000);
  const perTick = mix.rate / 60;
  // The WAV is every tick of the run, from the first.
  expect(mix.frames).toBe(summary.ticks * perTick);
  const tickOf = (f: string) => (JSON.parse(readFileSync(join(work, f), "utf8")) as { tick: number }).tick;
  const inputs = [tickOf("hover.json"), tickOf("press.json")];
  expect(mix.at.length).toBe(inputs.length);
  inputs.forEach((t, k) => {
    const first = mix.at[k]!;
    expect(first).toBeGreaterThan(t * perTick - 1);
    expect(first).toBeLessThan((t + 1) * perTick);
  });
}, 120_000);
