// The music: "Under the Hill" (tools/make-assets.ts, about 85 s of 44.1 kHz
// stereo) is cooked in 4-second chunks and streamed (music/hollow.wav.ogt,
// og-audio §5), plays on the music bus from the title and is heard; a
// glimmer's chime in play is heard over it.
import { playtest } from "@origingame/engine/playtest";
import { game, script, value } from "./lib.ts";

export default playtest({
  name: "the streamed music is cooked in chunks and heard from the title",
  commands: script(
    "2.0:get:music.og.AudioSource:music.json",
    "3.5:listen:title.wav:1.5",
    "3.5:get:Game:chunks.json",
    "3.6:play",
    "3.7:warp:24:31",
    "4.6:listen:play.wav:0.8",
    "4.6:get:sfx-64.Sfx:glimmer.json",
    "4.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const music = value<{ playing: boolean; loop: boolean; bus: string | number }>(run, "music.json");
    // `get:` answers the bus by its number (1), game code by its name (FINDINGS.md #19).
    t.ok(music.playing && music.loop && (music.bus === "music" || music.bus === 1), "the music loops on the music bus");
    const chunks = game(run, "chunks.json").musicChunks;
    t.ok(chunks >= 20, `it was cooked in chunks and streams (${chunks} chunks)`);
    const title = run.wav("title.wav");
    t.ok(title.rms > 0.01 && title.peak < 1, `heard from the title (rms ${title.rms.toFixed(3)}, peak ${title.peak.toFixed(2)})`);
    t.ok(value<{ plays: number }>(run, "glimmer.json").plays >= 1, "the well's glimmers chimed on the way down");
    const play = run.wav("play.wav");
    t.ok(play.rms > 0.01 && play.peak < 1, `and the mix stays under clipping in play (peak ${play.peak.toFixed(2)})`);
  },
});
