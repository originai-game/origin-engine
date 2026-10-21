// Sound: the music streams on its bus from the title (cooked in 4-second
// chunks), a jump, a landing, a gear and a plate each start their voice on
// the tick they happen, the music bus ducks under the effects, and the
// mix heard is neither silent nor clipped.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, script, value } from "./lib.ts";

interface Source {
  bus: string;
  playing: boolean;
  loop: boolean;
}

export default playtest({
  name: "the music streams on its bus, the jump, landing, gear and plate voices play, the mix is heard unclipped",
  commands: script(
    "1.0:get:music.og.AudioSource:music.json",
    "1.0:get:Sfx:sfx0.json",
    "1.1:play:1",
    hold("Space", 1.6, 0.3),
    "1.633:get:voice-jump.og.AudioSource:jump.json",
    "2.6:get:Sfx:sfx1.json",
    "2.6:get:Game:moved.json",
    "2.7:warp:l1-gear-0:0:-0.4",
    "2.75:get:voice-gear.og.AudioSource:gear.json",
    "2.9:warp:l1-plate-a:0:0",
    "3.0:get:voice-plate.Voice:plate.json",
    "3.5:listen:mix.wav:3",
    "3.6:get:bus-music.og.AudioBusFx:bus.json",
    "3.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const music = value<Source>(run, "music.json");
    t.ok(music.playing && music.loop && music.bus === "music", "the music loops on the music bus from the title");
    t.ok(value<Source>(run, "jump.json").playing, "the jump's voice is playing two ticks after the press (the controller's og.CharacterEvent is read the next tick)");
    const moved = game(run, "moved.json");
    t.ok(moved.jumps === 1 && moved.landings >= 1, "one jump and its landing");
    const plays = value<{ plays: number }>(run, "sfx1.json").plays - value<{ plays: number }>(run, "sfx0.json").plays;
    t.ok(plays >= 2, `the jump and the landing were heard (${plays} voices started)`);
    t.ok(value<Source>(run, "gear.json").playing, "a gear taken plays its chime");
    t.equal(value<{ plays: number }>(run, "plate.json").plays, 1, "a plate pressed played its click once");
    const bus = value<{ duck: boolean; duckBus: string }>(run, "bus.json");
    t.ok(bus.duck && bus.duckBus === "sfx", "the music bus ducks under the effects");
    const wav = run.wav("mix.wav");
    t.ok(wav.rms > 0.005, `the mix is heard (RMS ${wav.rms.toFixed(4)})`);
    t.ok(wav.peak < 0.999, `and never clips (peak ${wav.peak.toFixed(3)})`);
  },
});
