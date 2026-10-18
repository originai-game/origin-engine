// Sounds: a selection, a refused swap, a swap, its clear and the cascade,
// the win and (in a second round of one move) the loss each start their own
// voice once (Sfx.plays on the sfx-<bit> entity); the mix is heard.
import { playtest } from "@origingame/engine/playtest";
import { CASCADE_BOARD, tap, tapCell } from "./lib.ts";

interface EntityDump {
  components: Record<string, Record<string, unknown>>;
}

const VOICES = [
  [1, "a selection", 2],
  [2, "a swap", 1],
  [4, "a refused swap", 1],
  [8, "a clear", 1],
  [16, "a cascade", 1],
  [32, "the win", 1],
  [64, "the loss", 1],
] as const;

export default playtest({
  name: "selections, swaps, clears, a cascade, the win and the loss each play their sound",
  commands: [
    "0.1:set:board.Tuning.target=30",
    "0.45:listen:title.wav:0.3",
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    tapCell(1.0, 5, 0),
    tapCell(1.2, 6, 0),
    tapCell(2.0, 2, 0),
    tapCell(2.2, 2, 1),
    "2.5:listen:clear.wav:0.3",
    "4.3:set:board.Tuning.target=10000",
    "4.3:set:board.Tuning.moves=1",
    tap(4.5),
    `4.8:deal:${CASCADE_BOARD}`,
    "5.0:swap:2,0:2,1",
    ...VOICES.map(([bit]) => `7.2:dump_entity:sfx-${bit}:sfx-${bit}.json`),
    "7.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.wav("title.wav").peak < 0.001, `the title is silent (peak ${run.wav("title.wav").peak})`);
    t.ok(run.wav("clear.wav").peak > 0.05, `the swap and its clear are heard (peak ${run.wav("clear.wav").peak})`);
    for (const [bit, what, least] of VOICES) {
      const plays = Number(run.json<EntityDump>(`sfx-${bit}.json`).components["Sfx"]?.["plays"] ?? 0);
      t.ok(plays >= least, `${what} played its sound (${plays} time(s), at least ${least})`);
    }
  },
});
