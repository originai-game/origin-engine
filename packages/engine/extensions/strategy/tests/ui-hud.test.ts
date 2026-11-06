import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { at, drawsDuring, dump, engineFor } from "../../../tests/support/ui.ts";

import { strategy } from "@origingame/engine/extensions/strategy/index.ts";

import { below, states, LOADED_MS, VIEWPORTS } from "../../../tests/support/hud.ts";

import "../../../tests/support/hud.ts";

test("strategy.resourceBar in the flow folds to the box it is laid in, not to the screen", () => {
  // Wrong: the fold measured against the screen alone, so a bar in a column narrower than its one line runs past it.
  const items = [
    { key: "gold", name: "Gold", icon: "coin", amount: 1240, rate: 14 },
    { key: "wood", name: "Wood", amount: 860, rate: 9 },
    { key: "stone", name: "Stone", amount: 310, rate: 4 },
    { key: "pop", name: "Supply", amount: 46, capacity: 46 },
  ];
  const e = engineFor("HudResRoom", () => {
    og.ui.column("Room", { width: { kind: "fixed", px: 200 } }, () => strategy.resourceBar("Res", { items, anchor: null }));
  });
  e.advance(4);
  const bar = at(dump(e), "Room/Res")!;
  expect(bar.width).toBeLessThanOrEqual(200.5);
});

test("strategy.rtsConsole and dragSelect: four zones side by side, a paged selection, a fixed command grid, a queue, the box", () => {
  let building = false;
  let queued = true;
  let drag: { x: number; y: number } | null = null;
  let box: { dragging: boolean; x: number; y: number; width: number; height: number } | null = null;
  const e = engineFor("HudRts", () => {
    const units = Array.from({ length: 14 }, (_, i) => ({ key: `u${i}`, name: `Unit ${i}`, health: i === 3 ? 20 : 100, healthMax: 100 }));
    const commands = [{ key: "a", name: "Attack", hotkey: "Q" }, { key: "" }, { key: "s", name: "Stop", hotkey: "S", cooldown: 4, cooldownMax: 8 }, ...Array.from({ length: 4 }, (_, i) => ({ key: `c${i}`, name: `C${i}` }))];
    strategy.rtsConsole("Rts", {
      map: { shape: "square", zoom: 1 },
      selection: {
        units: building ? [{ key: "b", name: "Barracks", health: 900, healthMax: 1000 }] : units, focus: building ? "b" : "u3",
        groups: [{ key: "g1", label: "1", count: 14 }, { key: "g2", label: "2", count: 0 }], group: "g1",
        stats: [{ key: "armour", label: "Armour", value: "1" }],
        ...(building && queued ? { queue: { items: [{ key: "q0", name: "Marine", remaining: 5, duration: 10 }, { key: "q1", name: "Marine", count: 3 }] } } : {}),
      },
      commands: { commands, columns: 5 },
    });
    if (drag) box = strategy.dragSelect("Drag", { from: { x: 100, y: 120 }, to: drag });
  });
  e.advance(4);
  const draws = drawsDuring(() => e.advance(1));
  let els = dump(e);
  const zone = (p: string) => at(els, `RtsHud/Rts/Panel/${p}`)!;
  const map = zone("Map");
  // The map fills its zone, square, edge to edge (not a dish inset in a well under a north badge's room).
  const glass = at(els, "RtsHud/Rts/Panel/Map/View/Bezel/Map")!;
  expect([glass.x, glass.y, glass.width, glass.height]).toEqual([map.x, map.y, map.width, map.height]);
  // The control group tabs stand on the panel's top edge, the current one's seam over the panel's top hairline.
  const panel = at(els, "RtsHud/Rts/Panel")!;
  const tab = at(els, "RtsHud/Rts/TabRow/Tabs/g1")!;
  expect(tab.y + tab.height).toBeCloseTo(panel.y, 0);
  const seam = at(els, "RtsHud/Rts/TabRow/Tabs/g1/Seam")!;
  expect(seam.y).toBeLessThanOrEqual(panel.y);
  expect(seam.y + seam.height).toBeGreaterThan(panel.y);
  expect(at(els, "RtsHud/Rts/TabRow/Tabs/g2/Seam")).toBe(undefined);
  // The focused tile in the selected-tile rule: the highlight ring and a glow lifting it; the others neither.
  const effects = (k: string) => JSON.stringify(draws.get(`RtsHud/Rts/Panel/Selection/Body/Grid/Tiles/${k}`)?.[0]?.effects ?? []);
  expect(effects("u3")).toContain("outerGlow");
  expect(effects("u0")).not.toContain("outerGlow");
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Grid/Tiles/u3/Focus")).toBeDefined();
  const sel = zone("Selection");
  const cmd = zone("Commands");
  // Minimap, selection and command card in that order along one panel, none over another.
  expect(map.x + map.width).toBeLessThanOrEqual(sel.x);
  expect(sel.x + sel.width).toBeLessThanOrEqual(cmd.x);
  // Fourteen units in a two-row grid: the last tile of the page counts what does not fit.
  const tiles = below(els, "RtsHud/Rts/Panel/Selection/Body/Grid/Tiles").filter((x) => x.path.split("/").length === 8);
  const more = tiles.find((x) => x.id === "More");
  expect(more).toBeDefined();
  const shown = tiles.filter((x) => x.id.startsWith("u")).length;
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Grid/Tiles/More/Count")?.text).toBe(`+${14 - shown}`);
  // The tiles are a listbox's options (keys and pads reach each), not buttons: the screen's primary is never a unit.
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Grid/Tiles/u3")?.role).toBe("option");
  expect(at(els, "RtsHud/Rts/TabRow/Tabs/g1")?.role).toBe("tab");
  expect(at(els, "RtsHud/Rts/Panel/Commands/Grid")?.role).toBe("grid");
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Card/Detail/Info/Name")?.text).toBe("Unit 3");
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Card/Queue")).toBe(undefined);
  // The command card keeps its places: an empty key leaves a well where the second command would be, so Stop is third.
  const row0 = below(els, "RtsHud/Rts/Panel/Commands/Grid/Row#0").filter((x) => x.path.split("/").length === 7).map((x) => x.id);
  expect(row0).toEqual(["a", "Empty#1", "s", "c0", "c1"]);
  expect(at(els, "RtsHud/Rts/Panel/Commands/Grid/Row#0/a/Slot/Key")?.text).toBe("Q");
  expect(at(els, "RtsHud/Rts/Panel/Commands/Grid/Row#1/Empty#9")).toBeDefined();
  // A building: no grid of one; its production beside the card, the first item large and counting down, a stacked slot's count after it.
  building = true;
  drag = { x: 300, y: 260 };
  e.advance(3);
  els = dump(e);
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Grid")).toBe(undefined);
  const card = at(els, "RtsHud/Rts/Panel/Selection/Body/Card")!;
  const prod = at(els, "RtsHud/Rts/Panel/Selection/Body/Production")!;
  expect(card.x + card.width).toBeLessThanOrEqual(prod.x);
  const cur = at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Current/Active/Queue/q0")!;
  const next = at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Next/q1")!;
  expect(cur.width).toBeGreaterThan(next.width);
  // The item being built is a slot the size of a command cell, so the building's card stays the one large image.
  const cellW = at(els, "RtsHud/Rts/Panel/Commands/Grid/Row#0/a")!.width;
  expect(cur.width).toBeCloseTo(cellW, -0.5);
  expect(cur.width).toBeLessThan(at(els, "RtsHud/Rts/Panel/Selection/Body/Card/Detail/Portrait")!.height * 0.5);
  // Its time under it, not over its art.
  const time = at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Current/Active/Time")!;
  expect(time.text).toBe("5");
  expect(time.y).toBeGreaterThanOrEqual(cur.y + cur.height - 0.5);
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Current/Active/Queue/q0/Time")).toBe(undefined);
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Current/Info/Name")?.text).toBe("Marine");
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Next/q1/Count")?.text).toBe("3");
  expect(states(at(els, "RtsHud/Rts/Panel/Selection/Body/Production/Current/Active/Queue")).queue).toBe("building");
  // The drag box: the rectangle between its corners, drawn where it says.
  expect(box).toEqual({ dragging: true, released: false, x: 100, y: 120, width: 200, height: 140 } as never);
  const b = at(els, "DragHud/Drag")!;
  expect([b.x, b.y, b.width, b.height]).toEqual([100, 120, 200, 140]);
  // Nothing queued: the card keeps its width and its stats take the grid's zone, rather than the card stretching over it.
  queued = false;
  e.advance(2);
  els = dump(e);
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Card")!.width).toBe(card.width);
  expect(at(els, "RtsHud/Rts/Panel/Selection/Body/Stats/Columns/Col#0/armour/Value")?.text).toBe("1");
}, LOADED_MS);

test("strategy.commandCard by keys: the grid is one focus stop whose cursor the arrows walk, Enter issuing the command under it", () => {
  // Wrong: cells no key reaches, or Enter issuing the first command wherever the cursor is.
  const issued: string[] = [];
  let focus = true;
  const e = engineFor("HudCmdKeys", () => {
    const r = strategy.commandCard("Cmd", { anchor: null, columns: 3, commands: [{ key: "a", name: "Attack" }, { key: "", name: "" }, { key: "s", name: "Stop" }, { key: "h", name: "Hold" }] });
    if (r.clicked) issued.push(r.clicked);
    if (focus) og.ui.setFocus("Cmd/Grid");
    focus = false;
  });
  e.advance(3);
  const key = (code: string) => {
    e.probe.commands(`0:key_down:${code};0.05:key_up:${code}`);
    e.advance(8);
  };
  key("ArrowRight");
  key("ArrowRight");
  key("Enter");
  expect(issued).toEqual(["s"]);
  // Down from Stop's column to the second row's last cell is a well: Enter there issues nothing; back to Hold, it does.
  key("ArrowDown");
  key("Enter");
  expect(issued).toEqual(["s"]);
  key("ArrowLeft");
  key("ArrowLeft");
  key("Enter");
  expect(issued).toEqual(["s", "h"]);
  expect(at(dump(e), "Cmd/Grid/Cursor")).toBeDefined();
}, LOADED_MS);

test("strategy.rtsConsole: a minimap pushed out of a phone held sideways stays in its corner tick after tick", () => {
  const e = engineFor("HudRtsLand", () => {
    strategy.rtsConsole("Rts", {
      map: { shape: "square", zoom: 1 },
      selection: { units: Array.from({ length: 8 }, (_, i) => ({ key: `u${i}`, name: `Unit ${i}`, health: 100, healthMax: 100 })), groups: [1, 2, 3, 4, 5].map((n) => ({ key: `g${n}`, label: `${n}`, count: n === 1 ? 8 : 0 })), group: "g1" },
      commands: { commands: [{ key: "a", name: "Attack", hotkey: "Q" }], columns: 5 },
    });
  }, VIEWPORTS.phoneLandscape);
  e.advance(4);
  // Sized by last tick's corner stack, the map once counted itself and so came and went on alternate ticks.
  const sizes: number[] = [];
  for (let i = 0; i < 4; i++) {
    e.advance(1);
    sizes.push(at(dump(e), "RtsMapHud")?.width ?? 0);
  }
  expect(sizes[0]).toBeGreaterThan(64);
  expect(new Set(sizes).size).toBe(1);
  // The group tabs are whole touch targets seated in the panel before the selection, not a row over it that would
  // take the height the map's corner needs.
  const els = dump(e);
  expect(at(els, "RtsHud/Rts/TabRow")).toBe(undefined);
  const panel = at(els, "RtsHud/Rts/Panel")!;
  const sel = at(els, "RtsHud/Rts/Panel/Selection")!;
  for (const n of [1, 2, 3, 4, 5]) {
    const t = at(els, `RtsHud/Rts/Panel/Tabs/g${n}`)!;
    expect([t.width, t.height, t.role]).toEqual([44, 44, "tab"]);
    expect(t.y).toBeGreaterThanOrEqual(panel.y);
    expect(t.y + t.height).toBeLessThanOrEqual(panel.y + panel.height);
    expect(t.x + t.width).toBeLessThanOrEqual(sel.x);
  }
}, LOADED_MS);

test("strategy.commandCard at 150 % text: a cooling cell's seconds stay clear of its key corner; a charge badge's numeral never under the smallest text", () => {
  // Wrong: the seconds centred in a 40 px cell, over the corner key once the text grows; a 16 px badge's 10.6 px digit.
  const PHONE = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
  const e = engineFor("HudCard200", (world) => {
    og.ui.configure(world, { textScale: 1.5 });
    strategy.commandCard("Card", { columns: 3, size: 40, commands: [{ key: "q", name: "Stim", hotkey: "Q", cooldown: 12, cooldownMax: 20 }] });
    og.ui.abilityBar("Bar", { size: 56, slots: [{ key: "w", name: "Blink", hotkey: "W", maxCharges: 3, charges: 2 }] });
  }, PHONE);
  e.advance(3);
  const els = dump(e);
  const key = els.find((x) => x.path.endsWith("/q/Slot/Key"))!;
  const time = els.find((x) => x.path.endsWith("/q/Slot/Time"))!;
  expect([key?.text, time?.text]).toEqual(["Q", "12"]);
  const meet = Math.min(key.x + key.width, time.x + time.width) > Math.max(key.x, time.x) && Math.min(key.y + key.height, time.y + time.height) > Math.max(key.y, time.y);
  expect(meet).toBe(false);
  const slot = els.find((x) => x.path.endsWith("/q/Slot"))!;
  expect(time.y + time.height).toBeLessThanOrEqual(slot.y + slot.height);
  // 12 px on a coarse pointer: the badge holds its numeral at two thirds of its side.
  expect(els.find((x) => x.path.endsWith("/w/Slot/ChargeBadge"))!.width * 0.66).toBeGreaterThanOrEqual(12);
});
