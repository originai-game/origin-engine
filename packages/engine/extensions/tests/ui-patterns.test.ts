import { expect, test } from "bun:test";

import { og, defineSystem } from "@origingame/engine";

import { createEngine, type GameToken } from "../../src/runtime.ts";

import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

import { card } from "@origingame/engine/extensions/card/index.ts";

import { rhythm, type SongEntry } from "@origingame/engine/extensions/rhythm/index.ts";

import { versus, type DraftTeam } from "@origingame/engine/extensions/versus/index.ts";

import { systemIds, type Element, centre, byPath, cardEl, run, elements } from "../../tests/support/patterns.ts";

import { BAG } from "../rpg/tests/support/patterns.ts";

import { HAND, TARGETS } from "../card/tests/support/patterns.ts";

import { ROSTER } from "../versus/tests/support/patterns.ts";

import "../../tests/support/patterns.ts";

test("long press: an item's menu and a card's inspect are og-ui's recogniser, timed in ms whatever the tick rate", () => {
  // Wrong: a hand-counted 30 ticks (a second at 30 Hz, not og-ui's 500 ms), so a 600 ms hold opens nothing.
  const menus: string[] = [];
  const inspects: string[] = [];
  const sys = defineSystem(`UpLong${systemIds.next++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never, () => {
      const r = rpg.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items: BAG, label: "Backpack" });
      if (r.menu) menus.push(r.menu);
    });
    og.ui.anchor("Table", { anchor: "bottom", width: { kind: "fixed", px: 1200 }, direction: "column" } as never, () => {
      const r = card.cardHand("Hand", { cards: HAND, targets: TARGETS, cardWidth: 150 });
      if (r.inspect) inspects.push(r.inspect);
    });
  });
  const made = createEngine(og.game({ tick: 30, systems: [sys] }) as unknown as GameToken);
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  engine.advance(4);
  const holdOn = (el: Element): void => {
    engine.probe.commands(`0:touch_move:${centre(el)};0.05:touch_start;0.7:touch_end`);
    engine.advance(18);
  };
  holdOn(byPath(engine, "Cells/helm")!);
  expect(menus).toEqual(["helm"]);
  engine.advance(8);
  holdOn(cardEl(engine, "defend"));
  expect(inspects).toEqual(["defend"]);
});

test("a screen-filling pattern given less than the safe area (under a game's header) lays out within its box, its action inside", () => {
  // Wrong: the pattern sizes its parts from its first guess (the whole safe area), its box grows to fit them, and the
  // grown box is kept as its room from then on, so its foot stands past the parent's on every tick (the kit's phone
  // character select had Lock in 42 px under the safe area).
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  const songs: SongEntry[] = ["a", "b", "c", "d", "e", "f"].map((k) => ({ key: k, title: `Song ${k}`, artist: "Artist", difficulties: [{ key: "easy", name: "Easy", level: 3 }] }));
  const cases: Array<[string, () => void]> = [
    ["Foot/Lock", () => versus.characterSelect("Sel", { roster: [...ROSTER, ...ROSTER.map((e) => ({ ...e, key: `${e.key}2` }))], cursors: [{ key: "p1", name: "P1", entry: "kael", you: true, team: 0 }] })],
    ["Bar/Action", () => versus.lobby("Lobby", { members: [{ key: "a", name: "Aria", you: true }, { key: "b", name: "Kael" }, { key: "c", name: "Mira" }], slots: 5, mode: { name: "Ranked" } })],
    ["Play", () => rhythm.songSelect("Songs", { songs, selected: "c", difficulty: "easy", preview: {} })],
  ];
  for (const [action, build] of cases) {
    const engine = run(() => og.ui.column("Box", { width: { kind: "fixed", px: 374 }, height: { kind: "fixed", px: 600 } }, build), phone);
    engine.advance(20);
    const box = byPath(engine, "Root/Box")!;
    const a = elements(engine).find((e) => e.path.endsWith(action) && e.role === "button")!;
    expect(a).toBeDefined();
    expect(a.y + a.height).toBeLessThanOrEqual(box.y + box.height + 0.5);
  }
});

test("on a phone in portrait the draft leaves the roster two rows, the map's legend is one row, the song wheel only whole rows", () => {
  // Wrong: the draft's two teams of tall portraits leave the roster one row (117 px of 691), the legend's chips wrap
  // into rows over the map, or a song row past the wheel's edge stands over the detail above it.
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  const box = (build: () => void) => run(() => og.ui.column("Box", { width: { kind: "fixed", px: 374 }, height: { kind: "fixed", px: 691 } }, build), phone);
  const roster = [...ROSTER, ...ROSTER.map((e) => ({ ...e, key: `${e.key}2` }))].map((e) => ({ ...e, locked: false, taken: false }));
  const team = (n: number): DraftTeam => ({ name: `Team ${n}`, team: n, picks: [{ player: "A" }, { player: "B" }, { player: "C" }], bans: ["", ""] });
  const d = box(() => versus.draft("Draft", { teams: [team(0), team(1)], roster, phase: "pick", secondsLeft: 10, yourTurn: true }));
  d.advance(10);
  expect(byPath(d, "Middle/Room")!.height).toBeGreaterThanOrEqual(200);
  const kinds = ["quest", "objective", "fastTravel", "shop", "poi", "boss"];
  const m = box(() => rpg.worldMap("Map", { markers: kinds.map((k, i) => ({ key: k, kind: k, name: k, x: 0.1 + i * 0.15, y: 0.5 })) }));
  m.advance(10);
  expect(byPath(m, "Side/Legend")!.height).toBeLessThanOrEqual(60);
  const songs: SongEntry[] = ["a", "b", "c", "d", "e", "f", "g"].map((k) => ({ key: k, title: `Song ${k}`, artist: "Artist", difficulties: [{ key: "easy", name: "Easy", level: 3 }] }));
  const s = box(() => rhythm.songSelect("Songs", { songs, selected: "d", difficulty: "easy", preview: {} }));
  s.advance(10);
  const wheel = byPath(s, "Songs/Wheel")!;
  const rows = elements(s).filter((e) => e.role === "option" && e.path.includes("/Wheel/"));
  expect(rows.length).toBeGreaterThan(1);
  for (const r of rows) expect(r.y).toBeGreaterThanOrEqual(wheel.y - 0.5);
});
