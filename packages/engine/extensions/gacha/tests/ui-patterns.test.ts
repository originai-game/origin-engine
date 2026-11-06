import { expect, test } from "bun:test";

import { og, defineSystem } from "@origingame/engine";

import { createEngine, type GameToken } from "../../../src/runtime.ts";

import { gacha } from "@origingame/engine/extensions/gacha/index.ts";

import { systemIds, byPath, run, elements, key, centre } from "../../../tests/support/patterns.ts";

import { banner, pull, PULL } from "./support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("gachaBanner: full-bleed content reserves safe area and chrome independently of prior siblings", () => {
  const sys = defineSystem(`UpBuild${systemIds.next++}`, {}, () => {
    // lastPath is this unrelated element, not the banner opened below.
    og.ui.anchor("Earlier", { anchor: "center", width: { kind: "fixed", px: 30 }, height: { kind: "fixed", px: 30 } }, () => {});
    og.ui.anchor("Bleed", { anchor: "topLeft", offsetY: -47, margin: 0, width: { kind: "fixed", px: 390 }, height: { kind: "fixed", px: 844 } }, () => {
      gacha.gachaBanner("Banner", {
        topInset: 86, title: "MOONLIT TIDE", current: "event",
        banners: [{ key: "event", name: "Moonlit Tide" }, { key: "weapon", name: "Forged Oath" }],
        balance: 1800, single: 160, multi: 1600, pityMax: 90, pity: 72,
      }, () => {});
    });
  });
  const created = createEngine(og.game({ systems: [sys] }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  try {
    og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 1, breakpoint: "phonePortrait", coarsePointer: true, safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as never);
    og.ui.configure(engine.world, { textScale: 1.5 });
    engine.advance(12);
    const main = byPath(engine, "Banner/Body/Main")!;
    const tabs = byPath(engine, "Main/Info/Tabs")!;
    // The chrome ends at 47 + 86; padding must remain outside the scroll.
    expect(main.y).toBeGreaterThanOrEqual(133);
    expect(tabs.y).toBeGreaterThanOrEqual(main.y);
    expect(main.y + main.height).toBeLessThanOrEqual(810);
    expect(byPath(engine, "Info/Title")!.text).toBe("MOONLIT TIDE");
  } finally {
    engine.destroy();
  }
});

test("gachaBanner: a pull the purse pays is asked for; one it cannot is reported short and said so", () => {
  // Wrong: a pull asked for that the balance cannot pay (the game would spend money it has not), or short shown only by colour.
  const b = banner(1000);
  b.press("Buttons/Single");
  expect(b.results.some((r) => r.pull === 1)).toBe(true);
  b.press("Buttons/Multi");
  expect(b.results.some((r) => r.pull === 10)).toBe(false);
  expect(b.results.some((r) => r.short === 10)).toBe(true);
  expect(byPath(b.engine, "Buttons/Multi")!.name).toContain("not enough");
  expect(byPath(b.engine, "Pulls/Status/Pity")!.name).toBe("Guaranteed in 18");
});

test("gachaBanner: the time left is in the game's words, days and hours on the timer, minutes on a tab", () => {
  // Wrong: the English units (3d 5h) under a locale's words, or a tab's minutes in the timer's form.
  const engine = run(() => {
    gacha.gachaBanner("Banner", {
      banners: [{ key: "event", name: "Event", endsIn: 3 * 86400 + 5 * 3600 }, { key: "weapon", name: "Weapon", endsIn: 50 * 60 }],
      current: "event", title: "Moonlit Tide", endsIn: 3 * 86400 + 5 * 3600 + 30,
      daysLabel: "{d}日{h}時間", hoursLabel: "{h}時間{m}分", minutesLabel: "{m}分",
    }, () => {});
  }, { width: 1280, height: 720 });
  const texts = elements(engine).map((e) => e.text);
  expect(texts.filter((t) => t === "3日5時間").length).toBe(2);
  expect(texts).toContain("50分");
  expect(texts.some((t) => /\d[dhm]\b/.test(t))).toBe(false);
});

test("gachaBanner: tabs and the bumpers switch banners, wrapping at the ends", () => {
  const b = banner(5000);
  b.press("Tabs/std");
  expect(b.current()).toBe("std");
  key(b.engine, "KeyE");
  expect(b.current()).toBe("event");
  key(b.engine, "KeyQ");
  expect(b.current()).toBe("std");
  expect(byPath(b.engine, "Tabs/std")!.states).toContain("selected");
});

test("pullSequence: a new token charges, then reveals each result in order, then the summary sorts the highest tier first", () => {
  // Wrong: the summary in draw order, a reveal skipped, or a reveal reported twice.
  const p = pull();
  expect(p.last().phase).toBe("idle");
  p.start(1);
  expect(p.last().phase).toBe("charge");
  p.engine.advance(100);
  expect(p.last().phase).toBe("reveal");
  p.tap();
  p.tap();
  expect(p.last().index).toBe(2);
  expect(p.results.flatMap((r) => (r.revealed ? [r.revealed] : []))).toEqual(["p1", "p2", "p3"]);
  p.tap();
  expect(p.last().phase).toBe("summary");
  const tiles = elements(p.engine).filter((e) => e.role === "listitem").sort((a, b) => a.y - b.y || a.x - b.x).map((e) => e.path.split("/").at(-1));
  expect(tiles).toEqual(["p2", "p3", "p1"]);
});

test("pullSequence: a game's own tier outranks the engine's rarities and names itself on the reveal", () => {
  // Wrong: the tier ignored (the legendary first, the rarity's word), or its rank read as pips of six.
  const p = pull([
    { key: "p1", name: "Aria", stars: 5, rarity: "legendary" },
    { key: "p2", name: "Nyx", stars: 6, rarity: "common", tier: { label: "UR", rank: 9, color: "#ff4fd8" } },
  ]);
  p.start(1);
  p.engine.advance(100);
  p.tap();
  expect(p.last().index).toBe(1);
  expect(elements(p.engine).map((e) => e.text).join(" ")).toContain("UR");
  p.tap();
  expect(p.last().phase).toBe("summary");
  const tiles = elements(p.engine).filter((e) => e.role === "listitem").sort((a, b) => a.y - b.y || a.x - b.x).map((e) => e.path.split("/").at(-1));
  expect(tiles).toEqual(["p2", "p1"]);
});

test("pullSequence: a unit seen for the first time wears the game's word for new", () => {
  let token = 0;
  const engine = run(() => {
    gacha.pullSequence("Pull", { token, items: PULL, againPrice: 1600, newLabel: "新" });
  });
  token = 1;
  engine.advance(2);
  key(engine, "Escape");
  const texts = elements(engine).map((e) => e.text);
  expect(texts).toContain("新");
  expect(texts).not.toContain("NEW");
});

test("pullSequence: Skip goes straight to the summary; Done closes it; a new token starts again", () => {
  const p = pull();
  p.start(1);
  p.engine.advance(4);
  p.engine.probe.commands(`0:touch_move:${centre(byPath(p.engine, "Bar/Skip")!)};0.05:touch_start;0.1:touch_end`);
  p.engine.advance(14);
  expect(p.last().phase).toBe("summary");
  key(p.engine, "Escape");
  expect(p.results.some((r) => r.done)).toBe(true);
  expect(p.last().phase).toBe("idle");
  p.start(2);
  expect(p.last().phase).toBe("charge");
});
