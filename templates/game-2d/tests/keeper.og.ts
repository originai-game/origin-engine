// The lamp keeper's playtest (og-game §8, 2D skeletons): the rig plays its
// idle clip (its event counted as breaths), both legs' IK targets sit on
// the ramp and are reached, and after `keeper:7` walks it up the ramp the
// feet are planted again, higher.
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

export default playtest({
  name: "The lamp keeper breathes on its ramp with both feet planted, and stays planted when it moves up the slope",
  commands:
    "1.0:screenshot:keeper.png;1.01:dump_entity:keeper:keeper.json;1.02:dump_entity:keeper-leg-f:front.json;1.03:dump_entity:keeper-leg-b:back.json;" +
    "2.1:get:Game:game.json;2.2:keeper:7;2.4:dump_entity:keeper-leg-f:front-up.json;2.41:dump_entity:keeper-leg-b:back-up.json;2.5:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    t.frame("keeper.png", () => {
      const frame = run.png("keeper.png");
      t.ok(frame.width > 0 && frame.height > 0, `a ${frame.width}x${frame.height} frame was captured`);
    });
    const skeleton = run.json<EntityDump>("keeper.json").components["og.Skeleton2D"]!;
    t.ok((skeleton["time"] as number) > 0.9 && (skeleton["time"] as number) < 1.1, `the idle clip plays (time ${skeleton["time"]})`);
    for (const [file, label, dx] of [["front.json", "front", -0.15], ["back.json", "back", 0.15], ["front-up.json", "front after the move", -0.15], ["back-up.json", "back after the move", 0.15]] as const) {
      const ik = run.json<EntityDump>(file).components["og.IkTarget2D"]!;
      const target = ik["target"] as number[];
      const x = (file.includes("up") ? 7 : 6) + dx;
      t.ok(Math.abs(target[0]! - x) < 1e-4 && Math.abs(target[1]! - ankleY(x)) < 1e-4, `the ${label} foot's target is on the ramp (${target.join(", ")})`);
      t.ok(ik["reached"] === true, `the ${label} foot reached it (error ${ik["error"]})`);
    }
    // A loop of 2 s from tick 0 to 2.1 s passed its event at 0 and at 2.
    const game = run.json<Got<{ inhales: number }>>("game.json").value;
    t.ok(game.inhales === 2, `og.Skeleton2DEvent counted two breaths (${game.inhales})`);
  },
});
