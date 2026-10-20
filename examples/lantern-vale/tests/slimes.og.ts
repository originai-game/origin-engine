// A slime's sight and touch: by the slime of the west round, the hero is
// seen within Tuning.sight; the slime turns alert (its bubble, an og.Sprite
// drawing layer 1 of the faces' texture array, with a drop shadow) and
// bounds after the hero (og.AnimationPlayer changes to the `chase` clip,
// still stepped at 12 poses a second). Its touch costs a heart with the
// hit's weight: a hit stop, a shake, the red flash, the hurt sound, the
// hero thrown back and spared a moment.
import { playtest } from "@origingame/engine/playtest";
import { game, value } from "./lib.ts";

const SAMPLES: number[] = [];
for (let t = 1.3; t < 2.31; t += 1 / 30) SAMPLES.push(Math.round(t * 1000) / 1000);

export default playtest({
  name: "a slime sees the hero, turns alert and chases on twos, and its touch lands with weight",
  commands: [
    "0.3:run",
    "0.35:get:slime-0.og.AnimationPlayer:idle.json",
    "0.4:warp:-19:0",
    "0.7:get:slime-0.Slime:seen.json",
    "0.7:get:slime-0-face.og.Sprite:face.json",
    "0.7:get:slime-0.og.AnimationPlayer:chase.json",
    ...SAMPLES.flatMap((t) => [`${t}:get:og.TimeScale:ts-${t}.json`, `${t}:get:og.Camera:cam-${t}.json`, `${t}:get:Game:g-${t}.json`]),
    "2.4:get:Game:after.json",
    "2.4:get:sfx-4.Sfx:hurt.json",
    "2.4:get:slime-0.Slime:slime.json",
    "2.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const idle = value<{ clip: number; stepFps: number }>(run, "idle.json");
    t.equal(idle.stepFps, 12, "the slime's AnimationPlayer holds each pose for 1/12 s (on twos)");
    const seen = value<{ chasing: boolean; face: number }>(run, "seen.json");
    t.ok(seen.chasing && seen.face === 1, "within sight it chases, alert");
    const face = value<{ array: number; arrayLayer: number; dropShadow: number }>(run, "face.json");
    t.ok(face.array !== 0 && face.arrayLayer === 1, `its bubble draws the alert layer of the faces' array (layer ${face.arrayLayer})`);
    t.ok(face.dropShadow > 0, "with a drop shadow on the ground");
    const chase = value<{ clip: number; stepFps: number }>(run, "chase.json");
    t.ok(chase.clip !== idle.clip && chase.stepFps === 12, "and bounds on the chase clip, still on twos");
    // The touch: the first sample after the heart is lost.
    const hit = SAMPLES.find((s) => game(run, `g-${s}.json`).hurts > 0);
    t.ok(hit !== undefined, `the slime touched the hero (at ${hit} s)`);
    if (hit === undefined) return;
    const at = SAMPLES.indexOf(hit);
    const near = SAMPLES.slice(Math.max(0, at - 1), at + 2);
    t.ok(near.some((s) => value<{ hitStop: number }>(run, `ts-${s}.json`).hitStop > 0), "the world held for a hit stop");
    t.ok(near.some((s) => value<{ trauma: number }>(run, `cam-${s}.json`).trauma > 0.1), "and the camera shook");
    const after = game(run, "after.json");
    t.ok(after.hearts === 2 && after.hurts === 1, `a heart gone (${after.hearts} left)`);
    t.ok(value<{ plays: number }>(run, "hurt.json").plays >= 1, "the hurt sound played");
    t.ok(value<{ hits: number; chasing: boolean }>(run, "slime.json").hits === 1, "the slime counts its hit and reels back");
  },
});
