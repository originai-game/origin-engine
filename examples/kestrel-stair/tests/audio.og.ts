// The sound of the stair (og-audio §4-§6). "The Kestrel Stair" is cooked in
// 4-second chunks and streams, looping on the music bus from the title,
// heard and never clipping. A brazier's crackle is a spatial source heard
// from the listener on the camera: loud and nearly centred beside it, quiet,
// panned and dulled (distanceCutoff) across the terrace. The crypt's drip
// sounds behind its walls (occlude: the low-pass at og.Audio's
// occlusionCutoff) until the courier walks in. The crypt is a reverb zone:
// inside, the effects bus sends 0.55 to the shared reverb in a room of 0.92
// (0.12 and 0.5 outside), eased over half a second.
import { playtest } from "@origingame/engine/playtest";
import { game, script, value } from "./lib.ts";

interface Playback { gain: number; pan: number; lowpass: number }

export default playtest({
  name: "the streamed music loops unclipped; a brazier is heard from where the camera is; the crypt's drip is muffled by its walls; the crypt is a reverb zone",
  commands: script(
    "1.5:listen:title.wav:1.2",
    "1.6:get:music.og.AudioSource:music.json",
    "1.6:get:Game:chunks.json",
    "1.7:play:1:quick",
    "1.75:set:Tuning.sight=0",
    "1.8:warp:-5:8",
    "3.0:get:brazier-0.og.AudioPlayback:near.json",
    "3.0:listen:play.wav:1",
    "3.1:warp:15:-20",
    "4.3:get:brazier-0.og.AudioPlayback:far.json",
    "4.4:warp:crypt:1:0",
    "5.4:get:crypt-drip.og.AudioPlayback:walled.json",
    "5.4:get:Game:outside.json",
    "5.4:get:sfx-bus.og.AudioBusFx:bus-out.json",
    "5.5:warp:crypt:-8:-3",
    "6.5:get:crypt-drip.og.AudioPlayback:drip-inside.json",
    "6.5:get:Game:inside.json",
    "6.5:get:sfx-bus.og.AudioBusFx:bus-in.json",
    "6.5:get:og.Audio:room.json",
    "6.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const music = value<{ playing: boolean; loop: boolean; bus: string }>(run, "music.json");
    t.ok(music.playing && music.loop && music.bus === "music", "the music loops on the music bus");
    const chunks = game(run, "chunks.json").musicChunks;
    t.ok(chunks >= 30, `cooked in chunks and streamed (${chunks} chunks)`);
    const title = run.wav("title.wav");
    t.ok(title.rms > 0.01 && title.peak < 1, `heard from the title (rms ${title.rms.toFixed(3)}, peak ${title.peak.toFixed(2)})`);
    const play = run.wav("play.wav");
    t.ok(play.peak < 1, `the mix in play stays under clipping (peak ${play.peak.toFixed(2)})`);
    const near = value<Playback>(run, "near.json");
    const far = value<Playback>(run, "far.json");
    t.ok(near.gain > far.gain * 3, `a brazier beside the courier is louder than across the terrace (${near.gain.toFixed(3)} against ${far.gain.toFixed(3)})`);
    t.ok(Math.abs(far.pan) > Math.abs(near.pan), `and from across the terrace it is panned to its side (${near.pan.toFixed(2)}, then ${far.pan.toFixed(2)})`);
    t.ok(far.lowpass < near.lowpass, `and dulled with distance (low-pass ${near.lowpass.toFixed(0)} Hz, then ${far.lowpass.toFixed(0)} Hz)`);
    const walled = value<Playback>(run, "walled.json");
    const inside = value<Playback>(run, "drip-inside.json");
    t.ok(walled.lowpass > 0 && walled.lowpass <= 900, `from the door the drip is behind the crypt's walls (low-pass ${walled.lowpass.toFixed(0)} Hz)`);
    t.ok(inside.lowpass === 0 && inside.gain > walled.gain * 2, `inside it is clear and near (gain ${walled.gain.toFixed(3)}, then ${inside.gain.toFixed(3)})`);
    t.equal([game(run, "outside.json").inCrypt, game(run, "inside.json").inCrypt], [false, true], "the crypt is known as a zone");
    t.near(value<{ reverb: number }>(run, "bus-out.json").reverb, 0.12, 0.02, "outside, the effects bus sends 0.12 to the reverb");
    t.near(value<{ reverb: number }>(run, "bus-in.json").reverb, 0.55, 0.03, "inside, 0.55");
    t.ok(value<{ reverbRoom: number }>(run, "room.json").reverbRoom > 0.85, "in a large room");
  },
});
