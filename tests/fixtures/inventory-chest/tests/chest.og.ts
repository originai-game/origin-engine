// The bag and the chest as `og test` plays them (src/main.ts): the walker
// picks up 80 items through the pickups' sensors, then the probe's commands
// refuse a move into a full bag (neither side changes), deposit, split and
// merge back, refuse a craft with no room and do it once a slot is free,
// and draw loot. The game's og.assert sites hold on every tick (nothing
// made or lost but by craft, no stack past its row's limit) and two runs
// end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("pickups fill the bag, moves are all or nothing, a craft needs room, loot replays", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["everything lying was picked up"],
    commands: [
      // Coins at x = -2 (tick 10), ore at 0 and 2: all taken, the bag full.
      "0.95:expect:state:picked==80",
      "0.95:expect:state:lying==0",
      "0.95:expect:state:bagCoin==60",
      "0.95:expect:state:bagOre==20",
      "0.95:expect:state:chestOre==4",
      "1:overfill",
      "1.05:expect:state:refused==1",
      "1.05:expect:state:chestSword==1",
      "1.05:expect:state:bagOre==20",
      "1.1:deposit",
      "1.15:expect:state:chestOre==10",
      "1.15:expect:state:bagOre==14",
      "1.2:split",
      "1.25:expect:state:split==3",
      "1.25:expect:state:merged==3",
      "1.25:expect:state:chestOre==10",
      "1.3:smelt",
      "1.35:expect:state:refused==2",
      "1.35:expect:state:bagIngot==0",
      "1.4:stash",
      "1.45:expect:state:chestOre==14",
      "1.5:smelt",
      "1.55:expect:state:bagIngot==1",
      "1.55:expect:state:bagOre==8",
      "1.6:roll",
      "1.65:expect:state:rolled==5",
      "1.65:expect:state:rollSum>=5",
      "1.65:expect:state:rollSum<=50",
      "1.7:quit",
    ].join(";"),
  });
  expect(run.hashes.length).toBe(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
