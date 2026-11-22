// The hero as `og test` plays it (src/main.ts): two dashes 12 ticks apart
// (a press inside the lock is not taken), their charges back one at a
// time; a fireball that spends mana, recharges and is refused once mana is
// under its cost; poison that stacks to 3, a slow refreshed, a stronger
// shield replacing a weaker one. The game's og.assert sites hold the stats
// to the statuses on every tick and two runs end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.02).toFixed(2)}:key_up:${key}`;

test("charges return one by one, costs are paid, statuses stack by their rules", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["both dashes spent"],
    commands: [
      "0.4:expect:state:dashCharges==2",
      tap(0.5, "KeyK"), // dash 1 (tick 30)
      tap(0.6, "KeyK"), // inside its 12-tick lock: not taken
      "0.68:expect:state:dashUses==1",
      tap(0.75, "KeyK"), // dash 2 (tick 45)
      "0.8:expect:state:dashUses==2",
      "0.8:expect:state:dashCharges==0",
      // The first charge is back 60 ticks after dash 1 (tick 90), the second 60 after that.
      "1.45:expect:state:dashCharges==0",
      "1.55:expect:state:dashCharges==1",
      "2.45:expect:state:dashCharges==1",
      "2.55:expect:state:dashCharges==2",
      tap(1, "KeyJ"), // fireball (tick 60): 50 - 20 mana
      tap(1.1, "KeyJ"), // no charge: not taken
      "1.15:expect:state:fireUses==1",
      "1.15:expect:state:mana==30",
      tap(2.6, "KeyJ"), // recharged at tick 150
      "2.65:expect:state:mana==10",
      tap(4.2, "KeyJ"), // recharged, but 10 mana is under its cost
      "4.25:expect:state:fireUses==2",
      "4.25:expect:state:mana==10",
      // Shields: 5, then a weaker 3 (dropped), then a stronger 8 (replaces it).
      "1:shield:5",
      "1.1:shield:3",
      "1.15:expect:state:shield==5",
      "1.2:shield:8",
      "1.25:expect:state:shield==8",
      // Poison four times: 3 stacks at most, -2 armour each.
      "1.2:poison",
      "1.3:poison",
      "1.4:poison",
      "1.5:poison",
      "1.6:expect:state:poisonStacks==3",
      "1.6:expect:state:armour==12",
      // A slow at 2 s, refreshed at 2.5 s: it ends 60 ticks after the refresh.
      "2:slow",
      "2.05:expect:state:speed==3",
      "2.5:slow",
      "3.2:expect:state:slowed==1",
      "3.6:expect:state:slowed==0",
      "3.6:expect:state:speed==6",
      // The poison's last refresh was at 1.5 s: gone 120 ticks later.
      "3.45:expect:state:poisonStacks==3",
      "3.6:expect:state:poisonStacks==0",
      "3.6:expect:state:armour==18",
      "4.3:quit",
    ]
      .join(";")
      .split(";")
      // In time order, steps at one time as written.
      .map((step, i) => ({ step, i, at: Number(step.split(":")[0]) }))
      .sort((a, b) => a.at - b.at || a.i - b.i)
      .map((s) => s.step)
      .join(";"),
  });
  expect(run.hashes.length).toBe(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
