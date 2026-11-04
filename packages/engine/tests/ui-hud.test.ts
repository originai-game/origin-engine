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
// - a killfeed shows the oldest first, past max or past its time, or lays
//   out a line stamped late in a run off screen or not at all;
// - a unit frame shows no hit when its health falls;
// - a resource at its cap reads as fine;
// - an anchored HUD piece leaves the safe area on a phone with a notch.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";
import type { DrawDesc, KillfeedEntry, World } from "@origingame/engine";
import { kit as uiKit } from "../src/ui-kit/bind.ts";
import { loadNativeHost, native } from "../src/node/index.ts";
import { checkFrame, type UiFrameSnapshot } from "../src/node/ui-check.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";
import { at, drawsDuring, dump, engineFor, type El } from "./support/ui.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "templates", "ui-kit");
/** A desktop surface with a 40 px top and 30 px right safe inset (a TV's overscan). */
const DESKTOP_SAFE = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 40, right: 30, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };

{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

// The frames and the misc HUD tests are CPU-bound and nothing else: ticks of
// a dozen components on the dev (-O0) host, no sleep and no wall-clock wait.
// 1.3–1.6 s idle; a loaded gate ran them past 8 s, so the limit leaves that
// headroom three times over. Each advances only as far as its script needs:
// the tick is fixed (60 Hz), so the counts hold on any machine.
const LOADED_MS = 30_000;

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

test("og.ui.unitFrame, raidFrames and castBar: health and states read, raids in columns of five, an interrupted cast leaves", () => {
  let state: "casting" | "interrupted" = "casting";
  let endedAt = 0;
  let health = 30;
  const e = engineFor("HudFrames", () => {
    og.ui.unitFrame("Target", { name: "Grull the Ogre", level: "62", health, hostile: true, unitClass: "elite", reverse: true, targeted: true, anchor: "top" });
    og.ui.raidFrames("Raid", { members: Array.from({ length: 12 }, (_, i) => ({ key: `m${i}`, name: `Member ${i}`, health: 10 * i, dead: i === 0, role: i % 3 === 0 ? "tank" as const : "damage" as const })) });
    og.ui.castBar("Cast", { name: "Pyroblast", progress: 0.4, remaining: 2.1, state, stateAt: endedAt, anchor: "bottom" });
    og.ui.unitFrame("Boss", { name: "Grull the Devourer", level: "??", health: 800_000, healthMax: 1_250_000, shield: 150_000, anchor: "topRight" });
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

test("og.ui.raidFrames: health a bar along a neutral cell's foot under the name; out of range fades the bar, never the name's outline", () => {
  // Wrong: a cell flooded with its health's green (the grid the loudest thing on screen); the name drawn over the
  // fill; an out-of-range cell faded as a whole element, which fades its name's outline with the fill.
  const e = engineFor("HudRaid", () => {
    og.ui.raidFrames("Raid", { anchor: "left", members: [
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
  // A broken combo shows for 600 ms: 36 ticks after it broke.
  e.advance(35);
  expect(states(at(dump(e), "ComboHud/Combo")).combo).toBe("idle");
}, LOADED_MS);

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

test("og.ui.turnOrder as a list: rows top to bottom, the acting one first and selected, an enemy's stripe on the start edge, allies lined up, mirrored right to left", () => {
  // Wrong: the list drawn as the strip (no rows, no names), numbers on the rows, an ally's portrait set in where
  // an enemy's stripe would be, the stripe left of the portrait right to left, or a screen reader told neither
  // the side nor who acts.
  const units = [
    { key: "valiant", name: "Valiant", current: true },
    { key: "raider", name: "Raider", team: 1 },
    { key: "aegis", name: "Aegis" },
    { key: "wasp", name: "Wasp", team: 2, dead: true },
  ];
  const build = () => og.ui.anchor("Side", { anchor: "topLeft", width: { kind: "fixed", px: 320 } } as never, () => og.ui.turnOrder("Turns", { style: "list", units, nowLabel: "Acting", anchor: null }));
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

test("og.ui.standingsTower, lapTimer, rearView and a minimap's track: order, signed gaps, the cut field, the split", () => {
  let player = 11;
  let delta = 180;
  const e = engineFor("HudRace", () => {
    const entries = Array.from({ length: 14 }, (_, i) => ({ key: `c${i}`, name: `Car ${i}`, gap: i * 0.5, player: i === player, pit: i === 5, out: i === 12 }));
    og.ui.standingsTower("Tower", { entries, rows: 8 });
    og.ui.standingsTower("Rel", { entries: entries.slice(0, 4).map((x, i) => ({ ...x, player: i === 2, gap: (i - 2) * 0.4 })), relative: true, anchor: "right" });
    og.ui.lapTimer("Lap", { position: 2, positionTotal: 8, lap: 3, lapTotal: 3, currentMs: 72384, bestMs: 101226, deltaMs: delta });
    og.ui.rearView("Mirror", {});
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
  expect(at(els, "LapHud/Lap/Final")?.text).toBe("FINAL LAP");
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

test("og.ui.rtsConsole and dragSelect: four zones side by side, a paged selection, a fixed command grid, a queue, the box", () => {
  let building = false;
  let queued = true;
  let drag: { x: number; y: number } | null = null;
  let box: { dragging: boolean; x: number; y: number; width: number; height: number } | null = null;
  const e = engineFor("HudRts", () => {
    const units = Array.from({ length: 14 }, (_, i) => ({ key: `u${i}`, name: `Unit ${i}`, health: i === 3 ? 20 : 100, healthMax: 100 }));
    const commands = [{ key: "a", name: "Attack", hotkey: "Q" }, { key: "" }, { key: "s", name: "Stop", hotkey: "S", cooldown: 4, cooldownMax: 8 }, ...Array.from({ length: 4 }, (_, i) => ({ key: `c${i}`, name: `C${i}` }))];
    og.ui.rtsConsole("Rts", {
      map: { shape: "square", zoom: 1 },
      selection: {
        units: building ? [{ key: "b", name: "Barracks", health: 900, healthMax: 1000 }] : units, focus: building ? "b" : "u3",
        groups: [{ key: "g1", label: "1", count: 14 }, { key: "g2", label: "2", count: 0 }], group: "g1",
        stats: [{ key: "armour", label: "Armour", value: "1" }],
        ...(building && queued ? { queue: { items: [{ key: "q0", name: "Marine", remaining: 5, duration: 10 }, { key: "q1", name: "Marine", count: 3 }] } } : {}),
      },
      commands: { commands, columns: 5 },
    });
    if (drag) box = og.ui.dragSelect("Drag", { from: { x: 100, y: 120 }, to: drag });
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

test("og.ui.commandCard by keys: the grid is one focus stop whose cursor the arrows walk, Enter issuing the command under it", () => {
  // Wrong: cells no key reaches, or Enter issuing the first command wherever the cursor is.
  const issued: string[] = [];
  let focus = true;
  const e = engineFor("HudCmdKeys", () => {
    const r = og.ui.commandCard("Cmd", { anchor: null, columns: 3, commands: [{ key: "a", name: "Attack" }, { key: "", name: "" }, { key: "s", name: "Stop" }, { key: "h", name: "Hold" }] });
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

test("og.ui.rtsConsole: a minimap pushed out of a phone held sideways stays in its corner tick after tick", () => {
  const e = engineFor("HudRtsLand", () => {
    og.ui.rtsConsole("Rts", {
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

test("og.ui.matchStats, goalTimeline, playerPlate, substitution and the scorebug's added time: shares, sides, stamina", () => {
  let stamina = 0.5;
  const e = engineFor("HudSports", () => {
    og.ui.scorebug("Bug", { home: { abbr: "NOR", score: 2 }, away: { abbr: "VAL", score: 1 }, clock: "67:42", added: "+3" });
    og.ui.matchStats("Poss", { title: "Possession", anchor: "topRight", stats: [{ key: "poss", home: 58, away: 42, homeText: "58%", awayText: "42%" }] });
    og.ui.matchStats("Stats", { header: true, home: { name: "Northgate" }, away: { name: "Valence" }, stats: [
      { key: "shots", label: "Shots", home: 14, away: 9 },
      { key: "fouls", label: "Fouls", home: 11, away: 14, worse: true },
      { key: "none", label: "Offsides", home: 0, away: 0 },
    ] });
    og.ui.goalTimeline("Goals", { events: [
      { key: "g1", team: 0, name: "J. Anderson", minute: "23'" },
      { key: "g2", team: 1, name: "L. Moreau", minute: "57'" },
      { key: "g3", team: 0, name: "M. Tanaka", minute: "68'" },
    ] });
    og.ui.playerPlate("Me", { number: "10", name: "Martin", stamina });
    og.ui.substitution("Sub", { off: { number: "7", name: "Reyes" }, on: { number: "19", name: "Kova" } });
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

test("og.ui.scorebug at full time and a radar's pitch: a final chip in place of the clock and never the added time; a pitch under the markers", () => {
  // Wrong: a finished match still showing 90:00 +3; a sports radar a black square with dots and no pitch.
  const e = engineFor("HudFinal", () => {
    og.ui.scorebug("Bug", { home: { abbr: "NOR", score: 2 }, away: { abbr: "VAL", score: 1 }, clock: "90:00", added: "+3", period: "2nd", final: "FT" });
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

test("og.ui.wavePreview and laneKeys: the waves past max fold into dots, the current lit; caps under their lanes, a held one down", () => {
  const e = engineFor("HudGenres", () => {
    og.ui.wavePreview("Waves", { current: 8, max: 4, waves: Array.from({ length: 6 }, (_, i) => ({ key: `w${i + 7}`, number: i + 7, name: "Crawlers", count: (i + 1) * 3, boss: i === 4 })) });
    og.ui.laneKeys("Lanes", { y: 600, lanes: [{ key: "a", hotkey: "D", x: 500 }, { key: "b", hotkey: "F", x: 640 }, { key: "c", hotkey: "J", x: 780 }], pressed: ["b"] });
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

test("og.ui.killfeed: lines stamped late in a run are laid out on screen, inside the safe area, newest on top", () => {
  const entries: KillfeedEntry[] = [];
  let push: string[] = [];
  const e = engineFor("HudFeedLate", () => {
    for (const k of push) entries.push({ key: k, killer: "Wraith", killerTeam: 0, victim: "Octane", victimTeam: 1, weaponText: "R-99", at: og.ui.tick() });
    push = [];
    og.ui.killfeed("Feed", { entries });
  }, { ...DESKTOP_SAFE });
  e.advance(500);
  push = ["a"];
  e.advance(20);
  push = ["b"];
  e.advance(20);
  const els = dump(e);
  const a = at(els, "FeedHud/Feed/a")!;
  const b = at(els, "FeedHud/Feed/b")!;
  for (const line of [a, b]) {
    expect(line.width).toBeGreaterThan(80);
    expect(line.height).toBeGreaterThan(10);
    expect(line.x).toBeGreaterThanOrEqual(0);
    expect(line.y).toBeGreaterThanOrEqual(40);
    expect(line.x + line.width).toBeLessThanOrEqual(1280 - 30);
  }
  expect(b.y).toBeLessThan(a.y);
  expect(at(els, "FeedHud/Feed/b/Killer")?.text).toBe("Wraith");
});

test("og.ui.commandCard at 150 % text: a cooling cell's seconds stay clear of its key corner; a charge badge's numeral never under the smallest text", () => {
  // Wrong: the seconds centred in a 40 px cell, over the corner key once the text grows; a 16 px badge's 10.6 px digit.
  const PHONE = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
  const e = engineFor("HudCard200", (world) => {
    og.ui.configure(world, { textScale: 1.5 });
    og.ui.commandCard("Card", { columns: 3, size: 40, commands: [{ key: "q", name: "Stim", hotkey: "Q", cooldown: 12, cooldownMax: 20 }] });
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

test("og.ui.minimapView and og.ui.speedometer on a phone: north's badge holds a 12 px letter and sits on the rim on purpose; no dial numeral meets the readout", () => {
  // Wrong: a 10.5 px north letter reported over the map, or the dial's lowest numeral 2 px into the speed (a readout
  // top the check measured at 6 % under the hub while the readout sat at 5 %).
  const PHONE = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
  const e = engineFor("HudPhoneNav", () => {
    og.ui.minimapView("Map", { size: 168, zoom: 1, anchor: "topRight", markers: [{ key: "me", kind: "player", x: 0.5, y: 0.5 }] });
    og.ui.speedometer("Speedo", { size: 210, speed: 64, maxSpeed: 240, anchor: "bottomLeft" });
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

test("og.ui.unitFrame and og.ui.partyFrames: a level badge's figure is never under the smallest text; a status tile's art lies under its stacks on purpose", () => {
  // Wrong: the badge sized off the portrait alone (a small party frame's 12 px badge, 6.7 px figure; 11.2 px on a large
  // frame), or a tile's stack count reported over its own art.
  const sys = defineSystem("HudLevels", {}, () => {
    og.ui.unitFrame("Me", { name: "Aria", level: "12", health: 80, size: "large", anchor: "topLeft" } as never);
    og.ui.partyFrames("Party", { size: "small", anchor: "left", members: [{ key: "a", name: "Bren", level: "47", health: 60 }, { key: "b", name: "Cass", level: "100", health: 90 }] } as never);
    og.ui.statusGrid("Auras", { effects: [{ key: "bleed", name: "Bleed", debuff: true, type: "bleed", stacks: 3, remaining: 3.5, duration: 10 }], anchor: "bottomLeft" });
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, { root: kit });
  if (!r.ok) throw new Error(r.reason);
  const e = r.value;
  og.ui.previewViewport(e.world, DESKTOP_SAFE as never);
  const font = og.load.font(e.world, "fonts/NotoSansSC-Regular-subset.ttf");
  if (!font.ok) throw new Error(font.reason);
  og.ui.configure(e.world, { font: font.value });
  e.advance(6);
  const f = JSON.parse((native() as unknown as { og_node_ui_frame(id: number): string }).og_node_ui_frame(e.id)) as UiFrameSnapshot;
  const levels = f.texts.filter((x) => ["12", "47", "100"].includes(x.text));
  expect(levels.map((x) => x.text).sort()).toEqual(["100", "12", "47"]);
  for (const l of levels) expect(l.size).toBeGreaterThanOrEqual(12);
  const overlaps = checkFrame(f, { screen: "hud", breakpoint: "desktop", locale: "", style: "", scale: 100, highContrast: false })
    .filter((x) => x.rule === "overlap" && x.paths.some((p) => p.includes("/Tile/")))
    .map((x) => x.paths.join(" × "));
  expect(overlaps.join("; ")).toBe("");
});

test("og.ui.speedometer tach: shift lights fill to the shift point and all light there, numbers outside the arc, dropped at 200 % before the readout shrinks, capped at a third of the short side", () => {
  // Wrong: the lights lit by the redline instead of shiftAt, a number laid out inside the arc where it can reach the
  // readout, the speed shrunk to keep the numbers at 200 %, or a dial wider than 34 % of a phone's short side.
  const tachFor = (rpm: number, view: object, textScale = 1) => {
    const sys = defineSystem("HudTach", {}, () => {
      og.ui.speedometer("Tach", { style: "tach", size: 200, speed: 187, maxSpeed: 300, rpm, redline: 0.85, shiftAt: 0.9, gear: "N", boost: 0.5, anchor: "bottomRight" } as never);
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

test("og.ui.raceBanner: the countdown's numerals and lights by the second, GO, a checkpoint's split that leaves after 1.5 s, the finish, and wrong way over every other moment", () => {
  // Wrong: a numeral that does not step down each second from `at`, lights lit by the wrong count, GO never green, a
  // split that stays, a minus that is a hyphen, or a moment drawn over a wrong way.
  let desc: Record<string, unknown> = {};
  let start = -1;
  let times = false;
  const sys = defineSystem("HudBanner", {}, () => {
    const t = (uiKit as unknown as { tick(): number }).tick();
    if (start < 0) start = t;
    if (times) og.ui.lapTimer("Times", { show: "times", compact: true, currentMs: 61234, bestMs: 60000, lastMs: 61900, deltaMs: -420, anchor: "top" } as never);
    og.ui.raceBanner("Banner", { ...desc, at: start } as never);
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
  expect([textOf("/Split/Delta"), textOf("/WrongWay/Label")]).toEqual([undefined, "WRONG WAY"]);
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

test("og.ui.standingsTower relative window: only the cars around the player, kept whole at the front, one each side on a phone, the player's row with its place and no gap", () => {
  // Wrong: the leaders shown with the window, a window cut short at the front of the field, a phone given two each
  // side, the player's place dropped from its row (the art director's note: show the rank once, not twice), or a gap
  // of its own.
  let player = 11;
  const PHONE = { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 34, bottom: 0 }, coarsePointer: true, breakpoint: "phoneLandscape" };
  const tower = (view?: object) => engineFor("HudWindow", () => {
    const entries = Array.from({ length: 14 }, (_, i) => ({ key: `c${i}`, name: `Car ${i}`, gap: (i - player) * 0.5, player: i === player }));
    og.ui.standingsTower("Tower", { entries, ahead: 2, behind: 2, relative: true } as never);
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

test("og.ui.lapTimer: sector pips in their timing colours, gold on the final lap, a purple best split, boxless text on the scene, the head and the times apart", () => {
  // Wrong: pips missing or all one colour, the final lap's undriven pips not gold, a best split in success, a plate
  // drawn under boxless text (or its text not outlined), or show building the whole block.
  let desc: Record<string, unknown> = {};
  const e = engineFor("HudLapParts", () => {
    og.ui.lapTimer("Lap", { position: 3, positionTotal: 8, lap: 2, lapTotal: 3, currentMs: 61000, deltaMs: -250, ...desc } as never);
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
    og.ui.speedometer("Speedo", { style: "arcade", speed: 143, boost, boostSegments: 3, anchor: "bottomRight" } as never);
    og.ui.lapTimer("Lap", { position: place, positionTotal: 8, positionText: `${place}rd`, placeBadge: true, show: "head" } as never);
    og.ui.itemSlot("Item", { item: tex, roulette: tex ? [tex, tex] : [], rollingAt, rollMs: 1000, label: "Shell" } as never);
    og.ui.driftScore("Drift", drift as never);
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
