// The round's whole loop through real input: the title waits for a press,
// the run starts and speeds up, an untouched crate ends it on the game-over
// card, and a press after the card's hold starts round two from zero.
// Every system sees the rows it needs.
import { playtest } from "@origingame/engine/playtest";
import { OVER, RUNNING, TITLE, runOf, tap } from "./lib.ts";

export default playtest({
  name: "title, run, crash on a crate, game over, run again",
  commands: [
    "0.3:screenshot:title.png",
    "0.4:dump_entity:runner:title.json",
    tap(0.5),
    "0.6:set:runner.Tuning.spawnInterval=60",
    "2.0:screenshot:play.png",
    "2.1:dump_entity:runner:running.json",
    "2.2:place:crate:3",
    "3.6:dump_entity:runner:over.json",
    tap(3.65),
    "3.75:dump_entity:runner:held.json",
    "4.0:screenshot:over.png",
    tap(4.6),
    "4.9:dump_entity:runner:again.json",
    "5.0:dump_world:world.json",
    "5.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["title.png", "play.png", "over.png"]) {
      const png = run.png(f);
      t.ok(png.width === 1280 && png.height === 720, `${f} is a 1280x720 frame`);
    }
    const title = runOf(run, "title.json");
    t.equal(title.phase, TITLE, "the game opens on the title");
    t.equal(title.rounds, 0, "no round has started on the title");

    const running = runOf(run, "running.json");
    t.equal(running.phase, RUNNING, "a press starts the run");
    t.ok(running.distance > 5 && running.distance < 12, `the runner covered ${running.distance.toFixed(2)} m in about 1.5 s`);
    t.ok(running.speed > 6, `the speed ramps above the start speed (${running.speed.toFixed(3)} m/s)`);

    const over = runOf(run, "over.json");
    t.equal(over.phase, OVER, "running into the crate ends the round");
    t.equal(over.best, over.score, "the best score takes the round's score");
    t.ok(over.score >= Math.floor(running.distance), `the score kept the distance (${over.score})`);

    const held = runOf(run, "held.json");
    t.equal(held.phase, OVER, "a press during the card's hold does not restart");

    const again = runOf(run, "again.json");
    t.equal(again.phase, RUNNING, "a press after the hold starts round two");
    t.equal(again.rounds, 2, "two rounds were started");
    t.ok(again.distance < 3 && again.coins === 0, "round two starts from zero");
    t.equal(again.best, over.best, "the best score survives the restart");

    const world = run.json<{ systems: Array<{ name: string; matched: number | null }> }>("world.json");
    // 36 actor rows: sky, moon, 3 clouds, 3 city maps, 27 pooled rows and the runner; 33 of them draw an og.Sprite.
    for (const [s, rows] of [["Director", 33], ["Scroll", 36], ["Spawn", 33], ["Collide", 36], ["Dust", 36]] as const) {
      const sys = world.systems.find((x) => x.name === s);
      t.equal(sys?.matched, rows, `${s} sees its ${rows} actor rows`);
    }
    t.equal(world.systems.find((x) => x.name === "Animate")?.matched, 1, "Animate sees the runner row only");
    t.equal(world.systems.find((x) => x.name === "Jump")?.matched, 1, "Jump sees the runner row only (the one with og.PlatformerController)");
    t.equal(world.systems.find((x) => x.name === "Hud")?.matched, 1, "Hud sees the runner row only");
  },
});
