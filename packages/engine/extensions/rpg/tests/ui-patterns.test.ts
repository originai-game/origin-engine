import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { conversation as keeperLine } from "@origingame/engine/extensions/rpg/fixtures/party.ts";

import { rpg, type InventoryCarry, type PaperdollEquip, type WorldMapResult } from "@origingame/engine/extensions/rpg/index.ts";

import { bag, BAG, smith } from "./support/patterns.ts";

import { elements, byPath, drag, cell, key, run, centre, release, hold, storyRun, ok, tapOn } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("inventoryGrid: a drag lands a tall item by the cell it was grabbed at, and the grid says what it holds", () => {
  // Wrong: the footprint's top-left put under the pointer (the sword would land two rows up), or a move answered for a press that never moved.
  const b = bag();
  const grid = elements(b.engine).find((e) => e.role === "grid")!;
  expect(grid.name).toBe("Backpack, 10 / 32");
  expect(byPath(b.engine, "Cells/sword")?.name).toBe("Ashbringer, legendary");
  const sword = byPath(b.engine, "Cells/sword")!;
  // Grabbed by its bottom cell (row 2), dropped on cell (6, 3): it lands at (6, 1).
  drag(b.engine, { ...sword, y: sword.y + 100, height: 50 }, cell(b.engine, 6, 3));
  expect(b.moves.length).toBe(1);
  expect(b.moves[0]).toMatchObject({ key: "sword", from: "Bag", x: 6, y: 1, rotated: false, count: 1, split: false, mergeInto: null, swapWith: null });
  expect(byPath(b.engine, "Cells/sword")!.x).toBeCloseTo(byPath(b.engine, "Bag/Cells")!.x + 300, 0);
});

test("inventoryGrid: onto a stack of its kind it merges, onto another item it swaps, past the edge or onto a locked item it is refused", () => {
  // Wrong: every overlap refused (no stacking), or a locked item swapped away.
  const b = bag();
  drag(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 2, 0));
  expect(b.moves.at(-1)).toMatchObject({ key: "potions2", mergeInto: "potions", count: 4 });
  expect(b.items.find((i) => i.key === "potions")!.count).toBe(10);
  // The helm (2×2), grabbed by its top-left cell, dropped on the last cell: three of its cells past the edges, refused.
  const n = b.moves.length;
  const helm = byPath(b.engine, "Cells/helm")!;
  drag(b.engine, { ...helm, width: 50, height: 50 }, cell(b.engine, 7, 3));
  expect(b.moves.length).toBe(n);
  expect(b.results.some((r) => r.refused)).toBe(true);
  // The potions onto the locked ring: refused; onto the helm: a swap.
  drag(b.engine, byPath(b.engine, "Cells/potions")!, cell(b.engine, 2, 1));
  expect(b.moves.length).toBe(n);
  drag(b.engine, byPath(b.engine, "Cells/potions")!, cell(b.engine, 4, 0));
  expect(b.moves.at(-1)).toMatchObject({ key: "potions", swapWith: "helm", x: 4, y: 0 });
});

test("inventoryGrid: Shift and a drag carries half a stack; a locked item will not be picked up", () => {
  // Wrong: Shift ignored (the whole stack moves), or a locked item dragged off.
  const b = bag();
  drag(b.engine, byPath(b.engine, "Cells/potions")!, cell(b.engine, 6, 3), "ShiftLeft");
  expect(b.moves.at(-1)).toMatchObject({ key: "potions", split: true, count: 3, x: 6, y: 3 });
  expect(b.items.find((i) => i.key === "potions")!.count).toBe(3);
  const n = b.moves.length;
  drag(b.engine, byPath(b.engine, "Cells/ring")!, cell(b.engine, 7, 3));
  expect(b.moves.length).toBe(n);
  expect(b.results.some((r) => r.refused)).toBe(true);
});

test("inventoryGrid: the grid is one stop for keys and pads; its cursor walks the items, Enter picks up, R turns, Enter places turned", () => {
  // Wrong: a keyboard player unable to move items (only a pointer drags), a cursor that steps cell by cell through a
  // 2×2 item, the turn lost on the way, or the arrows kept at the grid's edge (the focus could never leave it).
  const b = bag();
  b.engine.advance(2);
  key(b.engine, "Tab");
  const grid = () => elements(b.engine).find((e) => e.role === "grid")!;
  expect(grid().states).toContain("focused");
  expect(grid().value).toBe("Ashbringer, legendary, 1, 1");
  // Right from the sword (column 0) over the empty column 1 to the potions, the second potions, then onto the 2×2 helm.
  for (const want of ["Empty, 2, 1", "Potion, ×6, 3, 1", "Potion, ×4, 4, 1", "Helm, rare, 5, 1"]) {
    key(b.engine, "ArrowRight");
    expect(grid().value).toBe(want);
  }
  expect(b.results.at(-1)!.hovered).toBe("helm");
  key(b.engine, "ArrowRight");
  expect(grid().value).toBe("Empty, 7, 1");
  key(b.engine, "ArrowLeft");
  expect(grid().value).toBe("Helm, rare, 5, 1");
  key(b.engine, "Enter");
  expect(b.results.at(-1)!.carried).toMatchObject({ key: "helm", source: "Bag", byFocus: true, w: 2, h: 2 });
  key(b.engine, "KeyR");
  expect(b.results.at(-1)!.carried).toMatchObject({ rotated: true });
  key(b.engine, "ArrowDown");
  key(b.engine, "ArrowDown");
  key(b.engine, "Enter");
  expect(b.moves.at(-1)).toMatchObject({ key: "helm", rotated: true, x: 4, y: 2 });
  // At the bottom edge Down is not the grid's: the focus leaves it.
  key(b.engine, "ArrowDown");
  expect(grid().states).not.toContain("focused");
  expect(byPath(b.engine, "Root/Below")!.states).toContain("focused");
});

test("inventoryGrid: Back puts a carried item back where it was", () => {
  const b = bag();
  b.setPick({ key: "helm", count: 0 });
  b.engine.advance(2);
  expect(b.results.at(-1)!.carried?.key).toBe("helm");
  key(b.engine, "Escape");
  expect(b.results.at(-1)!.carried).toBeNull();
  expect(b.moves).toEqual([]);
});

test("inventoryGrid: let go outside, the hand is reported released and a paperdoll beside it equips the item", () => {
  // Wrong: a carry that only lands inside its own grid (no equipping by drag), or a slot that takes any tag.
  const items = BAG.map((i) => ({ ...i }));
  let carry: InventoryCarry | null = null;
  const equips: PaperdollEquip[] = [];
  let refused = 0;
  const engine = run(() => {
    og.ui.row("Screen", { gap: 32 } as never, () => {
      const g = rpg.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items });
      const d = rpg.paperdoll("Doll", {
        slots: [{ key: "head", label: "Head", accept: ["helmet"], x: 0.5, y: 0.12 }, { key: "hand", label: "Main hand", accept: ["weapon"], x: 0.15, y: 0.5 }],
        title: "Aria", ...(carry ? { carry } : {}),
      }, () => {});
      carry = g.carried ?? (carry?.released ? null : carry);
      if (d.equip) equips.push(d.equip);
      if (d.refused) refused++;
    });
  });
  expect(byPath(engine, "Slots/head")!.name).toBe("Head, Empty");
  // The sword onto the head slot: refused; onto the main hand: equipped.
  drag(engine, byPath(engine, "Cells/sword")!, centre(byPath(engine, "Slots/head")!));
  expect(equips).toEqual([]);
  expect(refused).toBe(1);
  drag(engine, byPath(engine, "Cells/sword")!, centre(byPath(engine, "Slots/hand")!));
  expect(equips.length).toBe(1);
  expect(equips[0]).toMatchObject({ slot: "hand", key: "sword", from: "Bag", swapped: null });
});

test("inventoryGrid: what is carried says what a drop would do, each verdict its own mark: merge +n, swap, invalid", () => {
  // Wrong: the verdict drawn only on the footprint under the carried item (hidden), the same mark for merge and swap, or a
  // merge's count that is the whole carried stack rather than what the stack takes.
  const b = bag();
  const verdict = () => elements(b.engine).find((e) => e.path.endsWith("Ghost/Badge"))?.name;
  // Let go off the grid each time: the item goes home, nothing moves.
  const away = () => {
    b.engine.probe.commands("0:touch_move:1270,700");
    b.engine.advance(6);
    release(b.engine);
  };
  hold(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 2, 0));
  expect(verdict()).toBe("merge +4");
  away();
  b.items.find((i) => i.key === "potions")!.count = 18;
  b.engine.advance(2);
  hold(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 2, 0));
  expect(verdict()).toBe("merge +2");
  away();
  hold(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 4, 0));
  expect(verdict()).toBe("swap");
  b.engine.probe.commands(`0:touch_move:${cell(b.engine, 2, 1)}`);
  b.engine.advance(6);
  expect(verdict()).toBe("invalid");
  away();
  expect(b.moves).toEqual([]);
});

test("paperdoll: while an item is carried, the slot under it says no in danger, and slots elsewhere that take it only hint", () => {
  // Wrong: every slot that takes the item glowing the same whether or not it is under it, or a slot that refuses it
  // looking like one that does.
  const items = BAG.map((i) => ({ ...i }));
  let carry: InventoryCarry | null = null;
  const engine = run(() => {
    og.ui.row("Screen", { gap: 32 } as never, () => {
      const g = rpg.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items });
      rpg.paperdoll("Doll", {
        slots: [{ key: "head", label: "Head", accept: ["helmet"], x: 0.5, y: 0.12 }, { key: "feet", label: "Feet", accept: ["feet"], x: 0.5, y: 0.85 }],
        ...(carry ? { carry } : {}),
      }, () => {});
      carry = g.carried ?? (carry?.released ? null : carry);
    });
  });
  const target = (k: string) => (byPath(engine, `Slots/${k}`) as unknown as { visualStates: Record<string, string> }).visualStates.target;
  hold(engine, byPath(engine, "Cells/helm")!, centre(byPath(engine, "Slots/feet")!));
  expect(target("feet")).toBe("rejects");
  expect(byPath(engine, "Slots/feet/Rejects")).toBeDefined();
  expect(target("head")).toBe("hint");
  engine.probe.commands(`0:touch_move:${centre(byPath(engine, "Slots/head")!)}`);
  engine.advance(6);
  expect(target("head")).toBe("accepts");
  expect(target("feet")).toBe("dim");
  release(engine);
});

test("paperdoll: a compared stat's change is one signed run, sign first, in both directions, with the real minus", () => {
  // Wrong: right to left the sign lands after the number ("48 +"), or a minus set as a hyphen.
  for (const rtl of [false, true]) {
    const e = run(() => {
      rpg.paperdoll("Doll", { slots: [], compare: true, stats: [{ label: "Armor", value: "120", delta: 48 }, { label: "Speed", value: "12%", delta: -6, deltaText: "−6%" }] }, () => {});
    }, undefined, rtl);
    for (const k of [0, 1]) {
      const mark = byPath(e, `Stat#${k}/Delta/Sign`)!;
      const digits = byPath(e, `Stat#${k}/Delta/Text`)!;
      expect(mark.x < digits.x).toBe(true);
      expect(mark.text).toBe(k === 0 ? "+" : "\u2212");
      expect(digits.text.includes("\u2212") || digits.text.startsWith("+") || digits.text.startsWith("-")).toBe(false);
    }
    expect(byPath(e, "Stat#1/Delta/Text")!.text).toBe("6%");
  }
});

test("paperdoll: an empty slot shows its kind's silhouette, never a letter; a slot of no known kind is a plain socket", () => {
  // Wrong: an empty slot draws the first letter of its name (N, H, R read as debug output), or an off hand that also takes
  // weapons is drawn as a sword.
  const e = run(() => {
    rpg.paperdoll("Doll", { slots: [{ key: "neck", label: "Neck", accept: ["amulet"] }, { key: "off", label: "Off hand", accept: ["weapon", "shield"] }, { key: "x", label: "Xyz" }] }, () => {});
  });
  expect(byPath(e, "neck/Glyph")).toBeDefined();
  expect(byPath(e, "off/Glyph")).toBeDefined();
  expect(byPath(e, "x/Glyph") === undefined).toBe(true);
  expect(elements(e).some((el) => /^(N|O|X)$/.test(el.text))).toBe(false);
});

test("inventoryGrid: right to left, columns count from the right edge", () => {
  // Wrong: a pointer's cell read from the left of a mirrored grid (the item lands in the mirror cell).
  const b = bag(true);
  const g = byPath(b.engine, "Bag/Cells")!;
  const sword = byPath(b.engine, "Cells/sword")!;
  // Column 0 is now at the right.
  expect(sword.x + sword.width).toBeCloseTo(g.x + g.width, 0);
  // Grabbed by its middle cell, dropped on column 5's second row: it lands at (5, 0).
  drag(b.engine, sword, `${Math.round(g.x + g.width - 5 * 50 - 25)},${Math.round(g.y + 75)}`);
  expect(b.moves.at(-1)).toMatchObject({ key: "sword", x: 5, y: 0 });
});

test("conversation: the line types out by the tick; a press shows it whole before the next goes on", () => {
  // Wrong: a press while typing advancing (the rest of the line never read), or the whole line at once.
  const s = smith({ charsPerSecond: 20 }, (v) => ({ ...v, choices: [], ended: false }));
  const said = () => elements(s.engine).find((e) => e.role === "status" && e.path.endsWith("Line"))!;
  // Said whole at once to a screen reader, never letter by letter.
  expect(said().name).toBe("Brannoc: Steel's not cheap. You've 10 gold.");
  expect(s.results.at(-1)!.revealed).toBeFalse();
  // Keys and pads reach the press and every tool, not the pointer alone (Auto has no action of its own).
  const focusable = (end: string) => byPath(s.engine, end)?.states.includes("focusable") ?? false;
  expect(["Talk/Hit", "Tools/Skip", "Tools/Log", "Tools/Auto"].map(focusable)).toEqual([true, true, true, true]);
  s.tapEl("Talk/Hit");
  expect(s.results.at(-1)!.revealed).toBeTrue();
  expect(s.results.some((r) => r.advanced)).toBeFalse();
  // The chevron stands after the words, inside the words' box: not under them, not past the caption.
  const [more, line, words] = ["Words/More", "Words/Line", "Caption/Words"].map((e) => byPath(s.engine, e)!);
  expect(more.x).toBeGreaterThanOrEqual(line.x + line.width - 0.5);
  expect(more.x + more.width).toBeLessThanOrEqual(words.x + words.width + 0.5);
  expect(more.y + more.height).toBeLessThanOrEqual(words.y + words.height + 0.5);
  s.tapEl("Talk/Hit");
  expect(s.results.filter((r) => r.advanced).length).toBe(1);
});

test("conversation: on a phone in portrait at 150 % text the caption and its choices settle, clear of the tools and of each other", () => {
  // Wrong: the stack sized from the caption's last height and the caption from the stack's, so they trade room
  // every tick and never settle, the caption over the tools or the choices.
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 342 }, height: { kind: "fixed", px: 715 } }, () => rpg.conversation("Talk", { view: keeperLine(), charsPerSecond: 0 }));
  }, { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } });
  og.ui.configure(engine.world, { textScale: 1.5 });
  engine.advance(120);
  const boxes = () => ["Talk/Tools", "Talk/Caption", "Talk/Choices"].map((p) => byPath(engine, p)!).map((e) => ({ y: e.y, bottom: e.y + e.height }));
  const [tools, caption, choices] = boxes();
  engine.advance(1);
  expect(boxes()).toEqual([tools!, caption!, choices!]);
  expect(caption!.y).toBeGreaterThanOrEqual(tools!.bottom);
  expect(caption!.bottom).toBeLessThanOrEqual(choices!.y);
  // The stack outgrows its share: it scrolls to its last choice rather than cutting it off.
  const last = () => byPath(engine, "Choices/List/leave");
  expect(last()?.clipped ?? true).toBeTrue();
  const list = byPath(engine, "Choices/List")!;
  engine.probe.commands(`0:touch_move:${Math.round(list.x + list.width / 2)},${Math.round(list.y + list.height / 2)};0.05:scroll:0,600`);
  engine.advance(30);
  expect(last()?.clipped).toBeFalse();
});

test("conversation: a choice answers its ChoiceView.index (a hidden one counted), a disabled one says why and takes no press", () => {
  // Wrong: answering the row's position (Leave is the third row but index 3), or a disabled choice chosen.
  const s = smith({ charsPerSecond: 0, statNames: { charm: "Charm" } }, (v) => ({
    ...v,
    ended: false,
    timer: null,
    choices: v.choices.map((c) => (c.index === 0 ? { ...c, enabled: false, reason: "You need 5 more gold." } : c)),
  }));
  s.engine.advance(4);
  const buy = elements(s.engine).find((e) => e.role === "button" && e.name.includes("Buy the sword"))!;
  expect(buy.name).toBe("1, Buy the sword (10 gold), You need 5 more gold.");
  expect(buy.states).toContain("disabled");
  // The line whole and the choices up, a press on the scene does nothing: the press is disabled, so the pad walks
  // the choices and not a dead stop.
  expect(byPath(s.engine, "Talk/Hit")!.states).toContain("disabled");
  const haggle = elements(s.engine).find((e) => e.role === "button" && e.name.includes("Haggle"))!;
  // d20 + 3 against 12: 60 %.
  expect(haggle.name).toBe("2, Charm · 60 %, Haggle");
  s.tapEl(buy.path.split("/").slice(-2).join("/"));
  expect(s.results.some((r) => r.chosen !== null)).toBeFalse();
  s.tapEl("Choices/List/smith.leave");
  expect(s.results.find((r) => r.chosen !== null)!.chosen).toBe(3);
});

test("conversation: auto holds a whole line until its voice has ended, then goes on once", () => {
  // Wrong: auto cutting off a voice still playing (advancing on the pause alone), or advancing every tick after.
  let left = 120;
  const s = smith({ charsPerSecond: 0, auto: true, autoDelayMs: 100 }, (v) => ({ ...v, choices: [], ended: false, voiceTicks: 240, voiceLeft: left }));
  s.engine.advance(60);
  expect(s.results.some((r) => r.advanced)).toBeFalse();
  expect(elements(s.engine).some((e) => e.path.endsWith("Head/Voice"))).toBeTrue();
  left = 0;
  s.engine.advance(30);
  expect(s.results.filter((r) => r.advanced).length).toBe(1);
});

test("questLog: shows the tracked quest, its hidden objective as ???, and answers Track for the game to pass on", () => {
  // Wrong: the first quest shown instead of the tracked one, a hidden objective's text leaked, Track flipping state itself.
  const tracks: string[] = [];
  const engine = storyRun((w) => {
    if (og.story.quests(w).length === 0) {
      ok(og.story.startQuest(w, "rescue"));
      og.story.progress(w, "rescue", "wolves", 2);
    }
    const qs = og.story.quests(w).map((q) => ({ ...q, objectives: [...q.objectives, { id: "secret", text: "Find the traitor", count: 0, target: 1, done: false, failed: false, optional: false, hidden: true }] }));
    const decoy = { ...qs[0]!, id: "decoy", title: "Decoy", tracked: false };
    const r = rpg.questLog("Quests", { quests: [decoy, ...qs] });
    if (r.track) tracks.push(r.track);
  });
  engine.advance(4);
  expect(byPath(engine, "Detail/Body/Head/TitleRow/Title")!.text).toBe("The Lost Sword");
  // Lines of a list the game ticks, not checkboxes the player could press and no pad reaches.
  const objectives = elements(engine).filter((e) => e.role === "listitem" && e.path.includes("/Objectives/")).map((e) => e.name);
  expect(elements(engine).some((e) => e.role === "checkbox" && e.path.includes("/Objectives/"))).toBeFalse();
  expect(objectives).toContain("Drive off the wolves (2/3), 2 of 3");
  expect(objectives).toContain("???");
  expect(objectives.some((n) => n.includes("traitor"))).toBeFalse();
  engine.probe.commands(`0:touch_move:${centre(byPath(engine, "Actions/Track")!)};0.05:touch_start;0.1:touch_end`);
  engine.advance(10);
  expect(tracks).toEqual(["rescue"]);
});

test("questLog: a reward's caption stands in its own row under the tile; a phone's Back is the secondary button with a chevron", () => {
  // Wrong: the name inside the card a style dresses (its frame and notches cut it), or Back a bare glass pane.
  const engine = storyRun((w) => {
    if (og.story.quests(w).length === 0) ok(og.story.startQuest(w, "rescue"));
    rpg.questLog("Quests", { quests: og.story.quests(w) });
  });
  engine.advance(4);
  const tile = byPath(engine, "Rewards/R#0/Tile")!;
  const caption = byPath(engine, "Rewards/R#0/Caption")!;
  expect(tile.width).toBe(tile.height);
  expect(caption.y).toBeGreaterThanOrEqual(tile.y + tile.height);
  expect(byPath(engine, "Rewards/R#0/Caption/Name")!.text).toBe("Gold");
  expect(elements(engine).some((e) => e.path.includes("Rewards/R#0/Tile/") && e.text === "Gold")).toBeFalse();
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as never);
  engine.advance(4);
  engine.probe.commands(`0:touch_move:${centre(byPath(engine, "List/Active/rescue")!)};0.05:touch_start;0.1:touch_end`);
  engine.advance(10);
  expect(byPath(engine, "Detail/Back/Chevron")).toBeDefined();
  expect(byPath(engine, "Detail/Back/Label")!.text).toBe("Back");
});

test("questTracker: a tracked quest that completes flashes Complete instead of vanishing at once", () => {
  // Wrong: a completed quest dropped from the tracker the tick it completes (no Complete), or an untracked one shown.
  let done = false;
  const engine = storyRun((w) => {
    if (og.story.quests(w).length === 0) ok(og.story.startQuest(w, "rescue"));
    if (done && og.story.quest(w, "rescue")!.status === "active") ok(og.story.completeQuest(w, "rescue"));
    rpg.questTracker("Track", { quests: og.story.quests(w), anchor: null });
  });
  engine.advance(4);
  expect(elements(engine).some((e) => e.path.endsWith("Track/rescue"))).toBeTrue();
  done = true;
  engine.advance(6);
  expect(elements(engine).some((e) => e.text === "Complete")).toBeTrue();
});

// ── world map ───────────────────────────────────────────────────────────

test("worldMap: a pin opens its detail; a fast-travel point travels, another place takes a waypoint; a kind turned off draws no pins", () => {
  // Wrong: Fast travel offered on an undiscovered point or a shop, a hidden kind's pins still pressable, or the legend flipping state itself.
  const results: WorldMapResult[] = [];
  const st = { selected: "", hidden: [] as string[] };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = rpg.worldMap("World", {
        title: "Realm",
        markers: [
          { key: "gate", kind: "fastTravel", name: "Gate", x: 0.3, y: 0.4 },
          { key: "far", kind: "fastTravel", name: "Far", x: 0.5, y: 0.6, discovered: false },
          { key: "mill", kind: "shop", name: "Mill", x: 0.45, y: 0.7 },
        ],
        selected: st.selected,
        hidden: st.hidden,
      });
      results.push(r);
      if (r.selected !== null) st.selected = r.selected;
    });
  });
  tapOn(engine, byPath(engine, "Pins/gate")!);
  expect(st.selected).toBe("gate");
  expect(byPath(engine, "Detail/Action/Label")!.text).toBe("Fast travel");
  tapOn(engine, byPath(engine, "Detail/Action")!);
  expect(results.filter((r) => r.travel).map((r) => r.travel)).toEqual(["gate"]);
  tapOn(engine, byPath(engine, "Pins/far")!);
  expect(byPath(engine, "Detail/Action/Label")!.text).toBe("Set waypoint");
  tapOn(engine, byPath(engine, "Detail/Action")!);
  expect(results.filter((r) => r.track).map((r) => r.track)).toEqual(["far"]);
  // On a desktop the open detail stands in the legend's place; Back closes it and the legend returns.
  expect(byPath(engine, "Legend/shop") === undefined).toBeTrue();
  key(engine, "Escape");
  expect(st.selected).toBe("");
  tapOn(engine, byPath(engine, "Legend/shop")!);
  expect(results.filter((r) => r.toggled).map((r) => r.toggled)).toEqual(["shop"]);
  expect(byPath(engine, "Pins/mill")).toBeDefined();
  st.hidden = ["shop"];
  engine.advance(2);
  expect(byPath(engine, "Pins/mill") === undefined).toBeTrue();
});
