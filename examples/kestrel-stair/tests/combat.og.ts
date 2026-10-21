// The sword (og-feel §1-§7): a press swings on its tick (the swing sound,
// the attack clip over the blend tree), and the hit lands on the clip's
// `hit` marker, an og.AnimationEvent: hit stop by weight (7 ticks on a
// warden), a shake, the warden flinching (a squash: FINDINGS #7), a number
// and the hit sound. Six strikes fell the terrace warden (eight are swung, the
// courier pressing on): its die clip, the
// world held longer, and the third sigil dropped where it fell, which F
// takes (the burst, the sound, the HUD's count). A slam taken costs a third
// of the courier's strength with a red flash, a heavier shake and a
// knock-back.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, press, script, value } from "./lib.ts";
import type { Foe } from "./lib.ts";

const STRIKES = [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5];

export default playtest({
  name: "strikes land on the clip's marker with hit stop, shake, a flinch, a number and a sound; the warden falls and drops a sigil the courier takes; a slam hurts",
  commands: script(
    "0.3:play:1:quick",
    "0.35:set:Tuning.sight=0",
    "0.35:set:Tuning.wardenPatrol=0",
    "0.5:warp:warden-0:0:-1.2",
    "0.9:get:Game:before.json",
    ...STRIKES.map((t) => press("KeyJ", t)),
    // Pressing on into the warden, so a strike that threw it back still reaches.
    hold("KeyW", 1.3, 3.6),
    "1.017:get:hero-body.og.AnimationPlayer:swing.json",
    "1.017:get:sfx-2.Sfx:swing-sound.json",
    "1.15:get:og.TimeScale:stop.json",
    "1.15:get:warden-0.og.Transform:held0.json",
    "1.18:get:warden-0.og.Transform:held1.json",
    "1.18:get:warden-0.Foe:hit1.json",
    "1.25:get:og.Camera:shake.json",
    "1.25:screenshot:strike.png",
    "5.4:get:Game:felled.json",
    "5.4:get:warden-0.Foe:down.json",
    "5.4:get:warden-0.og.AnimationPlayer:die.json",
    "5.4:get:sfx-4.Sfx:hit-sound.json",
    "5.5:warp:sigil-2",
    press("KeyF", 5.8),
    "6.1:get:Game:taken.json",
    "6.1:get:sigil-burst.og.ParticleSystem:burst.json",
    "6.1:get:sfx-128.Sfx:sigil-sound.json",
    // The second warden's slam.
    "6.3:set:Tuning.sight=11",
    "6.3:set:Tuning.wardenPatrol=1.1",
    "6.4:play:2:quick",
    "6.5:warp:3:-37",
    "10.0:get:Game:slammed.json",
    "10.0:get:sfx-8.Sfx:hurt-sound.json",
    "10.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = game(run, "before.json");
    t.equal([before.hits, before.kills, before.sigils], [0, 0, 0], "a fresh chapter II");
    const swing = value<{ weight: number; time: number }>(run, "swing.json");
    t.ok(swing.weight === 1 && swing.time < 0.05, "the press swung on its tick: the attack clip over the graph");
    t.ok(value<{ plays: number }>(run, "swing-sound.json").plays >= 1, "with the swing's sound");
    t.ok(value<{ hitStop: number }>(run, "stop.json").hitStop > 0, "the hit froze the world (hit stop)");
    const h0 = value<{ position: number[]; scale: number[] }>(run, "held0.json");
    const h1 = value<{ position: number[]; scale: number[] }>(run, "held1.json");
    t.equal(h1.position, h0.position, "the warden held still through the stop");
    t.ok(h0.scale[0]! > 1.05 && h0.scale[1]! < 0.95, `and flinched (${h0.scale[0]!.toFixed(2)} x ${h0.scale[1]!.toFixed(2)})`);
    t.equal(value<Foe>(run, "hit1.json").hp, 5, "one stone lost");
    t.ok(value<{ trauma: number }>(run, "shake.json").trauma > 0.05, "the camera shook");
    const felled = game(run, "felled.json");
    t.ok(felled.hits >= 6 && felled.kills === 1 && (felled.foesDown & 4) === 4, `${felled.hits} strikes felled it`);
    t.ok(value<Foe>(run, "down.json").dying > 0, "it is falling");
    t.ok(value<{ weight: number; finished: boolean }>(run, "die.json").weight === 1, "on its die clip");
    t.ok(value<{ plays: number }>(run, "hit-sound.json").plays >= felled.hits, "every strike that landed was heard");
    const taken = game(run, "taken.json");
    t.equal(taken.sigils, 4, "the warden's sigil, dropped where it fell, was taken with F");
    t.ok(value<{ visible: boolean; startTick: number }>(run, "burst.json").visible, "in a burst of gold");
    t.ok(value<{ plays: number }>(run, "sigil-sound.json").plays >= 1, "with its sound");
    const slammed = game(run, "slammed.json");
    t.ok(slammed.hp <= 66 && slammed.hurts >= 1, `the high terrace's warden slammed the courier (strength ${slammed.hp})`);
    t.ok(value<{ plays: number }>(run, "hurt-sound.json").plays >= 1, "the hurt was heard");
  },
});
