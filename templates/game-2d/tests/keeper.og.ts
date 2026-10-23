// The lamp keeper's playtest (og-game §8, 2D skeletons): the rig plays its
// idle clip (its event counted as breaths) with both legs' IK targets on
// the ramp and reached; `keeper:7` sends it walking up the ramp with its
// walk clip, facing the way it goes, one foot planted on the plank while
// the other swings above it; it arrives, faces the spawn and breathes again
// with both feet planted, higher. Every tick from the command to the end
// is dumped: the turns take several ticks, and no foot ever moves further
// in a tick than the gait's own swing does, setting off and stopping
// included; the idle clip comes back only once both feet are down.
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
/**
 * The most a foot moves in a 60 Hz tick: the gait's swing at its start,
 * worked out by hand from src/main.ts's WALK and RAMP. Forward twice the
 * walking speed, 1 m/s; up the 0.21256 slope at that, plus the lift's
 * parabola at its steepest, 4 x 0.08 m over the swing's half cycle of
 * 0.4 s (2.5 per second of its parameter), 0.8 m/s: 1.01256 m/s. Together
 * 1.42316 m/s, 0.023719 m a tick.
 */
const STRIDE_STEP = 0.023719;
/** The ticks dumped every tick: from just before `keeper:7` (2.2 s) to the end (5 s). */
const FIRST = 130;
const LAST = 300;
const perTick = Array.from({ length: LAST - FIRST + 1 }, (_, i) => {
  const k = FIRST + i;
  const at = (j: number) => (k / 60 + 0.002 * j).toFixed(4);
  return `${at(1)}:dump_entity:keeper:k${k}.json;${at(2)}:dump_entity:keeper-leg-f:f${k}.json;${at(3)}:dump_entity:keeper-leg-b:b${k}.json`;
}).join(";");

export default playtest({
  name: "The lamp keeper breathes on its ramp with both feet planted, walks up the slope stepping on the plank, and stands planted again",
  commands: [
    "1.0:screenshot:keeper.png;1.01:dump_entity:keeper:keeper.json;1.02:dump_entity:keeper-leg-f:front.json;1.03:dump_entity:keeper-leg-b:back.json",
    "2.1:get:Game:game.json;2.2:keeper:7",
    "2.85:screenshot:walk.png;2.851:dump_entity:keeper:walk.json;2.852:dump_entity:keeper-leg-f:walk-front.json;2.853:dump_entity:keeper-leg-b:walk-back.json",
    "3.15:dump_entity:keeper:walk-later.json;3.151:dump_entity:keeper-leg-f:walk-front-later.json;3.152:dump_entity:keeper-leg-b:walk-back-later.json",
    perTick,
    "5.1:dump_entity:keeper:keeper-up.json;5.11:dump_entity:keeper-leg-f:front-up.json;5.12:dump_entity:keeper-leg-b:back-up.json;5.2:get:Game:game-end.json;5.3:quit",
  ]
    .join(";")
    .split(";")
    .sort((a, b) => Number(a.split(":")[0]) - Number(b.split(":")[0]))
    .join(";"),
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
    // The back foot carried the keeper from 2.85 s to 3.15 s (the walk starts after the turn): it did not move.
    const before = foot("walk-back.json")["target"] as number[];
    const later = foot("walk-back-later.json")["target"] as number[];
    t.ok(Math.abs(before[0]! - later[0]!) < 1e-4 && Math.abs(before[1]! - later[1]!) < 1e-4, `the planted foot stays put while the keeper passes over it (${before.join(", ")} then ${later.join(", ")})`);

    // Arrived: idle again, facing the spawn, at x 7.
    const up = skeleton("keeper-up.json");
    t.ok(up["clip"] === IDLE && up["flipX"] === true && Math.abs(at("keeper-up.json") - 7) < 1e-4, `it arrived at 7 and breathes facing the spawn (clip ${up["clip"]}, x ${at("keeper-up.json")})`);

    // Tick by tick: the keeper's scale x, facing, clip and place, and each foot's target.
    const ticks = Array.from({ length: LAST - FIRST + 1 }, (_, i) => {
      const k = FIRST + i;
      const keeper = run.json<EntityDump>(`k${k}.json`).components;
      const s = keeper["og.Skeleton2D"]!;
      return {
        k,
        x: (keeper["og.Transform"]!["position"] as number[])[0]!,
        squash: (keeper["og.Transform"]!["scale"] as number[])[0]!,
        flipX: s["flipX"] === true,
        clip: s["clip"] as number,
        feet: [`f${k}.json`, `b${k}.json`].map((f) => foot(f)["target"] as number[]),
      };
    });
    // No foot moves further in a tick than the gait's swing: setting off, walking, stopping and both turns.
    let worst = 0;
    let worstAt = 0;
    for (let i = 1; i < ticks.length; i++) {
      for (const f of [0, 1]) {
        const [a, b] = [ticks[i - 1]!.feet[f]!, ticks[i]!.feet[f]!];
        const d = Math.hypot(b[0]! - a[0]!, b[1]! - a[1]!);
        if (d > worst) [worst, worstAt] = [d, ticks[i]!.k];
      }
    }
    t.ok(worst <= STRIDE_STEP + 1e-6, `no foot moves more than a gait tick's ${STRIDE_STEP} m in one tick (at most ${worst.toFixed(6)} m, tick ${worstAt})`);
    // The walk itself moves the swinging foot nearly that far, so the bound is the gait's, not a loose one.
    t.ok(worst > STRIDE_STEP * 0.9, `the walk's swing reaches the bound (${worst.toFixed(6)} m)`);
    // Each turn: several ticks of a narrowed rig (never edge-on), standing still, between one facing and the other.
    const turns: { from: number; to: number; ticks: number; still: boolean; steady: boolean }[] = [];
    for (let i = 1; i < ticks.length; i++) {
      if (ticks[i]!.flipX === ticks[i - 1]!.flipX) continue;
      let j = i - 1;
      while (j > 0 && ticks[j]!.squash !== 1) j--;
      const span = ticks.slice(j, i + 1);
      turns.push({
        from: ticks[j]!.k,
        to: ticks[i]!.k,
        ticks: i - j - 1,
        still: span.every((u) => Math.abs(u.x - span[0]!.x) < 1e-6),
        steady: span.every((u, n) => u.squash !== 0 && (n === 0 || Math.abs(u.squash * (u.flipX ? -1 : 1) - span[n - 1]!.squash * (span[n - 1]!.flipX ? -1 : 1)) <= 0.5)),
      });
    }
    t.ok(turns.length === 2, `the keeper turned twice: setting off and on arrival (${turns.length})`);
    for (const u of turns) {
      t.ok(u.ticks >= 4 && u.still && u.steady, `a turn (ticks ${u.from} to ${u.to}) narrows the rig over ${u.ticks} ticks in place, never edge-on or by half its width at once`);
    }
    // The stop: the idle clip returns only with both feet down on the ramp, where they stand.
    const back = ticks.findIndex((u, i) => i > 0 && u.clip === IDLE && ticks[i - 1]!.clip !== IDLE);
    t.ok(back > 0, `the idle clip came back (tick ${ticks[back]?.k})`);
    if (back > 0) {
      const u = ticks[back]!;
      for (const [f, dx] of [[0, -0.15], [1, 0.15]] as const) {
        const [fx, fy] = u.feet[f]!;
        t.ok(Math.abs(fx! - (u.x + dx)) < 1e-4 && Math.abs(fy! - ankleY(fx!)) < 1e-4, `when idle returns, foot ${f} is down where it stands (${fx}, ${fy})`);
      }
      const arrived = ticks.findIndex((v) => Math.abs(v.x - 7) < 1e-6);
      t.ok(arrived > 0 && back - arrived > 1, `between arriving (tick ${ticks[arrived]?.k}) and breathing (tick ${u.k}) the feet land and the keeper turns`);
    }

    // A loop of 2 s from tick 0 to 2.1 s passed its event at 0 and at 2; the walk
    // (2 s at 0.5 m/s, a step every 0.4 s) stepped 5 times, and the idle clip
    // started over on arrival with a breath.
    const game = run.json<Got<{ inhales: number }>>("game.json").value;
    t.ok(game.inhales === 2, `og.Skeleton2DEvent counted two breaths before the walk (${game.inhales})`);
    const end = run.json<Got<{ inhales: number; steps: number }>>("game-end.json").value;
    t.ok(end.steps === 5 && end.inhales === 3, `the walk clip's events counted 5 steps, then a third breath (${end.steps}, ${end.inhales})`);
  },
});
