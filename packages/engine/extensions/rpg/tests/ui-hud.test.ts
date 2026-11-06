import { expect, test } from "bun:test";

import { defineSystem, og } from "@origingame/engine";

import type { DrawDesc } from "@origingame/engine";

import { kit as uiKit } from "../../../src/ui-kit/bind.ts";

import { native } from "../../../src/node/index.ts";

import { checkFrame, type UiFrameSnapshot } from "../../../src/node/ui-check.ts";

import { createEngine, type GameToken } from "../../../src/runtime.ts";

import { at, drawsDuring, dump, engineFor, type El } from "../../../tests/support/ui.ts";

import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

import { states, below, LOADED_MS, kit, DESKTOP_SAFE } from "../../../tests/support/hud.ts";

import "../../../tests/support/hud.ts";

test("rpg.unitFrame, raidFrames and castBar: health and states read, raids in columns of five, an interrupted cast leaves", () => {
  let state: "casting" | "interrupted" = "casting";
  let endedAt = 0;
  let health = 30;
  const e = engineFor("HudFrames", () => {
    rpg.unitFrame("Target", { name: "Grull the Ogre", level: "62", health, hostile: true, unitClass: "elite", reverse: true, targeted: true, anchor: "top" });
    rpg.raidFrames("Raid", { members: Array.from({ length: 12 }, (_, i) => ({ key: `m${i}`, name: `Member ${i}`, health: 10 * i, dead: i === 0, role: i % 3 === 0 ? "tank" as const : "damage" as const })) });
    rpg.castBar("Cast", { name: "Pyroblast", progress: 0.4, remaining: 2.1, state, stateAt: endedAt, anchor: "bottom" });
    rpg.unitFrame("Boss", { name: "Grull the Devourer", level: "??", health: 800_000, healthMax: 1_250_000, shield: 150_000, anchor: "topRight" });
  });
  e.advance(3);
  let els = dump(e);
  // A boss's health is shortened as shipped frames write it, and a frame with no art shows no initials.
  // Health is the dominant bar: its numerals sit inside its end, so its track spans the frame's whole bar column.
  const value = at(els, "BossHud/Boss/Bars/Health/Main/Track/Value/Text");
  const track = at(els, "BossHud/Boss/Bars/Health/Main/Track")!;
  expect(value?.text).toBe("800K / 1.25M");
  expect(track.width).toBeCloseTo(at(els, "BossHud/Boss/Bars")!.width, 0);
  expect(value!.x + value!.width).toBeLessThanOrEqual(track.x + track.width + 0.5);
  // The absorb lies on the health bar, not on a row of its own.
  expect(at(els, "BossHud/Boss/Bars/Health/Main/Track/Absorb")).toBeDefined();
  expect(at(els, "BossHud/Boss/Bars/Health/Shield")).toBe(undefined);
  // The level badge is at most 40 % of the portrait, at 100 % text.
  expect(at(els, "BossHud/Boss/Avatar/Level")!.height).toBeLessThanOrEqual(at(els, "BossHud/Boss/Avatar/Portrait")!.width * 0.4 + 0.5);
  expect(els.some((x) => x.path.startsWith("BossHud/Boss/") && x.id === "Initials")).toBe(false);
  // The cast's name and seconds on their own line over the bar.
  expect([at(els, "CastHud/Cast/Body/Head/Name")?.text, at(els, "CastHud/Cast/Body/Head/Time")?.text]).toEqual(["Pyroblast", "2.1"]);
  const target = at(els, "TargetHud/Target");
  expect([target?.role, target?.value, states(target).selection, states(target).hurt]).toEqual(["button", "Level 62, 30% health", "targeted", "none"]);
  // A hit: the health falls, the frame shows it for a moment.
  health = 18;
  e.advance(2);
  expect(states(at(dump(e), "TargetHud/Target")).hurt).toBe("hit");
  expect(at(dump(e), "TargetHud/Target/Avatar/Portrait/Hurt")).toBeDefined();
  // The hit shows for 450 ms: 27 ticks.
  e.advance(28);
  expect(states(at(dump(e), "TargetHud/Target")).hurt).toBe("none");
  // The portrait at the end when reversed.
  const portrait = at(els, "TargetHud/Target/Avatar/Portrait");
  // The level badge: a disc seated on the portrait's rim, below its middle, on the side toward the bars.
  const badge = at(els, "TargetHud/Target/Avatar/Level")!;
  const [bx, by] = [badge.x + badge.width / 2, badge.y + badge.height / 2];
  const [cx, cy, pr] = [portrait!.x + portrait!.width / 2, portrait!.y + portrait!.height / 2, portrait!.width / 2];
  expect(badge.width).toBeCloseTo(badge.height, 0);
  expect(Math.hypot(bx - cx, by - cy)).toBeCloseTo(pr, -0.3);
  expect(by).toBeGreaterThan(cy);
  expect(bx).toBeLessThan(cx);
  const bars = at(els, "TargetHud/Target/Bars");
  expect(portrait && bars && portrait.x > bars.x).toBe(true);
  const groupsOf = below(els, "RaidHud/Raid").filter((x) => /^Group#\d+$/.test(x.id));
  expect(groupsOf.length).toBe(3);
  expect(below(els, "RaidHud/Raid/Group#2").filter((x) => x.path.split("/").length === 4).map((x) => x.id)).toEqual(["m10", "m11"]);
  expect(at(els, "RaidHud/Raid/Group#0/m0")?.value).toBe("Dead");
  expect(at(els, "CastHud/Cast")?.role).toBe("progressbar");
  state = "interrupted";
  endedAt = og.ui.tick();
  e.advance(2);
  els = dump(e);
  expect(at(els, "CastHud/Cast/Body/Head/Name")?.text).toBe("Interrupted");
  expect(states(at(els, "CastHud/Cast")).cast).toBe("interrupted");
  // An interrupted cast holds 700 ms and fades 300: gone 60 ticks after it ended.
  e.advance(59);
  expect(at(dump(e), "CastHud/Cast")).toBe(undefined);
}, LOADED_MS);

test("rpg.raidFrames: health a bar along a neutral cell's foot under the name; out of range fades the bar, never the name's outline", () => {
  // Wrong: a cell flooded with its health's green (the grid the loudest thing on screen); the name drawn over the
  // fill; an out-of-range cell faded as a whole element, which fades its name's outline with the fill.
  const e = engineFor("HudRaid", () => {
    rpg.raidFrames("Raid", { anchor: "left", members: [
      { key: "full", name: "Aldric", health: 100 },
      { key: "far", name: "Brenna", health: 80, outOfRange: true },
      { key: "dead", name: "Corvin", health: 0, dead: true },
    ] });
  });
  e.advance(2);
  const opacities: number[] = [];
  const k = uiKit as { props: (p: Record<string, unknown>) => void };
  const was = k.props;
  k.props = (p) => {
    if (typeof p.opacity === "number") opacities.push(p.opacity);
    was(p);
  };
  let draws: Map<string, DrawDesc[]>;
  try {
    draws = drawsDuring(() => e.advance(1));
  } finally {
    k.props = was;
  }
  const els = dump(e);
  const cell = at(els, "RaidHud/Raid/Group#0/full")!;
  const fill = at(els, "RaidHud/Raid/Group#0/full/Bar/Fill")!;
  const name = at(els, "RaidHud/Raid/Group#0/full/Label/Name")!;
  // Full health is a bar along the foot, not a fill across the cell; the name's line sits over it.
  expect(fill.height).toBeLessThan(cell.height * 0.4);
  expect(fill.y + fill.height).toBeLessThanOrEqual(cell.y + cell.height);
  expect(name.y + name.height).toBeLessThanOrEqual(fill.y + 0.5);
  // The cell's ground is the HUD's neutral token, not a hue.
  expect(JSON.stringify(draws.get("RaidHud/Raid/Group#0/full")?.[0]?.fills?.[0])).toContain("\"hud\"");
  // Dead keeps its dark cell with no bar.
  expect(at(els, "RaidHud/Raid/Group#0/dead/Bar")).toBe(undefined);
  // Out of range: the bar's paint is faded; no element of the raid is faded as a whole.
  const farFill = JSON.stringify(draws.get("RaidHud/Raid/Group#0/far/Bar/Fill")?.[0]?.fills ?? null);
  expect(farFill).toContain("\"alpha\":0.45");
  expect(opacities.filter((o) => o < 1)).toEqual([]);
}, LOADED_MS);

test("rpg.turnOrder as a list: rows top to bottom, the acting one first and selected, an enemy's stripe on the start edge, allies lined up, mirrored right to left", () => {
  // Wrong: the list drawn as the strip (no rows, no names), numbers on the rows, an ally's portrait set in where
  // an enemy's stripe would be, the stripe left of the portrait right to left, or a screen reader told neither
  // the side nor who acts.
  const units = [
    { key: "valiant", name: "Valiant", current: true },
    { key: "raider", name: "Raider", team: 1 },
    { key: "aegis", name: "Aegis" },
    { key: "wasp", name: "Wasp", team: 2, dead: true },
  ];
  const build = () => og.ui.anchor("Side", { anchor: "topLeft", width: { kind: "fixed", px: 320 } } as never, () => rpg.turnOrder("Turns", { style: "list", units, nowLabel: "Acting", anchor: null }));
  const read = (rtl: boolean) => {
    const sys = defineSystem(`HudTurnList${rtl ? "Rtl" : ""}`, {}, () => build());
    const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, rtl ? { root: kit } : {});
    if (!r.ok) throw new Error(r.reason);
    if (rtl) {
      const loaded = og.i18n.load(r.value.world, "strings/ar.ogt");
      if (!loaded.ok) throw new Error(loaded.reason);
      og.i18n.setLocale(r.value.world, "ar-EG");
    }
    r.value.advance(3);
    const draws = drawsDuring(() => r.value.advance(1));
    return { els: dump(r.value) as Array<El & { states: string[] }>, draws };
  };
  const { els, draws } = read(false);
  const rows = units.map((un) => at(els, `Side/Turns/${un.key}`)! as El & { states: string[] });
  // Top to bottom in turn order, each a whole row of the list's width.
  expect(rows.every((x) => x !== undefined)).toBe(true);
  for (let i = 1; i < rows.length; i++) expect(rows[i]!.y).toBeGreaterThanOrEqual(rows[i - 1]!.y + rows[i - 1]!.height - 0.5);
  expect(rows.map((x) => Math.round(x.width))).toEqual([320, 320, 320, 320]);
  // A screen reader hears the side and who acts; only the acting one is selected.
  expect(rows.map((x) => [x.role, x.name])).toEqual([["listitem", "Valiant, Acting"], ["listitem", "Raider, Hostile"], ["listitem", "Aegis"], ["listitem", "Wasp, Hostile"]]);
  expect(rows.map((x) => x.states.includes("selected"))).toEqual([true, false, false, false]);
  expect(at(els, "Side/Turns")?.role).toBe("list");
  // Names, no numbers.
  expect(at(els, "Side/Turns/raider/Text/Name")?.text).toBe("Raider");
  expect(els.filter((x) => x.path.startsWith("Side/Turns/") && /^\d+$/.test(x.text))).toEqual([]);
  // The stripe starts every row at its start edge; every portrait starts on one line, ally or enemy.
  for (const un of units) expect(Math.round(at(els, `Side/Turns/${un.key}/Stripe`)!.x)).toBe(Math.round(at(els, `Side/Turns/${un.key}`)!.x));
  const portraits = units.map((un) => Math.round(at(els, `Side/Turns/${un.key}/Portrait`)!.x));
  expect(new Set(portraits).size).toBe(1);
  // Drawn: the enemies' stripes in their side's ink (danger for team 1, another for team 2), the allies' none;
  // the acting row on its selected ground, the others bare.
  const fill = (p: string) => JSON.stringify(draws.get(p)?.[0]?.fills ?? null);
  expect(fill("Side/Turns/valiant/Stripe")).toBe("null");
  expect(fill("Side/Turns/aegis/Stripe")).toBe("null");
  expect(fill("Side/Turns/raider/Stripe")).not.toBe("null");
  expect(fill("Side/Turns/wasp/Stripe")).not.toBe(fill("Side/Turns/raider/Stripe"));
  expect(fill("Side/Turns/valiant")).toContain("controlStrong");
  expect(fill("Side/Turns/aegis")).toBe("null");
  // Right to left the row mirrors: the stripe at the right edge, the portrait inside it.
  const r = read(true).els;
  const row = at(r, "Side/Turns/raider")!;
  const stripe = at(r, "Side/Turns/raider/Stripe")!;
  expect(Math.round(stripe.x + stripe.width)).toBe(Math.round(row.x + row.width));
  expect(at(r, "Side/Turns/raider/Portrait")!.x).toBeLessThan(stripe.x);
}, LOADED_MS);
