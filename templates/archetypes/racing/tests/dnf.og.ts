// Losing: a rival wins, and a seat still running when the race closes is
// out. The classification window is shortened to 2 s (Tuning.closeAfter,
// 30 s by default); the last rival is put on its last lap at the line
// while the seats stand on the grid. It finishes first, the race closes
// 2 s later with every seat out (DNF), the results say so with the rival
// on top (a result, not a game-over screen), the record counts a race and
// no win, and the results' Title button goes back to the title.
import { playtest } from "@origingame/engine/playtest";
import { car, carName, game, RESULTS, SEATS, sorted, tap, texts, TITLE, type PlatformDump } from "./lib.ts";

const RIVAL = 5;

export default playtest({
  name: "a rival wins, the seats still running are out, the results say so, Title goes back",
  commands: sorted([
    "0.2:set:Tuning.closeAfter=2",
    "0.3:race:go",
    `0.5:warp_car:${RIVAL}:300`,
    `0.6:lap:${RIVAL}:2`,
    `0.6:warp_car:${RIVAL}:-15:0:30`,
    `1.4:get:${carName(RIVAL)}.Car:winner.json`,
    "1.4:get:Game:first.json",
    "3.4:get:Game:closed.json",
    ...Array.from({ length: SEATS }, (_, s) => `3.4:get:car-${s}.Car:out${s}.json`),
    "6.0:get:Game:results.json",
    "6.0:dump_ui:results-ui.json",
    "6.0:dump_platform:platform.json",
    tap("Continue", 6.2),
    "6.6:get:Game:title.json",
    "6.7:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const w = car(run, "winner.json");
    t.ok(w.finish > 0 && w.place === 1, `the rival finished first (place ${w.place})`);
    t.ok(!game(run, "first.json").closed, "the race stays open for the rest");
    t.ok(game(run, "closed.json").closed, "and closes 2 s later");
    for (let s = 0; s < SEATS; s++) {
      const c = car(run, `out${s}.json`);
      t.ok(c.out && c.finish === 0 && c.place > 1, `seat ${s} is out (DNF, place ${c.place})`);
    }
    t.equal(game(run, "results.json").phase, RESULTS, "the results show");
    const words = texts(run, "results-ui.json");
    t.ok(words.includes("Did not finish") && words.includes("DNF"), "they say Did not finish, the seats' rows DNF");
    t.ok(!words.includes("Victory!"), "and claim no win");
    const p = run.json<PlatformDump>("platform.json");
    const rec = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "record").at(-1)?.args["value"] as { races: number; wins: number } | undefined;
    t.ok(rec !== undefined && rec.races === 1 && rec.wins === 0, `the record counts a race and no win (${JSON.stringify(rec)})`);
    t.equal(game(run, "title.json").phase, TITLE, "Title goes back to the title");
  },
});
