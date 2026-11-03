// UI sounds start in the tick of the input that caused them (og-runtime's
// og.ui.sound runs before og.audio): `og run --audio` on a game of one
// og.ui.button with hover and press sounds (written here, its blips made
// here) renders the mix to a WAV, and each blip's first sample lies inside
// its input's tick. The WAV holds every tick of the run, tick k (counting from 1) as
// frames (k - 1)·800 .. k·800 at 48 kHz and 60 Hz; a `dump_pulse` beside an
// input reports the ticks done before it, so the input is read by tick
// T + 1, frames T·800 .. (T + 1)·800.
//
// Ways it could fail: the sound starts a tick late (og.ui.sound after
// og.audio: frame (T + 1)·800, as before phase 2), the UI never queues it,
// or the blip starts at a frame that depends on anything but the tick.
import { expect, test } from "bun:test";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const work = join(repo, "target", "evidence", "ui-sound");
const game = join(work, "game");

const PROJECT = `type project
version 1
---
name "UI sound"
id "ui-sound"
entry "src/main.ts"
profile "2d"
tick 60
seed 1
`;

const MAIN = `import { og, defineSystem } from "@origingame/engine";

const Menu = defineSystem("Menu", {}, () => {
  og.ui.button("Play", "Play", { variant: "primary" });
});

export default og.game({
  start(world) {
    const sounds: Record<string, unknown> = {};
    for (const e of ["hover", "press"]) {
      const s = og.load.sound(world, \`audio/ui-\${e}.wav\`);
      if (!s.ok) throw new Error(s.reason);
      sounds[e] = s.value;
    }
    og.ui.configure(world, { sounds: sounds as never });
  },
  systems: [Menu],
});
`;

/** A 40 ms blip: 16-bit mono PCM at 44.1 kHz, every sample loud. */
function blip(hz: number): Uint8Array {
  const n = Math.round(44100 * 0.04);
  const out = new Uint8Array(44 + 2 * n);
  const v = new DataView(out.buffer);
  const tag = (at: number, s: string) => [...s].forEach((c, k) => v.setUint8(at + k, c.charCodeAt(0)));
  tag(0, "RIFF");
  v.setUint32(4, 36 + 2 * n, true);
  tag(8, "WAVEfmt ");
  v.setUint32(16, 16, true);
  v.setUint16(20, 1, true);
  v.setUint16(22, 1, true);
  v.setUint32(24, 44100, true);
  v.setUint32(28, 88200, true);
  v.setUint16(32, 2, true);
  v.setUint16(34, 16, true);
  tag(36, "data");
  v.setUint32(40, 2 * n, true);
  for (let i = 0; i < n; i++) v.setInt16(44 + 2 * i, Math.sin((2 * Math.PI * hz * i) / 44100) >= 0 ? 12000 : -12000, true);
  return out;
}

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
  mkdirSync(join(game, "src"), { recursive: true });
  mkdirSync(join(game, "assets", "audio"), { recursive: true });
  writeFileSync(join(game, "project.ogt"), PROJECT);
  writeFileSync(join(game, "src", "main.ts"), MAIN);
  writeFileSync(join(game, "assets", "audio", "ui-hover.wav"), blip(880));
  writeFileSync(join(game, "assets", "audio", "ui-press.wav"), blip(440));
  const script = [
    // The pointer reaches Play (hover), then presses it (press).
    "1.5:move_to_text:=Play",
    "1.5:dump_pulse:hover.json",
    "2.5:touch_start",
    "2.5:dump_pulse:press.json",
    "2.6:touch_end",
    "3.2:quit",
  ].join(";");
  const wav = join(work, "mix.wav");
  const r = Bun.spawnSync([ogBin(), "run", game, "--no-check", "--commands", script, "--audio", wav, "--out", work, "--json"], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
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
