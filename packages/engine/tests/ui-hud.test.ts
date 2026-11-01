// The HUD components (docs/ui.md §11 "HUD", Appendix C, U6c) on the native
// host, read through dump_ui and the accessibility fields.
//
// Ways they could fail, each covered below:
// - a bar's ghost chunk never shows after a hit, or never drains; blocks
//   or shield cells are counted off by one; a falling bar never says it
//   is critical;
// - an ability with a charge left reads as cooling because its next
//   charge is; a cooldown's seconds stay after it ends;
// - more effects than max overflow instead of folding into +n; a debuff
//   lands in the buff row;
// - an interrupted cast stays up, or a success never leaves;
// - an empty magazine reads as low, an infinite reserve as 0;
// - a minimap marker out of range is drawn off the map instead of
//   dropped, or an objective is dropped instead of held on the rim;
// - a compass point east of the heading sits west of the notch, and
//   under a right-to-left locale the strip mirrors so east reads west;
// - a timer's last ten seconds lose their tenths;
// - a killfeed shows the oldest first, past max or past its time;
// - a resource at its cap reads as fine;
// - an anchored HUD piece leaves the safe area on a phone with a notch.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";
import { at, dump, engineFor, type El } from "./support/ui.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "templates", "ui-kit");

{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

const below = (els: El[], prefix: string): El[] => els.filter((x) => x.path.startsWith(`${prefix}/`));
const states = (el: El | undefined): Record<string, string> => el?.visualStates ?? {};

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
        { key: "q", name: "Fireball", hotkey: "Q", cooldown: cd, cooldownMax: 8 },
        { key: "w", name: "Blink", hotkey: "W", maxCharges: 3, charges: 1, cooldown: 4, cooldownMax: 10, chargeProgress: 0.6 },
        { key: "e", name: "Ward", action: "ward", maxCharges: 2, charges: 0, cooldown: 2, cooldownMax: 6 },
        { key: "r", name: "Meteor", hotkey: "R", ultimate: 0.62 },
        { key: "t", name: "Locked", locked: true },
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
  expect(at(els, "BarHud/Bar/w/Slot/Charges")?.text).toBe("1");
  expect(states(at(els, "BarHud/Bar/e/Slot")).ready).toBe("cooling");
  expect(at(els, "BarHud/Bar/r/Slot/Ultimate")?.text).toBe("62%");
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
  expect(at(els, "AurasHud/Auras/Buffs/More/Count")?.text).toBe("+3");
  expect(at(els, "AurasHud/Auras/Debuffs/bleed/Tile/Stacks")?.text).toBe("3");
  expect(at(els, "AurasHud/Auras/Debuffs/bleed/Tile")?.name).toBe("Bleed, 3 stacks, 4 seconds");
  expect(states(at(els, "AurasHud/Auras/Debuffs/bleed/Tile")).time).toBe("expiring");
  expect(at(els, "AurasHud/Auras/Buffs/haste/Time")?.text).toBe("12");
  expect(at(els, "AurasHud/Auras/Buffs/might/Time")).toBe(undefined);
  expect(at(els, "AurasHud/Auras/Buffs")?.role).toBe("list");
});

test("og.ui.unitFrame, raidFrames and castBar: health and states read, raids in columns of five, an interrupted cast leaves", () => {
  let state: "casting" | "interrupted" = "casting";
  let endedAt = 0;
  const e = engineFor("HudFrames", () => {
    og.ui.unitFrame("Target", { name: "Grull the Ogre", level: "62", health: 30, hostile: true, unitClass: "elite", reverse: true, targeted: true, anchor: "top" });
    og.ui.raidFrames("Raid", { members: Array.from({ length: 12 }, (_, i) => ({ key: `m${i}`, name: `Member ${i}`, health: 10 * i, dead: i === 0, role: i % 3 === 0 ? "tank" as const : "damage" as const })) });
    og.ui.castBar("Cast", { name: "Pyroblast", progress: 0.4, remaining: 2.1, state, stateAt: endedAt, anchor: "bottom" });
  });
  e.advance(3);
  let els = dump(e);
  const target = at(els, "TargetHud/Target");
  expect([target?.role, target?.value, states(target).selection]).toEqual(["button", "Level 62, 30% health", "targeted"]);
  // The portrait at the end when reversed.
  const portrait = at(els, "TargetHud/Target/Portrait");
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
  expect(at(els, "CastHud/Cast/Track/Text/Name")?.text).toBe("Interrupted");
  expect(states(at(els, "CastHud/Cast")).cast).toBe("interrupted");
  e.advance(80);
  expect(at(dump(e), "CastHud/Cast")).toBe(undefined);
});

test("og.ui.crosshair and ammoCounter: a hit marker flashes and leaves; low, empty and an infinite reserve", () => {
  let hitAt: number | null = null;
  let mag = 7;
  const e = engineFor("HudShooter", () => {
    og.ui.crosshair("Aim", { spread: 4, hitAt, hit: "headshot" });
    og.ui.ammoCounter("Ammo", { magazine: mag, magazineMax: 30, reserve: null, weapon: "R-301", reloadHint: "R Reload" });
  });
  e.advance(3);
  let els = dump(e);
  expect(at(els, "AimHud/Aim/Marker")).toBe(undefined);
  expect(states(at(els, "AmmoHud/Ammo")).ammo).toBe("low");
  expect(at(els, "AmmoHud/Ammo/Count/Reserve")?.text).toBe("∞");
  expect(at(els, "AmmoHud/Ammo/Hint")?.text).toBe("R Reload");
  expect(below(els, "AmmoHud/Ammo/Rounds").length).toBe(30);
  hitAt = og.ui.tick();
  mag = 0;
  e.advance(2);
  els = dump(e);
  expect(at(els, "AimHud/Aim/Marker")).toBeDefined();
  expect(states(at(els, "AimHud/Aim")).hit).toBe("headshot");
  expect(states(at(els, "AmmoHud/Ammo")).ammo).toBe("empty");
  e.advance(40);
  expect(at(dump(e), "AimHud/Aim/Marker")).toBe(undefined);
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
  const map = at(els, "MapHud/Map/Map")!;
  expect(map.role).toBe("image");
  expect(at(els, "MapHud/Map/Map/far")).toBe(undefined);
  const near = at(els, "MapHud/Map/Map/near")!;
  expect(near.x + near.width / 2).toBeGreaterThan(map.x + map.width / 2 + 30);
  const goal = at(els, "MapHud/Map/Map/goal")!;
  const cx = map.x + map.width / 2;
  const cy = map.y + map.height / 2;
  const gx = goal.x + goal.width / 2;
  const gy = goal.y + goal.height / 2;
  // Straight up, inside the round frame, near its rim.
  expect(Math.abs(gx - cx)).toBeLessThan(2);
  expect(Math.hypot(gx - cx, gy - cy)).toBeLessThan(map.width / 2);
  expect(Math.hypot(gx - cx, gy - cy)).toBeGreaterThan(map.width / 2 - 25);
  expect(at(els, "MapHud/Map/pingPulse") ?? at(els, "MapHud/Map/Map/pingPulse")).toBeDefined();
  expect(at(els, "MapHud/Map/Zone")?.text).toBe("Kings Canyon");
});

/** An engine over the UI kit's project, its string tables loaded (Arabic lays out right to left). */
function kitEngine(name: string, build: (world: World) => void): Engine {
  const sys = defineSystem(name, {}, (_q, _t, world: World) => build(world));
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, { root: kit });
  if (!r.ok) throw new Error(r.reason);
  for (const l of ["en", "ar"]) {
    const s = og.i18n.load(r.value.world, `strings/${l}.ogt`);
    if (!s.ok) throw new Error(s.reason);
  }
  og.ui.previewViewport(r.value.world, { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as never);
  return r.value;
}

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
  expect([at(els, "ObjHud/Obj/q1/s1")?.role, states(at(els, "ObjHud/Obj/q1/s1")).step]).toEqual(["checkbox", "open"]);
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

test("og.ui.killfeed, resourceBar and scoreboard: newest first within max and time, a cap that warns, rows in order", () => {
  const entries = Array.from({ length: 7 }, (_, i) => ({ key: `k${i}`, killer: `P${i}`, victim: `V${i}`, weaponText: "R-301", at: 0, killerTeam: 0, victimTeam: 1, local: i === 6 }));
  const e = engineFor("HudLists", () => {
    og.ui.killfeed("Feed", { entries, max: 4, durationMs: 1000 });
    og.ui.resourceBar("Res", { items: [{ key: "min", name: "Minerals", amount: 1250, rate: 12.5 }, { key: "sup", name: "Supply", amount: 190, capacity: 200, rate: 0 }] });
    og.ui.scoreboard("Board", {
      title: "Bind", columns: [{ key: "k", label: "K" }, { key: "d", label: "D" }],
      teams: [{ team: 0, name: "Attackers", score: "7", rows: [{ key: "a", name: "Ana", values: ["12", "4"], local: true, ping: 32 }, { key: "b", name: "Bo", values: ["3", "9"], dead: true, ping: 240 }] }],
    });
  });
  e.advance(3);
  let els = dump(e);
  const lines = below(els, "FeedHud/Feed").filter((x) => x.path.split("/").length === 3);
  expect(lines.map((x) => x.id)).toEqual(["k6", "k5", "k4", "k3"]);
  expect(at(els, "FeedHud/Feed")?.role).toBe("log");
  expect(at(els, "ResHud/Res/sup/Amount")?.text).toBe("190/200");
  expect(at(els, "ResHud/Res/min/Rate")?.text).toBe("▲12.5/s");
  expect(at(els, "BoardHud/Board/Team#0/a/Value#0")?.text).toBe("12");
  expect(at(els, "BoardHud/Board/Team#0/b")?.value).toBe("3, 9");
  e.advance(70);
  els = dump(e);
  expect(below(els, "FeedHud/Feed").length).toBe(0);
});

test("og.ui.comboCounter, judgement, turnOrder, scorebug, speedometer, flightHud: what each reads", () => {
  let count = 9;
  let judgeAt = 0;
  const e = engineFor("HudMisc", () => {
    og.ui.comboCounter("Combo", { count, multiplier: 4 });
    og.ui.judgement("Judge", { last: { text: "PERFECT", tier: 0, offsetMs: -12, at: judgeAt } });
    og.ui.turnOrder("Turns", { units: [{ key: "hero", name: "Hero", current: true }, { key: "imp", name: "Imp", team: 1 }, { key: "mage", name: "Mage", atb: 0.5 }] });
    og.ui.scorebug("Bug", { home: { abbr: "LAL", score: 98 }, away: { abbr: "BOS", score: 101, possession: true }, clock: "2:41", period: "Q4" });
    og.ui.speedometer("Speedo", { speed: 212, maxSpeed: 300, rpm: 0.92, gear: "5" });
    og.ui.flightHud("Flight", { speed: 420, altitude: 12500, heading: Math.PI / 2, pitch: 0.1, roll: 0.3 });
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
  e.advance(60);
  expect(states(at(dump(e), "ComboHud/Combo")).combo).toBe("idle");
});

test("components anchored at one point stack in the order built instead of drawing over each other", () => {
  const e = engineFor("HudStack", () => {
    og.ui.compass("Compass", { heading: 0 });
    og.ui.hudTimer("Round", { seconds: 42 });
    og.ui.resourceBar("Res", { items: [{ key: "g", name: "Gold", amount: 10 }] });
    og.ui.objectiveTracker("Obj", { quests: [{ key: "q", title: "Hold", steps: [{ key: "s", text: "Hold the line" }] }] });
    og.ui.comboCounter("Combo", { count: 12 });
    og.ui.ammoCounter("Ammo", { magazine: 12, magazineMax: 30 });
    og.ui.speedometer("Speed", { speed: 120 });
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

const VIEWPORTS = {
  desktop: { width: 1920, height: 1080, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  ultrawide: { width: 3440, height: 1440, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  tablet: { width: 1024, height: 768, dpr: 2, safeArea: { left: 0, top: 24, right: 0, bottom: 20 }, coarsePointer: true, breakpoint: "tablet" },
  phonePortrait: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" },
  phoneLandscape: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" },
};

test("every corner HUD piece stays inside the safe area at every breakpoint", () => {
  for (const [name, vp] of Object.entries(VIEWPORTS)) {
    const e = engineFor(`HudSafe${name}`, () => {
      og.ui.statBar("HP", { value: 64, label: "HP", showValue: true, anchor: "bottomLeft" });
      og.ui.abilityBar("Bar", { slots: ["q", "w", "e", "r", "d", "f", "g", "h"].map((key) => ({ key, hotkey: key.toUpperCase(), cooldown: 2, cooldownMax: 5 })) });
      og.ui.ammoCounter("Ammo", { magazine: 18, magazineMax: 30, reserve: 120 });
      og.ui.minimapView("Map", { zone: "Docks" });
      og.ui.killfeed("Feed", { entries: [{ key: "a", killer: "Wraith", victim: "Bloodhound", weaponText: "R-99", at: 0 }] });
      og.ui.compass("Compass", { heading: 1 });
      og.ui.unitFrame("Me", { name: "Aria", level: "12", health: 80, resource: 40, anchor: "topLeft" });
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
