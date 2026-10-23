// The lamp keeper's playtest (og-game §8, 2D skeletons): the rig plays its
// idle clip (its event counted as breaths) with both legs' IK targets on
// the ramp and reached; `keeper:7` sends it walking up the ramp with its
// walk clip, facing the way it goes, one foot planted on the plank while
// the other swings above it; it arrives, faces the spawn and breathes again
// with both feet planted, higher.
import { playtest } from "@origingame/engine/playtest";

interface Got<T> {
  value: T;
}

type Fields = Record<string, number | boolean | string | number[]>;

interface EntityDump {
  components: Record<string, Fields>;
}

/** The ramp's top at x (src/main.ts's RAMP), plus the ankle's height. */
const ankleY = (x: number): number => 2 + 0.21256 * (x - 4.5) + 0.12;
/** The plank's ends and the highest a swinging foot lifts (src/main.ts's RAMP and WALK). */
const PLANK = { x0: 4.5, x1: 7.6 } as const;
const LIFT = 0.08;
/** The rig's clips, in file order. */
const IDLE = 0;
const WALK = 1;

export default playtest({
  name: "The lamp keeper breathes on its ramp with both feet planted, walks up the slope stepping on the plank, and stands planted again",
  commands:
    "1.0:screenshot:keeper.png;1.01:dump_entity:keeper:keeper.json;1.02:dump_entity:keeper-leg-f:front.json;1.03:dump_entity:keeper-leg-b:back.json;" +
    "2.1:get:Game:game.json;2.2:keeper:7;" +
    "2.6:screenshot:walk.png;2.61:dump_entity:keeper:walk.json;2.62:dump_entity:keeper-leg-f:walk-front.json;2.63:dump_entity:keeper-leg-b:walk-back.json;" +
    "2.9:dump_entity:keeper:walk-later.json;2.91:dump_entity:keeper-leg-f:walk-front-later.json;2.92:dump_entity:keeper-leg-b:walk-back-later.json;" +
    "4.6:dump_entity:keeper:keeper-up.json;4.61:dump_entity:keeper-leg-f:front-up.json;4.62:dump_entity:keeper-leg-b:back-up.json;4.7:get:Game:game-end.json;4.8:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    for (const png of ["keeper.png", "walk.png"]) {
      t.frame(png, () => {
        const frame = run.png(png);
        t.ok(frame.width > 0 && frame.height > 0, `a ${frame.width}x${frame.height} frame was captured`);
      });
    }
    const skeleton = (file: string) => run.json<EntityDump>(file).components["og.Skeleton2D"]!;
    const at = (file: string) => (run.json<EntityDump>(file).components["og.Transform"]!["position"] as number[])[0]!;
    const foot = (file: string) => run.json<EntityDump>(file).components["og.IkTarget2D"]!;

    // Standing: the idle clip, both feet on the ramp under the keeper.
    const idle = skeleton("keeper.json");
    t.ok(idle["clip"] === IDLE && (idle["time"] as number) > 0.9 && (idle["time"] as number) < 1.1, `the idle clip plays (clip ${idle["clip"]}, time ${idle["time"]})`);
    for (const [file, label, dx, x0] of [["front.json", "front", -0.15, 6], ["back.json", "back", 0.15, 6], ["front-up.json", "front after the walk", -0.15, 7], ["back-up.json", "back after the walk", 0.15, 7]] as const) {
      const ik = foot(file);
      const target = ik["target"] as number[];
      const x = x0 + dx;
      t.ok(Math.abs(target[0]! - x) < 1e-4 && Math.abs(target[1]! - ankleY(x)) < 1e-4, `the ${label} foot's target is on the ramp (${target.join(", ")})`);
      t.ok(ik["reached"] === true, `the ${label} foot reached it (error ${ik["error"]})`);
    }

    // Walking: the walk clip, facing up the ramp, the keeper between where it was and where it goes.
    let planted = 0;
    for (const [keeper, front, back] of [["walk.json", "walk-front.json", "walk-back.json"], ["walk-later.json", "walk-front-later.json", "walk-back-later.json"]] as const) {
      const s = skeleton(keeper);
      const x = at(keeper);
      t.ok(s["clip"] === WALK && s["flipX"] === false, `walking, the keeper plays its walk clip facing right (clip ${s["clip"]}, flipX ${s["flipX"]})`);
      t.ok(x > 6 && x < 7, `walking, it is part of the way along the ramp (x ${x})`);
      for (const [file, label] of [[front, "front"], [back, "back"]] as const) {
        const ik = foot(file);
        const [fx, fy] = ik["target"] as [number, number];
        const lift = fy - ankleY(fx);
        t.ok(fx > PLANK.x0 && fx < PLANK.x1 && lift > -1e-4 && lift <= LIFT + 1e-4, `walking, the ${label} foot is on the plank or above it (x ${fx}, ${lift.toFixed(4)} up)`);
        t.ok(ik["reached"] === true, `walking, the ${label} foot reached its target (error ${ik["error"]})`);
        if (Math.abs(lift) < 1e-4) planted += 1;
      }
    }
    t.ok(planted >= 2, `while it walks a foot stays planted on the plank (${planted} of 4 targets on it)`);
    // The back foot carried the keeper from 2.6 s to 2.9 s: it did not move.
    const before = foot("walk-back.json")["target"] as number[];
    const later = foot("walk-back-later.json")["target"] as number[];
    t.ok(Math.abs(before[0]! - later[0]!) < 1e-4 && Math.abs(before[1]! - later[1]!) < 1e-4, `the planted foot stays put while the keeper passes over it (${before.join(", ")} then ${later.join(", ")})`);

    // Arrived: idle again, facing the spawn, at x 7.
    const up = skeleton("keeper-up.json");
    t.ok(up["clip"] === IDLE && up["flipX"] === true && Math.abs(at("keeper-up.json") - 7) < 1e-4, `it arrived at 7 and breathes facing the spawn (clip ${up["clip"]}, x ${at("keeper-up.json")})`);

    // A loop of 2 s from tick 0 to 2.1 s passed its event at 0 and at 2; the walk
    // (2 s at 0.5 m/s, a step every 0.4 s) stepped 5 times, and the idle clip
    // started over on arrival with a breath.
    const game = run.json<Got<{ inhales: number }>>("game.json").value;
    t.ok(game.inhales === 2, `og.Skeleton2DEvent counted two breaths before the walk (${game.inhales})`);
    const end = run.json<Got<{ inhales: number; steps: number }>>("game-end.json").value;
    t.ok(end.steps === 5 && end.inhales === 3, `the walk clip's events counted 5 steps, then a third breath (${end.steps}, ${end.inhales})`);
  },
});
