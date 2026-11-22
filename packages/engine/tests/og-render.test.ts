// The actual media, not only frames.json: the leader must not overflow
// the audio queue while simulating between --jobs snapshots.
import { expect } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { ogBin } from "./support/og-bin.ts";
import { gpuTest } from "./support/gpu.ts";

declare const Bun: {
  spawnSync(cmd: string[], options: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

gpuTest("render keeps every encoded frame and the full audio duration with multiple jobs", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-render-duration-"));
  const exec = (args: string[]) => {
    const result = Bun.spawnSync(args, { cwd: dir });
    expect(result.exitCode, new TextDecoder().decode(result.stderr)).toBe(0);
    return new TextDecoder().decode(result.stdout);
  };
  try {
    writeFileSync(join(dir, "main.ts"), `
import { og, defineSystem } from "@origingame/engine";
const Volume = defineSystem("Volume", { writes: [og.Audio] }, (_q, time, world) => {
  const audio = world.resource(og.Audio);
  if (audio.ok) audio.value.master = time.tick % 2 ? 0.5 : 1;
});
export default og.game({ tick: 60, systems: [Volume] });
`);
    // Also exercise the script-file path through the leader and its workers.
    const script = "0:set:og.Audio.master=1;".repeat(6_000) + "90:quit";
    writeFileSync(join(dir, "commands.txt"), `\n${script}\n`);
    for (const [jobs, audio] of [[1, true], [3, true], [3, false]] as const) {
      const name = `j${jobs}-${audio}`;
      // Low fps keeps this small. Each leader interval still has more
      // audio packets than the link's 1024-packet bound at 60 Hz.
      exec([ogBin(), "render", "main.ts", "--no-check", "--commands-file", "commands.txt", "--to", "82", "--fps", "2", "--size", "16x16", "--jobs", String(jobs), ...(audio ? ["--audio"] : []), "--out", `${name}.mp4`]);
      const manifest = JSON.parse(readFileSync(join(dir, `${name}.frames.json`), "utf8"));
      expect(manifest.commands).toBe(script);
      expect(manifest.frames.length).toBe(164);
      const video = JSON.parse(exec(["ffprobe", "-v", "error", "-count_frames", "-select_streams", "v:0", "-show_entries", "stream=nb_read_frames,duration", "-of", "json", `${name}.mp4`])).streams[0];
      const media = { frames: Number(video.nb_read_frames), videoSeconds: Number(video.duration), wavSeconds: 0, audioSeconds: 0 };
      if (audio) {
        const wav = JSON.parse(exec(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "json", `${name}.audio.wav`]));
        const muxed = JSON.parse(exec(["ffprobe", "-v", "error", "-select_streams", "a:0", "-show_entries", "stream=duration", "-of", "json", `${name}.mp4`])).streams[0];
        media.wavSeconds = Number(wav.format.duration);
        media.audioSeconds = Number(muxed.duration);
      }
      expect(media).toEqual({ frames: 164, videoSeconds: 82, wavSeconds: audio ? 82 : 0, audioSeconds: audio ? 82 : 0 });
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
