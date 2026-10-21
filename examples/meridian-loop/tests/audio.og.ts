// The sound: the engine's voice pitched by the revs (low on the grid, high
// at speed, dropping at a shift), the tyres' loop faded in by a slide and
// out again, the countdown's beeps and GO, the music streamed in chunks and
// looping, and a mix that is heard and never clips.
import { playtest } from "@origingame/engine/playtest";
import { script, value } from "./lib.ts";

interface Source { pitch: number; volume: number; playing: boolean; loop: boolean; bus: string }

export default playtest({
  name: "the engine's pitch follows the revs, the tyres sound in a slide, the countdown beeps, the music streams and the mix never clips",
  commands: script(
    "0.5:race",
    "2:get:engine.og.AudioSource:grid.json",
    "3.6:drive:auto",
    "3.7:get:sfx-1.Sfx:beeps.json",
    "3.7:get:sfx-2.Sfx:go.json",
    "9:get:engine.og.AudioSource:fast.json",
    "9:get:Race:fast-race.json",
    "9:get:tyres.og.AudioSource:straight.json",
    "10:drive:off",
    "10:warp:150:30",
    "10.05:key_down:KeyW",
    "10.05:key_down:KeyA",
    "10.4:get:tyres.og.AudioSource:slide.json",
    "10.4:get:Race:slide-race.json",
    "11:key_up:KeyA",
    "11:key_up:KeyW",
    "12:get:tyres.og.AudioSource:after.json",
    "12:get:music.og.AudioSource:music.json",
    "12:listen:mix.wav:3",
    "12.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const grid = value<Source>(run, "grid.json");
    const fast = value<Source>(run, "fast.json");
    t.ok(grid.loop && grid.playing, "the engine's voice loops");
    t.ok(grid.pitch < 0.9, `on the grid it idles low (pitch ${grid.pitch.toFixed(2)})`);
    t.ok(fast.pitch > grid.pitch + 0.4, `at ${value<{ kmh: number }>(run, "fast-race.json").kmh.toFixed(0)} km/h it is higher (pitch ${fast.pitch.toFixed(2)})`);
    t.equal(value<{ plays: number }>(run, "beeps.json").plays, 3, "the countdown beeped three times");
    t.equal(value<{ plays: number }>(run, "go.json").plays, 1, "and GO once");
    t.equal(value<Source>(run, "straight.json").volume, 0, "the tyres are silent on a straight");
    const slide = value<Source>(run, "slide.json");
    t.ok(value<{ skidding: boolean }>(run, "slide-race.json").skidding, "full lock at 30 m/s slides");
    t.ok(slide.volume > 0.1, `the tyres sound in the slide (volume ${slide.volume.toFixed(2)})`);
    t.ok(value<Source>(run, "after.json").volume < 0.05, "and fall silent after it");
    const music = value<Source>(run, "music.json");
    t.ok(music.playing && music.loop && music.bus === "music", "the music loops on the music bus");
    const mix = run.wav("mix.wav");
    t.ok(mix.rms > 0.01, `the mix is heard (RMS ${mix.rms.toFixed(3)})`);
    t.ok(mix.peak < 0.999, `and never clips (peak ${mix.peak.toFixed(3)})`);
  },
});
