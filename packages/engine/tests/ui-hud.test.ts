import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { at, dump, engineFor } from "./support/ui.ts";

import { below, states } from "./support/hud.ts";

import "./support/hud.ts";

test("og.ui.objectiveTracker: steps tick, a completed objective leaves", () => {
  let completedAt: number | null = null;
  let kills = 2;
  const e = engineFor("HudTimer", () => {
    og.ui.objectiveTracker("Obj", {
      quests: [
        { key: "q1", title: "Clear the Camp", focused: true, completedAt, steps: [{ key: "s1", text: "Defeat bandits", count: kills, target: 5, done: kills >= 5 }, { key: "s2", text: "Find the chest", optional: true }] },
        { key: "q2", title: "Second", steps: [] },
        { key: "q3", title: "Third", steps: [] },
        { key: "q4", title: "Fourth", steps: [] },
      ],
    });
  });
  e.advance(3);
  let els = dump(e);
  expect(below(els, "ObjHud/Obj").filter((x) => x.path.split("/").length === 3).map((x) => x.id)).toEqual(["q1", "q2", "q3"]);
  expect(at(els, "ObjHud/Obj/q1/s1/Row/Count")?.text).toBe("2/5");
  expect([at(els, "ObjHud/Obj/q1/s1")?.role, states(at(els, "ObjHud/Obj/q1/s1")).step]).toEqual(["listitem", "open"]);
  kills = 5;
  e.advance(2);
  els = dump(e);
  expect(states(at(els, "ObjHud/Obj/q1/s1")).step).toBe("done");
  completedAt = og.ui.tick();
  e.advance(30);
  expect(at(dump(e), "ObjHud/Obj/q1/Complete")?.text).toBe("Complete");
  e.advance(200);
  expect(at(dump(e), "ObjHud/Obj/q1")).toBe(undefined);
});

