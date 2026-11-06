import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { at, drawsDuring, dump, engineFor } from "../../../tests/support/ui.ts";

import { sports } from "@origingame/engine/extensions/sports/index.ts";

import { below, states, LOADED_MS } from "../../../tests/support/hud.ts";

import "../../../tests/support/hud.ts";

test("sports.matchStats, goalTimeline, playerPlate, substitution and the scorebug's added time: shares, sides, stamina", () => {
  let stamina = 0.5;
  const e = engineFor("HudSports", () => {
    sports.scorebug("Bug", { home: { abbr: "NOR", score: 2 }, away: { abbr: "VAL", score: 1 }, clock: "67:42", added: "+3" });
    sports.matchStats("Poss", { title: "Possession", anchor: "topRight", stats: [{ key: "poss", home: 58, away: 42, homeText: "58%", awayText: "42%" }] });
    sports.matchStats("Stats", { header: true, home: { name: "Northgate" }, away: { name: "Valence" }, stats: [
      { key: "shots", label: "Shots", home: 14, away: 9 },
      { key: "fouls", label: "Fouls", home: 11, away: 14, worse: true },
      { key: "none", label: "Offsides", home: 0, away: 0 },
    ] });
    sports.goalTimeline("Goals", { events: [
      { key: "g1", team: 0, name: "J. Anderson", minute: "23'" },
      { key: "g2", team: 1, name: "L. Moreau", minute: "57'" },
      { key: "g3", team: 0, name: "M. Tanaka", minute: "68'" },
    ] });
    sports.playerPlate("Me", { number: "10", name: "Martin", stamina });
    sports.substitution("Sub", { off: { number: "7", name: "Reyes" }, on: { number: "19", name: "Kova" } });
  });
  e.advance(29);
  const draws = drawsDuring(() => e.advance(1));
  let els = dump(e);
  expect(at(els, "BugHud/Bug/Bar/Clock/Added/Text")?.text).toBe("+3");
  // A two-colour bar on every row: the home colour from the start, the away colour from the end, whichever side leads,
  // and fouls (more is worse) in the same two colours, never the danger hue; a 2 px gap where they meet.
  const paint = (p: string) => JSON.stringify(draws.get(p)?.[0]?.fills ?? null);
  for (const side of ["Home", "Away"]) {
    const rows = ["Stats/shots", "Stats/fouls", "Stats/none", "PossHud/Poss/poss"].map((r) => paint(`${r}/Mid/Bar/${side}`));
    expect(new Set(rows).size).toBe(1);
  }
  expect(paint("Stats/shots/Mid/Bar/Home")).not.toBe(paint("Stats/shots/Mid/Bar/Away"));
  const [fh, fa] = [at(els, "Stats/fouls/Mid/Bar/Home")!, at(els, "Stats/fouls/Mid/Bar/Away")!];
  expect(fa.x - (fh.x + fh.width)).toBeCloseTo(2, 0);
  // The bar splits at the home side's share, measured from the start: 58 of 100, not 58 against the larger.
  const share = (p: string) => {
    const h = at(els, `${p}/Mid/Bar/Home`)!.width;
    const a = at(els, `${p}/Mid/Bar/Away`)!.width;
    return h / (h + a);
  };
  expect(share("PossHud/Poss/poss")).toBeCloseTo(0.58, 1);
  expect(share("Stats/shots")).toBeCloseTo(14 / 23, 1);
  expect(share("Stats/none")).toBeCloseTo(0.5, 1);
  expect(at(els, "PossHud/Poss/poss/Home")?.text).toBe("58%");
  expect(at(els, "Stats/fouls/Away")?.text).toBe("14");
  // Every row's bar starts and ends at the same x, whatever its values print.
  expect(at(els, "Stats/shots/Mid/Bar")!.x).toBe(at(els, "Stats/fouls/Mid/Bar")!.x);
  // The scorers by side, in match order, the two columns either side of the rule.
  const homeKeys = below(els, "Goals/Home").filter((x) => x.path.split("/").length === 3).map((x) => x.id);
  expect(homeKeys).toEqual(["g1", "g3"]);
  expect(at(els, "Goals/Away/g2/Name")?.text).toBe("L. Moreau");
  const rule = at(els, "Goals/Rule")!;
  expect(at(els, "Goals/Home/g1")!.x + at(els, "Goals/Home/g1")!.width).toBeLessThanOrEqual(rule.x);
  expect(at(els, "Goals/Away/g2")!.x).toBeGreaterThanOrEqual(rule.x + rule.width);
  // Half of six segments: the third full, the fourth empty; low at a quarter.
  const seg = (i: number) => at(els, `MeHud/Me/Body/Info/Stamina/Seg#${i}/Fill`);
  expect(seg(2)!.width).toBeCloseTo(at(els, "MeHud/Me/Body/Info/Stamina/Seg#2")!.width, 0);
  expect(seg(3)).toBe(undefined);
  expect(states(at(els, "MeHud/Me")).stamina).toBe("fresh");
  // Off at the start, on at the end.
  expect(at(els, "SubHud/Sub/Plate/Off/Who/Line/Name")!.x).toBeLessThan(at(els, "SubHud/Sub/Plate/On/Who/Line/Name")!.x);
  stamina = 0.25;
  e.advance(1);
  els = dump(e);
  expect(states(at(els, "MeHud/Me")).stamina).toBe("low");
}, LOADED_MS);

test("sports.scorebug at full time and a radar's pitch: a final chip in place of the clock and never the added time; a pitch under the markers", () => {
  // Wrong: a finished match still showing 90:00 +3; a sports radar a black square with dots and no pitch.
  const e = engineFor("HudFinal", () => {
    sports.scorebug("Bug", { home: { abbr: "NOR", score: 2 }, away: { abbr: "VAL", score: 1 }, clock: "90:00", added: "+3", period: "2nd", final: "FT" });
    og.ui.minimapView("Radar", { shape: "square", zoom: 1, pitch: true, anchor: "bottomLeft", markers: [{ key: "me", kind: "player", x: 0.5, y: 0.5 }] });
  });
  e.advance(3);
  const els = dump(e);
  expect(at(els, "BugHud/Bug/Bar/Clock/Final/Text")?.text).toBe("FT");
  expect(at(els, "BugHud/Bug/Bar/Clock/Clock")).toBe(undefined);
  expect(at(els, "BugHud/Bug/Bar/Clock/Added")).toBe(undefined);
  expect(at(els, "BugHud/Bug")?.value).toContain("FT");
  // The pitch fills the map in place of the grid: touchlines, halfway line, both boxes and the centre circle.
  const map = at(els, "RadarHud/Radar/Bezel/Map")!;
  const pitch = at(els, "RadarHud/Radar/Bezel/Map/Pitch")!;
  expect([pitch.x, pitch.y, pitch.width, pitch.height]).toEqual([map.x, map.y, map.width, map.height]);
  expect(below(els, "RadarHud/Radar/Bezel/Map/Pitch").map((x) => x.id)).toEqual(["Touchlines", "Halfway", "BoxL", "BoxR", "Circle"]);
  expect(at(els, "RadarHud/Radar/Bezel/Map/Grid")).toBe(undefined);
}, LOADED_MS);
