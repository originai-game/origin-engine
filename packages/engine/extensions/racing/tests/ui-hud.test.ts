import { expect, test } from "bun:test";

import { defineSystem, og } from "@origingame/engine";

import type { DrawDesc } from "@origingame/engine";

import { kit as uiKit } from "../../../src/ui-kit/bind.ts";

import { native } from "../../../src/node/index.ts";

import { checkFrame, type UiFrameSnapshot } from "../../../src/node/ui-check.ts";

import { createEngine, type GameToken } from "../../../src/runtime.ts";

import { at, drawsDuring, dump, engineFor, type El } from "../../../tests/support/ui.ts";

import { racing } from "@origingame/engine/extensions/racing/index.ts";

import { below, states, LOADED_MS, kit, DESKTOP_SAFE } from "../../../tests/support/hud.ts";

import "../../../tests/support/hud.ts";

test("racing.standingsTower, lapTimer, rearView and a minimap's track: order, signed gaps, the cut field, the split", () => {
  let player = 11;
  let delta = 180;
  const e = engineFor("HudRace", () => {
    const entries = Array.from({ length: 14 }, (_, i) => ({ key: `c${i}`, name: `Car ${i}`, gap: i * 0.5, player: i === player, pit: i === 5, out: i === 12 }));
    racing.standingsTower("Tower", { entries, rows: 8 });
    racing.standingsTower("Rel", { entries: entries.slice(0, 4).map((x, i) => ({ ...x, player: i === 2, gap: (i - 2) * 0.4 })), relative: true, anchor: "right" });
    racing.lapTimer("Lap", { position: 2, positionTotal: 8, lap: 3, lapTotal: 3, currentMs: 72384, bestMs: 101226, deltaMs: delta });
    racing.rearView("Mirror", {});
    og.ui.minimapView("Map", { track: [{ x: 0.2, y: 0.5 }, { x: 0.5, y: 0.2 }, { x: 0.8, y: 0.5 }, { x: 0.5, y: 0.8 }], zoom: 1, anchor: "bottomRight" });
  });
  e.advance(3);
  let els = dump(e);
  // Fourteen cars in eight rows with the player 12th: the four leaders, a break, then 11th to 13th.
  const rows = below(els, "TowerHud/Tower").filter((x) => x.path.split("/").length === 3).map((x) => x.id);
  expect(rows).toEqual(["c0", "c1", "c2", "c3", "Break", "c10", "c11", "c12"]);
  expect(at(els, "TowerHud/Tower/c11/PositionBox/Position")?.text).toBe("12");
  expect(at(els, "TowerHud/Tower/c0/Gap")?.text).toBe("Leader");
  expect(at(els, "TowerHud/Tower/c10/Gap")?.text).toBe("+5.00");
  expect(at(els, "TowerHud/Tower/c12/Gap")?.text).toBe("OUT");
  expect(states(at(els, "TowerHud/Tower/c11")).row).toBe("player");
  // Relative: ahead is negative, the player has no gap of its own.
  expect(at(els, "RelHud/Rel/c0/Gap")?.text).toBe("−0.80");
  expect(at(els, "RelHud/Rel/c2/Gap")?.text).toBe("—");
  expect(at(els, "RelHud/Rel/c3/Gap")?.text).toBe("+0.40");
  // The final lap's words on their own line over the head; the lap keeps its caption.
  expect(at(els, "LapHud/Lap/Final")?.text).toBe("Final lap");
  expect(at(els, "LapHud/Lap/Head/Lap/Caption")?.text).toBe("Lap");
  expect(at(els, "LapHud/Lap/Head/Position/Num/Value")?.text).toBe("2");
  expect(at(els, "LapHud/Lap/Current/Time")?.text).toBe("1:12.384");
  expect(at(els, "LapHud/Lap/Best/Time")?.text).toBe("1:41.226");
  expect(at(els, "LapHud/Lap/Split/Time")?.text).toBe("+0.18");
  expect(states(at(els, "LapHud/Lap")).split).toBe("behind");
  expect(states(at(els, "LapHud/Lap")).lap).toBe("final");
  expect(at(els, "MirrorHud/Mirror")?.role).toBe("image");
  expect(at(els, "MapHud/Map/Bezel/Map/Track")).toBeDefined();
  // The player moves up into the leaders: the field is no longer cut; the split turns ahead.
  player = 2;
  delta = -420;
  e.advance(2);
  els = dump(e);
  expect(below(els, "TowerHud/Tower").filter((x) => x.path.split("/").length === 3).map((x) => x.id)).toEqual(["c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7"]);
  expect(states(at(els, "TowerHud/Tower/c5")).row).toBe("pit");
  expect(at(els, "TowerHud/Tower/c5/Gap")?.text).toBe("PIT");
  expect(at(els, "LapHud/Lap/Split/Time")?.text).toBe("−0.42");
  expect(states(at(els, "LapHud/Lap")).split).toBe("ahead");
}, LOADED_MS);

test("og.ui.minimapView and racing.speedometer on a phone: north's badge holds a 12 px letter and sits on the rim on purpose; no dial numeral meets the readout", () => {
  // Wrong: a 10.5 px north letter reported over the map, or the dial's lowest numeral 2 px into the speed (a readout
  // top the check measured at 6 % under the hub while the readout sat at 5 %).
  const PHONE = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
  const e = engineFor("HudPhoneNav", () => {
    og.ui.minimapView("Map", { size: 168, zoom: 1, anchor: "topRight", markers: [{ key: "me", kind: "player", x: 0.5, y: 0.5 }] });
    racing.speedometer("Speedo", { size: 210, speed: 64, maxSpeed: 240, anchor: "bottomLeft" });
  }, PHONE);
  e.advance(3);
  const els = dump(e);
  expect(els.find((x) => x.path.endsWith("/North"))!.width * 0.62).toBeGreaterThanOrEqual(12);
  const f = JSON.parse((native() as unknown as { og_node_ui_frame(id: number): string }).og_node_ui_frame(e.id)) as UiFrameSnapshot;
  const overlaps = checkFrame(f, { screen: "hud", breakpoint: "phonePortrait", locale: "", style: "", scale: 100, highContrast: false })
    .filter((x) => x.rule === "overlap" && x.paths.some((p) => /\/(North|Num#\d+)(\/|$)/.test(p)))
    .map((x) => x.paths.join(" × "));
  expect(overlaps.join("; ")).toBe("");
});

test("racing.speedometer tach: shift lights fill to the shift point and all light there, numbers outside the arc, dropped at 200 % before the readout shrinks, capped at a third of the short side", () => {
  // Wrong: the lights lit by the redline instead of shiftAt, a number laid out inside the arc where it can reach the
  // readout, the speed shrunk to keep the numbers at 200 %, or a dial wider than 34 % of a phone's short side.
  const tachFor = (rpm: number, view: object, textScale = 1) => {
    const sys = defineSystem("HudTach", {}, () => {
      racing.speedometer("Tach", { style: "tach", size: 200, speed: 187, maxSpeed: 300, rpm, redline: 0.85, shiftAt: 0.9, gear: "N", boost: 0.5, anchor: "bottomRight" } as never);
    });
    const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, { root: kit });
    if (!r.ok) throw new Error(r.reason);
    const e = r.value;
    og.ui.previewViewport(e.world, view as never);
    const font = og.load.font(e.world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (!font.ok) throw new Error(font.reason);
    og.ui.configure(e.world, { font: font.value, textScale, reducedMotion: true });
    e.advance(3);
    const draws = drawsDuring(() => e.advance(1));
    return { draws, els: dump(e), frame: JSON.parse((native() as unknown as { og_node_ui_frame(id: number): string }).og_node_ui_frame(e.id)) as UiFrameSnapshot };
  };
  const lights = (els: El[]) => els.filter((x) => /\/Light#\d+$/.test(x.path));
  expect(lights(tachFor(0.5, DESKTOP_SAFE).els).length).toBe(7);
  // A light is lit when it glows. 7 lights from shiftAt − 0.25 (0.65) to 0.9: at 0.8, 0.15 of 0.25 in, four are lit.
  const on = (rpm: number) => [...tachFor(rpm, DESKTOP_SAFE).draws].filter(([p, d]) => /\/Light#\d+$/.test(p) && (d.at(-1)?.effects?.length ?? 0) > 0).length;
  expect([on(0.5), on(0.8), on(0.89), on(0.9)]).toEqual([0, 4, 6, 7]);
  // Desktop: every number sits outside the arc's box and nothing in the dial overlaps.
  const desk = tachFor(0.7, DESKTOP_SAFE);
  const dial = desk.els.find((x) => x.path.endsWith("/Tach"))!;
  const track = desk.els.find((x) => x.path.endsWith("/Tach/TachTrack"))!;
  const nums = desk.els.filter((x) => /\/Num#\d+$/.test(x.path));
  expect(nums.map((x) => x.text)).toEqual(["0", "1", "2", "3", "4", "5", "6", "7", "8"]);
  const c = { x: dial.x + dial.width / 2, y: dial.y + dial.height / 2 };
  for (const n of nums) {
    const d = Math.hypot(n.x + n.width / 2 - c.x, n.y + n.height / 2 - c.y);
    expect(d).toBeGreaterThan(track.width / 2);
  }
  const overlaps = (f: UiFrameSnapshot) => checkFrame(f, { screen: "hud", breakpoint: "desktop", locale: "", style: "", scale: 100, highContrast: false })
    .filter((x) => x.rule === "overlap" && x.paths.some((p) => p.includes("/Tach/")))
    .map((x) => x.paths.join(" × "));
  expect(overlaps(desk.frame).join("; ")).toBe("");
  const speed = (els: El[]) => els.find((x) => x.path.endsWith("/Tach/Speed"))!;
  // A phone at 200 %: the dial a third of the short side at most, its numbers gone, the speed no smaller than at 100 %.
  const PHONE = { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 34, bottom: 0 }, coarsePointer: true, breakpoint: "phoneLandscape" };
  const small = tachFor(0.7, PHONE, 1);
  const big = tachFor(0.7, PHONE, 2);
  const bigDial = big.els.find((x) => x.path.endsWith("/Tach"))!;
  expect(bigDial.width).toBeLessThanOrEqual(Math.round(0.34 * 390));
  expect(big.els.filter((x) => /\/Num#\d+$/.test(x.path)).length).toBe(0);
  expect(speed(big.els).height).toBeGreaterThanOrEqual(speed(small.els).height);
  expect(overlaps(big.frame).join("; ")).toBe("");
  expect(speed(desk.els).text).toBe("187");
  // A portrait phone at 150 %: the dial still a third of the short side at most, and the speed at the whole text size
  // (34 → 51 px; a dial-bound step held it at 33), inside the dial or in a readout under it, overlapping nothing.
  const PORTRAIT = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
  const drawn = (f: UiFrameSnapshot) => f.texts.find((x) => x.text === "187")!.size;
  const p100 = tachFor(0.7, PORTRAIT, 1), p150 = tachFor(0.7, PORTRAIT, 1.5);
  expect(p150.els.find((x) => x.path.endsWith("/Tach"))!.width).toBeLessThanOrEqual(Math.round(0.34 * 390));
  expect(drawn(p150.frame)).toBeGreaterThanOrEqual(1.5 * drawn(p100.frame) - 0.5);
  expect(overlaps(p150.frame).join("; ")).toBe("");
});

test("racing.raceBanner: the countdown's numerals and lights by the second, GO, a checkpoint's split that leaves after 1.5 s, the finish, and wrong way over every other moment", () => {
  // Wrong: a numeral that does not step down each second from `at`, lights lit by the wrong count, GO never green, a
  // split that stays, a minus that is a hyphen, or a moment drawn over a wrong way.
  let desc: Record<string, unknown> = {};
  let start = -1;
  let times = false;
  const sys = defineSystem("HudBanner", {}, () => {
    const t = (uiKit as unknown as { tick(): number }).tick();
    if (start < 0) start = t;
    if (times) racing.lapTimer("Times", { show: "times", compact: true, currentMs: 61234, bestMs: 60000, lastMs: 61900, deltaMs: -420, anchor: "top" } as never);
    racing.raceBanner("Banner", { ...desc, at: start } as never);
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, { root: kit });
  if (!r.ok) throw new Error(r.reason);
  const e = r.value;
  og.ui.previewViewport(e.world, DESKTOP_SAFE as never);
  const font = og.load.font(e.world, "fonts/NotoSansSC-Regular-subset.ttf");
  if (!font.ok) throw new Error(font.reason);
  og.ui.configure(e.world, { font: font.value });
  const textOf = (suffix: string) => dump(e).find((x) => x.path.endsWith(suffix))?.text;
  const litLights = (): [number, number] => {
    const d = drawsDuring(() => e.advance(1));
    const lights = [...d].filter(([p]) => /\/Lights\/Light#\d+$/.test(p));
    return [lights.length, lights.filter(([, x]) => (x.at(-1)?.effects?.length ?? 0) > 0).length];
  };
  // A countdown from 3, at 60 ticks a second: 3 with one light, 2 with two, 1 with three, then GO with all, then nothing.
  desc = { event: "countdown", count: 3 };
  e.advance(30);
  expect([textOf("/Count"), litLights()]).toEqual(["3", [3, 1]]);
  e.advance(59);
  expect([textOf("/Count"), litLights()]).toEqual(["2", [3, 2]]);
  e.advance(59);
  expect([textOf("/Count"), litLights()]).toEqual(["1", [3, 3]]);
  e.advance(59);
  expect([textOf("/Count"), litLights()]).toEqual(["GO!", [3, 3]]);
  e.advance(60);
  expect(textOf("/Count")).toBe(undefined);
  // A checkpoint: its split with a true minus, gone after 1.5 s; wrong way wins while it holds.
  start = -1;
  desc = { event: "checkpoint", index: 2, deltaMs: -680 };
  e.advance(30);
  expect([textOf("/Split/Label"), textOf("/Split/Delta")]).toEqual(["Checkpoint 2", "\u22120.68"]);
  desc = { event: "checkpoint", index: 2, deltaMs: -680, wrongWay: true };
  e.advance(2);
  expect([textOf("/Split/Delta"), textOf("/WrongWay/Label")]).toEqual([undefined, "Wrong way"]);
  desc = { event: "checkpoint", index: 2, deltaMs: -680 };
  e.advance(70);
  expect(textOf("/Split/Delta")).toBe(undefined);
  // Under the lap timer's compact times (the lap large, the split beside it, the best under it, no last lap), clear
  // of its plate rather than over its split.
  times = true;
  start = -1;
  desc = { event: "checkpoint", index: 2, deltaMs: -420, below: "TimesHud/Times" };
  e.advance(30);
  const els = dump(e);
  expect([at(els, "TimesHud/Times/Lap/Time")?.text, at(els, "TimesHud/Times/Lap/Split")?.text, at(els, "TimesHud/Times/Best/Time")?.text]).toEqual(["1:01.234", "\u22120.42", "1:00.000"]);
  expect(at(els, "TimesHud/Times/Last/Time")).toBe(undefined);
  const plate = at(els, "TimesHud/Times")!;
  const split = els.find((x) => x.path === "Banner/Split")!;
  expect(split.y).toBeGreaterThanOrEqual(plate.y + plate.height);
  expect(Math.abs(split.x + split.width / 2 - (plate.x + plate.width / 2))).toBeLessThan(2);
  times = false;
  // The finish: the place as the game writes it, the time under it.
  start = -1;
  desc = { event: "finish", placeText: "1st", timeMs: 83456 };
  e.advance(40);
  expect([textOf("/Place"), textOf("/Time")]).toEqual(["1st", "1:23.456"]);
  const f = JSON.parse((native() as unknown as { og_node_ui_frame(id: number): string }).og_node_ui_frame(e.id)) as UiFrameSnapshot;
  const overlaps = checkFrame(f, { screen: "hud", breakpoint: "desktop", locale: "", style: "", scale: 100, highContrast: false })
    .filter((x) => x.rule === "overlap" && x.paths.some((p) => p.startsWith("Banner/")))
    .map((x) => x.paths.join(" × "));
  expect(overlaps.join("; ")).toBe("");
});

test("racing.standingsTower relative window: only the cars around the player, kept whole at the front, one each side on a phone, the player's row with its place and no gap", () => {
  // Wrong: the leaders shown with the window, a window cut short at the front of the field, a phone given two each
  // side, the player's place dropped from its row (the art director's note: show the rank once, not twice), or a gap
  // of its own.
  let player = 11;
  const PHONE = { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 34, bottom: 0 }, coarsePointer: true, breakpoint: "phoneLandscape" };
  const tower = (view?: object) => engineFor("HudWindow", () => {
    const entries = Array.from({ length: 14 }, (_, i) => ({ key: `c${i}`, name: `Car ${i}`, gap: (i - player) * 0.5, player: i === player }));
    racing.standingsTower("Tower", { entries, ahead: 2, behind: 2, relative: true } as never);
  }, view);
  const ids = (e: ReturnType<typeof tower>) => below(dump(e), "TowerHud/Tower").filter((x) => x.path.split("/").length === 3).map((x) => x.id);
  const e = tower();
  e.advance(3);
  expect(ids(e)).toEqual(["c9", "c10", "c11", "c12", "c13"]);
  const els = dump(e);
  expect(at(els, "TowerHud/Tower/c11/PositionBox/Position")?.text).toBe("12");
  expect(at(els, "TowerHud/Tower/c11/Gap")?.text).toBe("");
  expect(at(els, "TowerHud/Tower/c10/Gap")?.text).toBe("−0.50");
  expect(at(els, "TowerHud/Tower/c9/PositionBox/Position")?.text).toBe("10");
  player = 1;
  e.advance(2);
  expect(ids(e)).toEqual(["c0", "c1", "c2", "c3", "c4"]);
  player = 13;
  e.advance(2);
  expect(ids(e)).toEqual(["c9", "c10", "c11", "c12", "c13"]);
  player = 6;
  const p = tower(PHONE);
  p.advance(3);
  expect(ids(p)).toEqual(["c5", "c6", "c7"]);
});

test("racing.lapTimer: sector pips in their timing colours, gold on the final lap, a purple best split, boxless text on the scene, the head and the times apart", () => {
  // Wrong: pips missing or all one colour, the final lap's undriven pips not gold, a best split in success, a plate
  // drawn under boxless text (or its text not outlined), or show building the whole block.
  let desc: Record<string, unknown> = {};
  const e = engineFor("HudLapParts", () => {
    racing.lapTimer("Lap", { position: 3, positionTotal: 8, lap: 2, lapTotal: 3, currentMs: 61000, deltaMs: -250, ...desc } as never);
  });
  const draws = () => drawsDuring(() => e.advance(1));
  const pip = (d: Map<string, DrawDesc[]>, k: number) => d.get(`LapHud/Lap/Head/Lap/Sectors/Sector#${k}`)?.at(-1)?.fills?.[0];
  const colour = (fill: unknown) => JSON.stringify((fill as { paint?: { color?: unknown } })?.paint?.color ?? fill);
  desc = { sectors: ["best", "personal", "slower", "none"] };
  e.advance(2);
  let d = draws();
  const four = [0, 1, 2, 3].map((k) => colour(pip(d, k)));
  expect(new Set(four).size).toBe(4);
  // The final lap: the undriven pip turns gold; the split a new best in purple.
  desc = { lap: 3, sectors: ["personal", "none"], splitBest: true };
  e.advance(1);
  d = draws();
  const gold = colour(pip(d, 1));
  desc = { lap: 2, sectors: ["personal", "none"], splitBest: true };
  e.advance(1);
  d = draws();
  expect(colour(pip(d, 1))).not.toBe(gold);
  expect(states(at(dump(e), "LapHud/Lap")).split).toBe("best");
  // Boxless: no plate under it, its text outlined.
  desc = { boxless: true };
  e.advance(1);
  d = draws();
  expect(d.get("LapHud/Lap")?.at(-1)?.fills ?? []).toEqual([]);
  // An outlined text sets its draw while its row is open (og.ui.setDraw on the label built last).
  expect((d.get("LapHud/Lap/Head/Position/Num") ?? []).some((x) => x.text?.outline)).toBe(true);
  // show: the head alone, then the times alone.
  desc = { show: "head" };
  e.advance(2);
  let els = dump(e);
  expect([at(els, "LapHud/Lap/Head/Lap/Num/Value")?.text, at(els, "LapHud/Lap/Current/Time"), at(els, "LapHud/Lap/Split/Time")]).toEqual(["2", undefined, undefined]);
  desc = { show: "times" };
  e.advance(2);
  els = dump(e);
  expect([at(els, "LapHud/Lap/Head"), at(els, "LapHud/Lap/Current/Time")?.text, at(els, "LapHud/Lap/Split/Time")?.text]).toEqual([undefined, "1:01.000", "−0.25"]);
});

test("the racing HUD's arcade look: whole nitro segments that flare when full, a place badge, an item roulette that lands, a drift chain that banks and breaks", () => {
  // Wrong: a part-filled segment counted whole, no flare at full, a roulette that never lands or lands early, a banked
  // chain that stays, or a broken one that does not turn to danger.
  let boost = 0.5;
  let place = 3;
  let drift: Record<string, unknown> = { score: 0 };
  let rollingAt = 0;
  let tex: unknown = null;
  // The UI's tick, read while a frame is built (outside one og.ui.tick reads 0).
  let tick = 0;
  const sys = defineSystem("HudArcade", {}, () => {
    tick = (uiKit as unknown as { tick(): number }).tick();
    racing.speedometer("Speedo", { style: "arcade", speed: 143, boost, boostSegments: 3, anchor: "bottomRight" } as never);
    racing.lapTimer("Lap", { position: place, positionTotal: 8, positionText: `${place}rd`, placeBadge: true, show: "head" } as never);
    racing.itemSlot("Item", { item: tex, roulette: tex ? [tex, tex] : [], rollingAt, rollMs: 1000, label: "Shell" } as never);
    racing.driftScore("Drift", drift as never);
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, { root: kit });
  if (!r.ok) throw new Error(r.reason);
  const e = r.value;
  og.ui.previewViewport(e.world, DESKTOP_SAFE as never);
  const font = og.load.font(e.world, "fonts/NotoSansSC-Regular-subset.ttf");
  if (!font.ok) throw new Error(font.reason);
  og.ui.configure(e.world, { font: font.value });
  const t = og.load.texture(e.world, "textures/backdrop.png");
  if (!t.ok) throw new Error(t.reason);
  tex = t.value;
  const nitro = () => [...drawsDuring(() => e.advance(1))].filter(([p]) => /\/Nitro#\d+$/.test(p)).map(([, d]) => (d.at(-1)?.effects?.length ?? 0) > 0);
  // Half of three segments: one whole, one charging, no flare.
  e.advance(2);
  expect(nitro()).toEqual([false, false]);
  boost = 1;
  e.advance(1);
  expect(nitro()).toEqual([true, true, true]);
  expect(states(at(dump(e), "SpeedoHud/Speedo")).nitro).toBe("full");
  expect(at(dump(e), "SpeedoHud/Speedo/Readout/Plate/Speed")?.text).toBe("143");
  // The speed sits on its own plate under the tube's ends, never on the nitro (the tube's ends reach 0.69 of its radius down).
  const speedo = at(dump(e), "SpeedoHud/Speedo")!;
  expect(at(dump(e), "SpeedoHud/Speedo/Readout/Plate")!.y).toBeGreaterThan(speedo.y + (speedo.width / 2) * 0.69);
  // The place badge: the game's ordinal.
  expect(at(dump(e), "LapHud/Lap/Head/Position/Num/Value")?.text).toBe("3rd");
  // The roulette spins for a second from the pickup, then lands on the item.
  rollingAt = tick + 1;
  e.advance(30);
  expect(at(dump(e), "ItemHud/Item")?.name).toBe("Rolling");
  e.advance(40);
  expect(at(dump(e), "ItemHud/Item")?.name).toBe("Shell");
  expect(at(dump(e), "ItemHud/Item/Ring/Item")).toBeDefined();
  // A drift chain: running with its multiplier, then banked into the total (and gone after 0.8 s), then a break.
  drift = { score: 1840, multiplier: 3 };
  e.advance(2);
  let els = dump(e);
  expect([at(els, "DriftHud/Drift/Chain/Score")?.text, at(els, "DriftHud/Drift/Chain/Multiplier/Value")?.text, states(at(els, "DriftHud/Drift")).chain]).toEqual(["1,840", "×3", "running"]);
  drift = { score: 1840, multiplier: 3, bankedAt: tick + 1, total: 10000 };
  e.advance(25);
  els = dump(e);
  expect(states(at(els, "DriftHud/Drift")).chain).toBe("banked");
  const mid = Number((at(els, "DriftHud/Drift/Total")?.text ?? "").replaceAll(",", ""));
  expect(mid).toBeGreaterThan(8160);
  expect(mid).toBeLessThan(10000);
  e.advance(30);
  expect(at(dump(e), "DriftHud/Drift")?.path).toBe(undefined);
  drift = { score: 600, multiplier: 2, failedAt: tick + 1 };
  e.advance(5);
  expect(states(at(dump(e), "DriftHud/Drift")).chain).toBe("failed");
  // Their own slants and tubes never reach their neighbours inside the part.
  drift = { score: 12840, multiplier: 4 };
  e.advance(2);
  const f = JSON.parse((native() as unknown as { og_node_ui_frame(id: number): string }).og_node_ui_frame(e.id)) as UiFrameSnapshot;
  const own = (p: string) => p.split("/")[0];
  const overlaps = checkFrame(f, { screen: "hud", breakpoint: "desktop", locale: "", style: "", scale: 100, highContrast: false })
    .filter((x) => x.rule === "overlap" && own(x.paths[0]!) === own(x.paths[1] ?? ""))
    .map((x) => x.paths.join(" × "));
  expect(overlaps.join("; ")).toBe("");
});
