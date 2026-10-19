// The maze and its refusals: a row of hedges across the lawn leaves one
// gap, and the flow field takes the visitors through it; a hedge in the gap
// would close the path, so it is refused and costs nothing; so are a plant
// on a stone or on another plant, and one the purse cannot pay for.
import { playtest } from "@origingame/engine/playtest";
import { BLOCKED, game, POOR, TAKEN } from "./lib.ts";

const ROW = [1, 2, 3, 4, 5, 6].map((x) => `0.5:plant:hedge:${x}:2`).join(";");
// The first visitor's position, every tenth of a second while it crosses the hedge row.
const SAMPLES = Array.from({ length: 40 }, (_, k) => +(4.0 + k * 0.1).toFixed(1));

export default playtest({
  name: "hedges route the visitors through the gap; a closed path, a stone, a taken cell and an empty purse are refused",
  commands: [
    "0.3:begin",
    "0.4:gold:200",
    ROW,
    "0.6:get:Game:row.json",
    "0.7:plant:hedge:7:2",
    "0.8:get:Game:closed.json",
    "0.9:plant:thorn:2:3",
    "1.0:get:Game:stone.json",
    "1.1:plant:thorn:3:2",
    "1.2:get:Game:taken.json",
    "1.3:gold:20",
    "1.4:plant:frost:5:6",
    "1.5:get:Game:poor.json",
    "1.6:screenshot:maze.png",
    ...SAMPLES.map((t, k) => `${t}:get:visitor-0.og.Transform.position:p${k}.json`),
    "8.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const row = game(run, "row.json");
    t.equal(row.placed, 6, "six hedges across row 2, the gap at column 7");
    t.equal(row.gold, 200 - 60, "ten coins each");
    const closed = game(run, "closed.json");
    t.equal(closed.refusal, BLOCKED, "a hedge in the gap would close the path: refused");
    t.equal(closed.placed, 6, "and not set");
    t.equal(closed.gold, row.gold, "and not paid for");
    t.equal(game(run, "stone.json").refusal, TAKEN, "a stone takes no plant");
    t.equal(game(run, "taken.json").refusal, TAKEN, "a hedge's cell takes no other plant");
    const poor = game(run, "poor.json");
    t.equal(poor.refusal, POOR, "twenty coins do not buy a frost lily");
    t.equal(poor.refused, 4, "four refusals in all");
    // Row 2 spans y 9.5–10.5 in the world; the gap is column 7 (x 7–8).
    const at = SAMPLES.map((_, k) => run.json<{ value: number[] }>(`p${k}.json`).value);
    const crossing = at.filter((p) => p[1]! > 9.6 && p[1]! < 10.4);
    t.ok(crossing.length > 0, `the first visitor crossed the hedge row (${crossing.length} samples on it)`);
    t.ok(crossing.every((p) => p[0]! > 6.8), `through the gap only (x ${crossing.map((p) => p[0]!.toFixed(2)).join(", ")})`);
    t.ok(at.at(-1)![1]! < 9.5, `and walked on below it (y ${at.at(-1)![1]!.toFixed(2)})`);
  },
});
