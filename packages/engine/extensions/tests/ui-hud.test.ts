import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { at, dump, engineFor } from "../../tests/support/ui.ts";

import { racing } from "@origingame/engine/extensions/racing/index.ts";

import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

import { shooter } from "@origingame/engine/extensions/shooter/index.ts";

import { strategy } from "@origingame/engine/extensions/strategy/index.ts";

import { flight } from "@origingame/engine/extensions/flight/index.ts";

import { rhythm } from "@origingame/engine/extensions/rhythm/index.ts";

import { sports } from "@origingame/engine/extensions/sports/index.ts";

import { below, states, LOADED_MS, VIEWPORTS } from "../../tests/support/hud.ts";

import "../../tests/support/hud.ts";

test("shooter.killfeed, resourceBar and scoreboard: newest first within max and time, a cap that warns, rows in order", () => {
  const entries = Array.from({ length: 7 }, (_, i) => ({ key: `k${i}`, killer: `P${i}`, victim: `V${i}`, weaponText: "R-301", at: 0, killerTeam: 0, victimTeam: 1, local: i === 6 }));
  const e = engineFor("HudLists", () => {
    shooter.killfeed("Feed", { entries, max: 4, durationMs: 1000 });
    strategy.resourceBar("Res", { items: [{ key: "min", name: "Minerals", amount: 1250, rate: 12.5 }, { key: "sup", name: "Supply", amount: 190, capacity: 200, rate: 0 }] });
    shooter.scoreboard("Board", {
      title: "Bind", columns: [{ key: "k", label: "K" }, { key: "d", label: "D" }],
      teams: [{ team: 0, name: "Attackers", score: "7", rows: [{ key: "a", name: "Ana", values: ["12", "4"], local: true, ping: 32 }, { key: "b", name: "Bo", values: ["3", "9"], dead: true, ping: 240 }] }],
    });
  });
  e.advance(3);
  let els = dump(e);
  const lines = below(els, "FeedHud/Feed").filter((x) => x.path.split("/").length === 3);
  expect(lines.map((x) => x.id)).toEqual(["k6", "k5", "k4", "k3"]);
  expect(at(els, "FeedHud/Feed")?.role).toBe("log");
  expect(at(els, "ResHud/Res/sup/Count/Amount")?.text).toBe("190");
  expect(at(els, "ResHud/Res/sup/Count/Cap")?.text).toBe(" / 200");
  expect(at(els, "ResHud/Res/min/Rate/Text")?.text).toBe("12.5/s");
  expect(at(els, "ResHud/Res/min/Rate/Mark")).toBeDefined();
  expect(at(els, "BoardHud/Board/Team#0/a/Value#0")?.text).toBe("12");
  expect(at(els, "BoardHud/Board/Team#0/b")?.value).toBe("3, 9");
  e.advance(70);
  els = dump(e);
  expect(below(els, "FeedHud/Feed").length).toBe(0);
});

test("rhythm.comboCounter, judgement, turnOrder, scorebug, speedometer, flightHud: what each reads", () => {
  let count = 9;
  let judgeAt = 0;
  const e = engineFor("HudMisc", () => {
    rhythm.comboCounter("Combo", { count, multiplier: 4 });
    rhythm.judgement("Judge", { last: { text: "PERFECT", tier: 0, offsetMs: -12, at: judgeAt } });
    rpg.turnOrder("Turns", { units: [{ key: "hero", name: "Hero", current: true }, { key: "imp", name: "Imp", team: 1 }, { key: "mage", name: "Mage", atb: 0.5 }] });
    sports.scorebug("Bug", { home: { abbr: "LAL", score: 98 }, away: { abbr: "BOS", score: 101, possession: true }, clock: "2:41", period: "Q4" });
    racing.speedometer("Speedo", { speed: 212, maxSpeed: 300, rpm: 0.92, gear: "5" });
    flight.flightHud("Flight", { speed: 420, altitude: 12500, heading: Math.PI / 2, pitch: 0.1, roll: 0.3 });
  });
  e.advance(3);
  let els = dump(e);
  expect(at(els, "ComboHud/Combo/Count")?.text).toBe("9");
  expect(at(els, "JudgeHud/Judge/Word")?.text).toBe("PERFECT");
  expect(at(els, "JudgeHud/Judge/Timing")?.text).toBe("Early 12ms");
  expect(below(els, "TurnsHud/Turns").filter((x) => x.path.split("/").length === 3).map((x) => x.id)).toEqual(["hero", "imp", "mage"]);
  expect(at(els, "TurnsHud/Turns/hero/Now/Text")?.text).toBe("Now");
  expect(at(els, "BugHud/Bug/Bar/Away/ScoreBox/Score")?.text).toBe("101");
  expect(at(els, "SpeedoHud/Speedo")?.value).toBe("212 km/h, gear 5");
  expect(states(at(els, "SpeedoHud/Speedo")).rev).toBe("redline");
  expect(at(els, "FlightHud/Flight/SpeedBox")?.value).toBe("420");
  expect(at(els, "FlightHud/Flight/HeadingBox")?.value).toBe("090");
  count = 10;
  e.advance(2);
  els = dump(e);
  expect(states(at(els, "ComboHud/Combo")).combo).toBe("milestone");
  count = 0;
  judgeAt = -1000;
  e.advance(2);
  els = dump(e);
  expect(states(at(els, "ComboHud/Combo")).combo).toBe("broken");
  expect(at(els, "JudgeHud/Judge/Word")).toBe(undefined);
  // A broken combo shows for 600 ms: 36 ticks after it broke.
  e.advance(35);
  expect(states(at(dump(e), "ComboHud/Combo")).combo).toBe("idle");
}, LOADED_MS);

test("strategy.wavePreview and laneKeys: the waves past max fold into dots, the current lit; caps under their lanes, a held one down", () => {
  const e = engineFor("HudGenres", () => {
    strategy.wavePreview("Waves", { current: 8, max: 4, waves: Array.from({ length: 6 }, (_, i) => ({ key: `w${i + 7}`, number: i + 7, name: "Crawlers", count: (i + 1) * 3, boss: i === 4 })) });
    rhythm.laneKeys("Lanes", { y: 600, lanes: [{ key: "a", hotkey: "D", x: 500 }, { key: "b", hotkey: "F", x: 640 }, { key: "c", hotkey: "J", x: 780 }], pressed: ["b"] });
  });
  e.advance(4);
  const els = dump(e);
  // Four rows at most: three waves and the dots for the other three.
  const rows = below(els, "WavesHud/Waves").filter((x) => x.path.split("/").length === 3 && x.id.startsWith("w")).map((x) => x.id);
  expect(rows).toEqual(["w7", "w8", "w9"]);
  expect(at(els, "WavesHud/Waves/More")).toBeDefined();
  expect(at(els, "WavesHud/Waves/w7/Count")?.text).toBe("×3");
  expect(at(els, "WavesHud/Waves/w8")?.role).toBe("text");
  // Each cap centred on its lane, under the hit line; the held one pressed lower than its neighbours.
  for (const [k, x] of [["a", 500], ["b", 640], ["c", 780]] as const) {
    const cap = at(els, `LanesHud/Lanes/${k}`)!;
    expect(cap.x + cap.width / 2).toBeCloseTo(x, 0);
    expect(cap.y).toBeGreaterThan(600);
  }
  expect(at(els, "LanesHud/Lanes/b")!.y).toBeGreaterThan(at(els, "LanesHud/Lanes/a")!.y);
  expect(at(els, "LanesHud/Lanes/c/Glyph")?.text).toBe("J");
}, LOADED_MS);

test("components anchored at one point stack in the order built instead of drawing over each other", () => {
  const e = engineFor("HudStack", () => {
    og.ui.compass("Compass", { heading: 0 });
    og.ui.hudTimer("Round", { seconds: 42 });
    strategy.resourceBar("Res", { items: [{ key: "g", name: "Gold", amount: 10 }] });
    og.ui.objectiveTracker("Obj", { quests: [{ key: "q", title: "Hold", steps: [{ key: "s", text: "Hold the line" }] }] });
    rhythm.comboCounter("Combo", { count: 12 });
    shooter.ammoCounter("Ammo", { magazine: 12, magazineMax: 30 });
    racing.speedometer("Speed", { speed: 120 });
  });
  e.advance(3);
  const els = dump(e);
  const overlap = (a: string, b: string): boolean => {
    const p = at(els, a)!;
    const q = at(els, b)!;
    return Math.min(p.y + p.height, q.y + q.height) - Math.max(p.y, q.y) > 0.5 && Math.min(p.x + p.width, q.x + q.width) - Math.max(p.x, q.x) > 0.5;
  };
  expect([overlap("CompassHud", "RoundHud"), overlap("RoundHud", "ResHud"), overlap("CompassHud", "ResHud")]).toEqual([false, false, false]);
  expect(at(els, "CompassHud")!.y).toBeLessThan(at(els, "RoundHud")!.y);
  // At a side's middle: the first centred, the next under it.
  expect(overlap("ObjHud", "ComboHud")).toBe(false);
  // At a bottom corner: the second rises over the first.
  expect(overlap("AmmoHud", "SpeedHud")).toBe(false);
  expect(at(els, "SpeedHud")!.y).toBeLessThan(at(els, "AmmoHud")!.y);
});

test("every corner HUD piece stays inside the safe area at every breakpoint", () => {
  for (const [name, vp] of Object.entries(VIEWPORTS)) {
    const e = engineFor(`HudSafe${name}`, () => {
      og.ui.statBar("HP", { value: 64, label: "HP", showValue: true, anchor: "bottomLeft" });
      og.ui.abilityBar("Bar", { slots: ["q", "w", "e", "r", "d", "f", "g", "h"].map((key) => ({ key, hotkey: key.toUpperCase(), cooldown: 2, cooldownMax: 5 })) });
      shooter.ammoCounter("Ammo", { magazine: 18, magazineMax: 30, reserve: 120 });
      og.ui.minimapView("Map", { zone: "Docks" });
      shooter.killfeed("Feed", { entries: [{ key: "a", killer: "Wraith", victim: "Bloodhound", weaponText: "R-99", at: 0 }] });
      og.ui.compass("Compass", { heading: 1 });
      rpg.unitFrame("Me", { name: "Aria", level: "12", health: 80, resource: 40, anchor: "topLeft" });
      og.ui.objectiveTracker("Obj", { quests: [{ key: "q", title: "Reach the extraction point before the storm", steps: [{ key: "s", text: "Hold the zone", progress: 0.4 }] }] });
    }, vp);
    e.advance(3);
    const els = dump(e);
    const sa = vp.safeArea;
    for (const root of ["HPHud", "BarHud", "AmmoHud", "MapHud", "FeedHud", "CompassHud", "MeHud", "ObjHud"]) {
      const el = at(els, root);
      expect(el, `${name} ${root}`).toBeDefined();
      const r = el!;
      const inside = r.x >= sa.left - 0.5 && r.y >= sa.top - 0.5 && r.x + r.width <= vp.width - sa.right + 0.5 && r.y + r.height <= vp.height - sa.bottom + 0.5;
      expect(inside, `${name} ${root} at ${r.x},${r.y} ${r.width}×${r.height}`).toBe(true);
    }
  }
});
