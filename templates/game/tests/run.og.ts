// The run's loop, both ends: the trickster put in front of the guard is
// grabbed (a third of its Health, back to its start, a moment it blinks
// and cannot be grabbed again), and grabbed with no Health left the run is
// lost; Play again (S) starts a fresh run; then every power cell walked to
// and the pad stood on escapes, a win with its time kept as the best.
import { playtest } from "@origingame/engine/playtest";
import { CAUGHT, ESCAPED, OVER, PLAY, value } from "./lib.ts";
import type { Mission } from "./lib.ts";

/** Where the cells stand (src/play.ts CELL_AT). */
const CELLS = [[-7.5, 6.5], [7, -6.5], [-8.5, -6], [8, 6], [0.5, -9]] as const;

export default playtest({
  name: "a grab costs Health and a grab with none left loses; Play again, then every cell and the pad win",
  commands: [
    "0.3:key_down:KeyS",
    "0.4:key_up:KeyS",
    "0.6:health:0:40",
    "0.7:warp:0:-4:-1",
    "2.0:get:dropship.Mission:grabbed.json",
    "2.0:get:trickster.Hero:hero.json",
    "2.0:get:trickster.Health:hp.json",
    "3.5:warp:0:-4:-1",
    "6.0:get:dropship.Mission:caught.json",
    "7.5:key_down:KeyS",
    "7.6:key_up:KeyS",
    "7.8:get:dropship.Mission:again.json",
    "7.8:get:trickster.Health:hp-again.json",
    ...CELLS.map(([x, z], i) => `${(8 + i * 0.2).toFixed(1)}:warp:0:${x}:${z}`),
    "9.2:get:dropship.Mission:cells.json",
    "9.3:warp:0:0:0",
    "9.6:get:dropship.Mission:won.json",
    "9.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const grabbed = value<Mission>(run, "grabbed.json");
    t.equal(grabbed.catches, 1, "in front of the guard the trickster is grabbed");
    t.equal(grabbed.phase, PLAY, "and the run goes on");
    t.ok(value<{ hp: number }>(run, "hp.json").hp < 40, "the grab cost Health");
    t.ok(value<{ invulnerable: number }>(run, "hero.json").invulnerable > 0, "and the trickster blinks, ungrabbable for a moment");
    const caught = value<Mission>(run, "caught.json");
    t.equal(caught.phase, OVER, "grabbed with no Health left, the run is over");
    t.equal(caught.ending, CAUGHT, "lost: caught");
    const again = value<Mission>(run, "again.json");
    t.ok(again.phase === PLAY && again.runs === 2 && again.catches === 0, "S on the summary plays again, a fresh run");
    t.equal(value<{ hp: number; max: number }>(run, "hp-again.json").hp, 100, "with the trickster's Health whole");
    t.equal(value<Mission>(run, "cells.json").cells, 5, "every cell walked to is taken");
    const won = value<Mission>(run, "won.json");
    t.equal(won.ending, ESCAPED, "and the lit pad escapes: a win");
    t.ok(won.wins === 1, "counted");
  },
});
