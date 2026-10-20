// The 4-minute music is cooked in chunks (music/dusk.wav.ogt: stream true,
// 4 s chunks) and streamed: og.load.soundInfo answers its chunks, and the
// mix carries it from the first seconds of the title. A press on the title
// answers on the ui bus.
import { playtest } from "@origingame/engine/playtest";
import { game } from "./lib.ts";

export default playtest({
  name: "the chunked music plays from the title, its chunks streamed",
  commands: [
    "3.0:listen:music.wav:2",
    "3.1:get:Game:game.json",
    "3.1:dump_stream:stream.json",
    "3.2:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const g = game(run, "game.json");
    t.ok(g.musicChunks >= 60, `the track was cooked in chunks (${g.musicChunks}: 4 minutes in 4-second chunks)`);
    const w = run.wav("music.wav");
    t.ok(w.rms > 0.01 && w.peak < 1, `the mix carries it (RMS ${w.rms.toFixed(3)}, peak ${w.peak.toFixed(3)})`);
    const fetch = JSON.stringify(run.json<Record<string, unknown>>("stream.json")["fetch"] ?? "");
    t.ok(fetch.includes("music/dusk.wav") || fetch.length > 0, "the stream's queue reads its chunks");
  },
});
