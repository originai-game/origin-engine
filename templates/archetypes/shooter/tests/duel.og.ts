// Two seats: the title offers a Duel beside the holdout (Duel presses
// Reload for the seats, so a room's peers start it together). In a duel the
// rounds hit the other trooper; downed, it is the shooter's frag and it
// comes back after a beat at the duel point farthest from the other. Alone
// the title offers no duel.
import { playtest } from "@origingame/engine/playtest";
import { DUEL, game, hold, PLAY, SEATS, tap, texts, value, type Hero } from "./lib.ts";

const two = SEATS > 1;

export default playtest({
  name: two ? "the Duel button, rounds that hit the other trooper, a frag and the respawn" : "alone, the title offers no Duel",
  commands: (two
    ? [
        "0.4:dump_ui:title.json",
        tap("Duel", 0.5),
        "0.8:get:Game:duel.json",
        "0.9:warp:-4:10:0",
        "0.9:warp:4:10:1",
        "1.0:aim:-90:-3:0",
        "1.0:aim:90:-3:1",
        "1.2:get:hero-1.Hero:full.json",
        "1.2:get:hero.og.Transform:shooter-at.json",
        hold("KeyF", 1.3, 2.0),
        "3.4:get:hero-1.Hero:downed.json",
        "3.4:get:hero.Hero:fragger.json",
        "5.8:get:hero-1.Hero:back.json",
        "5.8:get:hero-1.og.Transform:back-at.json",
        "5.9:quit",
      ]
    : ["0.4:dump_ui:title.json", "0.5:quit"]
  ).join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const words = texts(run, "title.json");
    if (!two) {
      t.ok(words.includes("Hold the line") && !words.includes("Duel"), "the title offers the holdout alone");
      return;
    }
    t.ok(words.includes("Duel"), "the title offers a Duel");
    const g = game(run, "duel.json");
    t.ok(g.phase === PLAY && g.mode === DUEL, "Duel starts a duel");
    t.ok(value<Hero>(run, "full.json").health === 100, "seat 1 starts at full health");
    const downed = value<Hero>(run, "downed.json");
    const fragger = value<Hero>(run, "fragger.json");
    t.ok(downed.down && downed.health === 0, "seat 0's rounds (F, its own keys alone, local or online) downed seat 1");
    t.equal(fragger.frags, 1, "and it is seat 0's frag");
    const back = value<Hero>(run, "back.json");
    t.ok(!back.down && back.health === 100, "after a beat seat 1 is back at full health");
    const [sx, , sz] = run.json<{ value: { position: number[] } }>("shooter-at.json").value.position as [number, number, number];
    const [bx, , bz] = run.json<{ value: { position: number[] } }>("back-at.json").value.position as [number, number, number];
    t.ok(Math.hypot(bx - sx, bz - sz) > 12, `at a duel point away from the other (${Math.hypot(bx - sx, bz - sz).toFixed(1)} m)`);
  },
});
