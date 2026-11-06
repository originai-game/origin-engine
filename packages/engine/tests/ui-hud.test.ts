import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { at, drawsDuring, dump, engineFor, type El } from "./support/ui.ts";

import { below, states, kitEngine, LOADED_MS } from "./support/hud.ts";

import "./support/hud.ts";

test("og.ui.statBar: a hit leaves a ghost that holds, then drains; blocks, shield cells and the critical state", () => {
  let hp = 100;
  const e = engineFor("HudBar", () => {
    og.ui.statBar("HP", { value: hp, max: 100, segment: 25, shield: 60, shieldMax: 100, shieldCell: 25, shieldTier: "epic", label: "HP", showValue: true, anchor: "bottomLeft" });
  });
  e.advance(3);
  let els = dump(e);
  // Four blocks of 25 have three notches; 100 of shield in cells of 25 is four cells.
  expect(below(els, "HPHud/HP/Main/Track").filter((x) => x.id.startsWith("Block#")).length).toBe(3);
  expect(below(els, "HPHud/HP/Shield").filter((x) => x.id.startsWith("Cell#")).length).toBe(4);
  expect(at(els, "HPHud/HP/Main/Track/Ghost")).toBe(undefined);
  hp = 10;
  e.advance(2);
  els = dump(e);
  const bar = at(els, "HPHud/HP");
  expect([bar?.role, bar?.value, states(bar).level]).toEqual(["meter", "10 / 100", "critical"]);
  const ghost = at(els, "HPHud/HP/Main/Track/Ghost");
  const fill = at(els, "HPHud/HP/Main/Track/Fill");
  // The drop is instant; the ghost holds where the bar was.
  expect(ghost && fill && ghost.width > fill.width * 5).toBe(true);
  e.advance(20);
  expect((at(dump(e), "HPHud/HP/Main/Track/Ghost")?.width ?? 0) > (fill?.width ?? 0) * 5).toBe(true);
  e.advance(90);
  expect(at(dump(e), "HPHud/HP/Main/Track/Ghost")).toBe(undefined);
});

test("og.ui.abilityBar: the sweep's seconds, a charge left is ready, the ultimate's percent, the action's glyph on the device used last, a click answers its key", () => {
  let cd = 3.24;
  const e = engineFor("HudAbilities", () => {
    og.ui.abilityBar("Bar", {
      slots: [
        { key: "q", name: "Fireball", hotkey: "Q", cooldown: cd, cooldownMax: 8, cost: "40" },
        { key: "w", name: "Blink", hotkey: "W", maxCharges: 3, charges: 1, cooldown: 4, cooldownMax: 10, chargeProgress: 0.6 },
        { key: "e", name: "Ward", action: "ward", maxCharges: 2, charges: 0, cooldown: 2, cooldownMax: 6 },
        { key: "r", name: "Meteor", hotkey: "R", ultimate: 0.62 },
        { key: "t", name: "Locked", locked: true },
        { key: "h", name: "Hex", hotkey: "H", disabled: true, maxLevel: 0 },
        { key: "p", name: "Pyroblast", hotkey: "P", cost: "120", unaffordable: true, level: 1, maxLevel: 3 },
      ],
    });
  });
  // Ward's cap shows its action's binding, not a label.
  og.input.defineAction(e.world, { name: "ward", bindings: [{ control: "KeyF" }, { control: "pad.y" }] });
  e.advance(3);
  let els = dump(e);
  expect(at(els, "BarHud/Bar/e/Hotkey/Key")?.text).toBe("F");
  expect(at(els, "BarHud/Bar/q/Slot/Time")?.text).toBe("3.2");
  expect(states(at(els, "BarHud/Bar/q/Slot")).ready).toBe("cooling");
  expect(at(els, "BarHud/Bar/q/Slot")?.value).toBe("4s");
  expect(states(at(els, "BarHud/Bar/w/Slot")).ready).toBe("ready");
  expect(at(els, "BarHud/Bar/w/Slot/ChargeBadge/Charges")?.text).toBe("1");
  // One numeral per state: while cooling the cost is not drawn; ready, it is a corner figure.
  expect(at(els, "BarHud/Bar/q/Slot/Cost")).toBe(undefined);
  expect(at(els, "BarHud/Bar/p/Slot/Cost/Value")?.text).toBe("120");
  expect(states(at(els, "BarHud/Bar/h/Slot")).ready).toBe("disabled");
  expect(at(els, "BarHud/Bar/h/Slot")?.value).toBe("Unavailable");
  expect(at(els, "BarHud/Bar/h/Slot/Slash")).toBeDefined();
  // The hotkey is a label under its slot, never a cap on the art; every slot's label sits on one line,
  // whether or not the slot has rank pips.
  const slotQ = at(els, "BarHud/Bar/q/Slot")!;
  const keyQ = at(els, "BarHud/Bar/q/Hotkey")!;
  expect(keyQ.y).toBeGreaterThanOrEqual(slotQ.y + slotQ.height);
  expect(at(els, "BarHud/Bar/p/Hotkey")!.y).toBe(keyQ.y);
  expect(states(at(els, "BarHud/Bar/e/Slot")).ready).toBe("cooling");
  expect(at(els, "BarHud/Bar/r/Slot/Ultimate/Value")?.text).toBe("62%");
  // An ultimate is 1.15 times its neighbours, its bottom on their line and its hotkey on theirs.
  expect(at(els, "BarHud/Bar/r/Slot")!.width).toBeCloseTo(at(els, "BarHud/Bar/q/Slot")!.width * 1.15, 0);
  expect(at(els, "BarHud/Bar/r/Hotkey")!.y).toBe(at(els, "BarHud/Bar/q/Hotkey")!.y);
  const bottom = (k: string) => at(els, `BarHud/Bar/${k}/Slot`)!.y + at(els, `BarHud/Bar/${k}/Slot`)!.height;
  expect(bottom("r")).toBeCloseTo(bottom("q"), 0);
  expect(states(at(els, "BarHud/Bar/t/Slot")).ready).toBe("locked");
  expect(at(els, "BarHud/Bar/q/Hotkey/Key")?.text).toBe("Q");
  // A PlayStation pad's press: the cap follows it to the pad's triangle.
  e.probe.input({ kind: "gamepad", pad: 0, buttons: ["a"], device: "playstation" } as never);
  cd = 0;
  e.advance(2);
  els = dump(e);
  expect(at(els, "BarHud/Bar/e/Hotkey/Key")?.text).toBe("△");
  expect(at(els, "BarHud/Bar/q/Slot/Time")).toBe(undefined);
  expect([states(at(els, "BarHud/Bar/q/Slot")).ready, at(els, "BarHud/Bar/q/Slot")?.value]).toEqual(["ready", "Ready"]);
  // The tick it comes ready, it flashes.
  expect(at(els, "BarHud/Bar/q/Slot/Flash")).toBeDefined();
});

test("og.ui.statusGrid: buffs and debuffs in their rows, stacks and time, more than max folds into +n", () => {
  const e = engineFor("HudAuras", () => {
    og.ui.statusGrid("Auras", {
      max: 3,
      effects: [
        { key: "haste", name: "Haste", remaining: 12, duration: 30 },
        { key: "might", name: "Might", remaining: 0 },
        { key: "regen", name: "Regen", remaining: 600, duration: 900 },
        { key: "aegis", name: "Aegis", remaining: 4, duration: 10 },
        { key: "luck", name: "Luck", remaining: 30, duration: 60 },
        { key: "bleed", name: "Bleed", debuff: true, type: "bleed", stacks: 3, remaining: 3.5, duration: 10 },
      ],
      anchor: "topLeft",
    });
  });
  e.advance(3);
  const els = dump(e);
  const buffs = below(els, "AurasHud/Auras/Buffs").filter((x) => x.path.split("/").length === 4);
  expect(buffs.map((x) => x.id)).toEqual(["haste", "might", "More"]);
  expect(at(els, "AurasHud/Auras/Buffs/More/Tile/Count")?.text).toBe("+3");
  expect(at(els, "AurasHud/Auras/Debuffs/bleed/Tile/Stacks")?.text).toBe("3");
  expect(at(els, "AurasHud/Auras/Debuffs/bleed/Tile")?.name).toBe("Bleed, 3 stacks, 4 seconds");
  expect(states(at(els, "AurasHud/Auras/Debuffs/bleed/Tile")).time).toBe("expiring");
  expect(at(els, "AurasHud/Auras/Buffs/haste/Time")?.text).toBe("12");
  expect(at(els, "AurasHud/Auras/Buffs/might/Time")).toBe(undefined);
  expect(at(els, "AurasHud/Auras/Buffs")?.role).toBe("list");
});

test("og.ui.minimapView: out of range is dropped, an objective is held on the rim, a ping pulses", () => {
  const e = engineFor("HudMap", () => {
    og.ui.minimapView("Map", {
      size: 200, zoom: 0.2, centerX: 0.5, centerY: 0.5, zone: "Kings Canyon",
      markers: [
        { key: "me", kind: "player", x: 0.5, y: 0.5 },
        { key: "near", kind: "enemy", x: 0.55, y: 0.5 },
        { key: "far", kind: "enemy", x: 0.95, y: 0.5 },
        { key: "goal", kind: "objective", x: 0.5, y: 0.05 },
        { key: "ping", kind: "ping", x: 0.45, y: 0.55 },
      ],
    });
  });
  e.advance(3);
  const els = dump(e);
  const map = at(els, "MapHud/Map/Bezel/Map")!;
  expect(map.role).toBe("image");
  expect(at(els, "MapHud/Map/Bezel/Map/far")).toBe(undefined);
  const near = at(els, "MapHud/Map/Bezel/Map/near")!;
  expect(near.x + near.width / 2).toBeGreaterThan(map.x + map.width / 2 + 30);
  const goal = at(els, "MapHud/Map/Bezel/Map/goal")!;
  const cx = map.x + map.width / 2;
  const cy = map.y + map.height / 2;
  const gx = goal.x + goal.width / 2;
  const gy = goal.y + goal.height / 2;
  // Straight up, inside the round frame, near its rim.
  expect(Math.abs(gx - cx)).toBeLessThan(2);
  expect(Math.hypot(gx - cx, gy - cy)).toBeLessThan(map.width / 2);
  expect(Math.hypot(gx - cx, gy - cy)).toBeGreaterThan(map.width / 2 - 25);
  expect(at(els, "MapHud/Map/Bezel/pingPulse") ?? at(els, "MapHud/Map/Bezel/Map/pingPulse")).toBeDefined();
  expect(at(els, "MapHud/Map/ZonePlate/Zone")?.text).toBe("Kings Canyon");
  // North straddles the bezel above the map: its centre outside the map's glass, so it covers nothing on the map.
  const north = at(els, "MapHud/Map/Bezel/North")!;
  expect(Math.abs(north.x + north.width / 2 - cx)).toBeLessThan(2);
  expect(north.y + north.height / 2).toBeLessThan(map.y);
});

test("og.ui.compass: east of the heading is right of the notch, also under a right-to-left locale; behind, an objective holds the end", () => {
  for (const locale of ["en", "ar"]) {
    let set = false;
    const e = kitEngine(`HudCompass${locale}`, (w) => {
      if (!set) {
        og.i18n.setLocale(w, locale);
        set = true;
      }
      og.ui.compass("C", { heading: 0, pois: [{ key: "east", bearing: Math.PI / 4, kind: "poi", distance: 240 }, { key: "back", bearing: Math.PI * 0.95, kind: "objective" }] });
    });
    e.advance(3);
    const els = dump(e);
    const strip = at(els, "CHud/C/Strip")!;
    const mid = strip.x + strip.width / 2;
    const east = at(els, "CHud/C/Strip/east")!;
    expect(east.x + east.width / 2).toBeGreaterThan(mid + 50);
    expect(at(els, "CHud/C/Strip/east/Distance")?.text).toBe("240m");
    const back = at(els, "CHud/C/Strip/back")!;
    // Behind, to the right (it is clockwise of south): held at the right end.
    expect(back.x + back.width / 2).toBeGreaterThan(strip.x + strip.width - 40);
    expect(at(els, "CHud/C")?.value).toBe("0°, N");
  }
});

test("og.ui.hudTimer and objectiveTracker: tenths in the last ten seconds, steps tick, a completed objective leaves", () => {
  let secs = 74;
  let completedAt: number | null = null;
  let kills = 2;
  const e = engineFor("HudTimer", () => {
    og.ui.hudTimer("T", { seconds: secs, total: 90, label: "Round 2" });
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
  expect(at(els, "THud/T/Body/Time")?.text).toBe("1:14");
  expect(states(at(els, "THud/T")).time).toBe("normal");
  expect(below(els, "ObjHud/Obj").filter((x) => x.path.split("/").length === 3).map((x) => x.id)).toEqual(["q1", "q2", "q3"]);
  expect(at(els, "ObjHud/Obj/q1/s1/Row/Count")?.text).toBe("2/5");
  expect([at(els, "ObjHud/Obj/q1/s1")?.role, states(at(els, "ObjHud/Obj/q1/s1")).step]).toEqual(["listitem", "open"]);
  secs = 2.46;
  kills = 5;
  e.advance(2);
  els = dump(e);
  expect(at(els, "THud/T/Body/Time")?.text).toBe("2.5");
  expect(states(at(els, "THud/T")).time).toBe("critical");
  expect(states(at(els, "ObjHud/Obj/q1/s1")).step).toBe("done");
  completedAt = og.ui.tick();
  e.advance(30);
  expect(at(dump(e), "ObjHud/Obj/q1/Complete")?.text).toBe("Complete");
  e.advance(200);
  expect(at(dump(e), "ObjHud/Obj/q1")).toBe(undefined);
});


test("og.ui.statBar as pips: n of max lit with a gap between equal pips, its value read; an icon as tall as its label on a thin track", () => {
  // Wrong: pips drawn as one bar with notches (no gap), counted off by one, a part of a pip lit for a part of its
  // value, or pips of unequal widths that do not line up row over row; an icon sized by a thin track (an 8 px
  // reactor bar's 11 px speck), or by the bar's own label step beside a row's larger name.
  const e = engineFor("HudPips", () => {
    og.ui.anchor("Side", { anchor: "topLeft", direction: "column", width: { kind: "fixed", px: 300 } } as never, () => {
      og.ui.statBar("Shields", { value: 3.6, max: 4, style: "pips", kind: "mana", thickness: 12, showValue: true, valueText: "3 / 4", width: { kind: "grow" } });
      og.ui.statBar("Upgrade", { value: 4, max: 60, segment: 10, style: "pips", kind: "xp", thickness: 12, width: { kind: "grow" } });
      og.ui.statBar("Reactor", { value: 64, max: 100, kind: "mana", icon: "bolt", label: "Reactor", thickness: 8, width: { kind: "grow" } });
      // A row's own name outside the bar, at a larger step than the bar's label: the icon follows its line.
      og.ui.row("Row", { width: { kind: "grow" }, cross: "center" }, () => {
        og.ui.label("Name", "Reactor", { step: "xl", nowrap: true });
        og.ui.statBar("Bar", { value: 64, max: 100, kind: "mana", icon: "bolt", iconStep: "xl", thickness: 8, width: { kind: "grow" } });
      });
    });
  });
  e.advance(2);
  const draws = drawsDuring(() => e.advance(1));
  const els = dump(e);
  const pips = (id: string) => below(els, `Side/${id}/Main/Pips`).filter((x) => x.id.startsWith("Pip#"));
  const shields = pips("Shields");
  expect(shields.length).toBe(4);
  // A gap between pips, every pip one width.
  for (let i = 1; i < shields.length; i++) expect(shields[i]!.x - (shields[i - 1]!.x + shields[i - 1]!.width)).toBeGreaterThanOrEqual(2);
  expect(new Set(shields.map((x) => Math.round(x.width))).size).toBe(1);
  // 3.6 of 4 lights three whole pips: lit ones share one draw, the rest another.
  const fill = (id: string, i: number) => JSON.stringify(draws.get(`Side/${id}/Main/Pips/Pip#${i}`)?.[0]?.fills ?? null);
  expect([0, 1, 2, 3].map((i) => fill("Shields", i) === fill("Shields", 0))).toEqual([true, true, true, false]);
  expect(at(els, "Side/Shields")?.value).toBe("3 / 4");
  expect(at(els, "Side/Shields/Main/Value")?.text).toBe("3 / 4");
  // A segment per pip: 60 by tens is six pips, no Track and no blocks.
  expect(pips("Upgrade").length).toBe(6);
  expect(at(els, "Side/Upgrade/Main/Track")).toBe(undefined);
  // The reactor's mark stands as tall as its label's text, well past its 8 px track.
  const mark = at(els, "Side/Reactor/Main/Icon")!;
  const label = at(els, "Side/Reactor/Main/Label")!;
  const track = at(els, "Side/Reactor/Main/Track")!;
  expect(mark.height).toBeGreaterThanOrEqual(label.height - 0.5);
  expect(mark.height).toBeGreaterThan(track.height * 1.8);
  // Beside a row's xl name the mark is as tall as that name's line (it was the bar's own label step's, 17 px
  // beside a 24 px line).
  const rowMark = at(els, "Side/Row/Bar/Main/Icon")!;
  const name = at(els, "Side/Row/Name")!;
  expect(rowMark.height).toBeGreaterThanOrEqual(name.height - 0.5);
}, LOADED_MS);

test("og.ui.minimapView for racing: player-up by default on a track, north-up when asked, a checker start, sector ticks, rivals ringed, smaller on a sideways phone", () => {
  // Wrong: a track map left north-up, a rival east of a player heading east drawn to the right of it, rotate: false
  // ignored, a sector missing, or a sideways phone's map past 30 % of its height.
  let rotate: boolean | undefined;
  const track = [{ x: 0.2, y: 0.5 }, { x: 0.5, y: 0.2 }, { x: 0.8, y: 0.5 }, { x: 0.5, y: 0.8 }];
  const build = () => og.ui.minimapView("Map", {
    track, sectors: [0.33, 0.66], zoom: 1, heading: Math.PI / 2, ...(rotate === undefined ? {} : { rotate }), anchor: "topRight",
    markers: [{ key: "me", kind: "player", x: 0.5, y: 0.5 }, { key: "rival", kind: "enemy", x: 0.7, y: 0.5, ringed: true }],
  } as never);
  const e = engineFor("HudRaceMap", build);
  e.advance(3);
  const centre = (els: El[], p: string) => { const x = at(els, p)!; return { x: x.x + x.width / 2, y: x.y + x.height / 2 }; };
  let els = dump(e);
  const me = centre(els, "MapHud/Map/Bezel/Map/me");
  let rival = centre(els, "MapHud/Map/Bezel/Map/rival");
  // Heading east, player-up: the rival east of the player is ahead, above it.
  expect(rival.y).toBeLessThan(me.y - 10);
  expect(Math.abs(rival.x - me.x)).toBeLessThan(3);
  expect(at(els, "MapHud/Map/Bezel/Map/rivalRing")).toBeDefined();
  expect(els.filter((x) => /\/Sector#\d+$/.test(x.path)).length).toBe(2);
  expect(els.filter((x) => /\/Start(#\d+)?$/.test(x.path)).length).toBe(4);
  rotate = false;
  e.advance(2);
  els = dump(e);
  rival = centre(els, "MapHud/Map/Bezel/Map/rival");
  expect(rival.x).toBeGreaterThan(me.x + 10);
  const PHONE = { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 34, bottom: 0 }, coarsePointer: true, breakpoint: "phoneLandscape" };
  const p = engineFor("HudRaceMapPhone", build, PHONE);
  p.advance(3);
  expect(at(dump(p), "MapHud/Map/Bezel/Map")!.width).toBeLessThanOrEqual(Math.round(390 * 0.3));
});
