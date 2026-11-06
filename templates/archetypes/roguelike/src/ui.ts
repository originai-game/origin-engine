// Lamp Dive's screens on og.ui and the style in assets/styles/lampdive (the
// engine's pixel pack in tomb colours): the title and the pause menu (the
// game's screen documents, assets/ui/main-menu.ogt and pause.ogt), the HUD
// (health, strike, armour and gold in the top-left corner, the floor and
// pause in the top-right, the draught and, on a touch screen, Wait and a
// d-pad along the bottom), the settings dialog and the run's summary when
// the lamp goes out or the relic is taken. Screens change through a
// dissolve (a fade with reduced motion). Every string is a key of
// assets/strings/*.ogt.
//
// The Ui system also ends dives a beat after the lamp goes out or the relic
// is taken, sizes the camera for the surface (two delvers on one device:
// wide enough for both), sets the buses and keeps the settings and the
// dive record in og.platform (the record written at the revision read, and
// merged when another device wrote it first; a room keeps none, every
// peer's world being the same). A menu or HUD button that changes the
// world presses its seat's bit (src/seats.ts's press: pause, dive, quit,
// wait, drink) and the Hero system acts on it, so a room acts on one tick;
// the touch d-pads are the seats' move (useTouchControls).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { FOES, Game, Phase, Tuning } from "./components.ts";
import { prefs } from "./device.ts";
import { LOCAL, ONLINE, press, SEATS, useTouchControls, usesTouch } from "./seats.ts";
import { inRoom, lazyTuning, must, OVER_DELAY, partyDown } from "./systems.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** The floors of a dive. */
export const FLOORS = 2;

/** The tokens' spacing scale in CSS px (og.ui.frameTheme answers it too, but builds the whole theme each call). */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** This device's own screen state: the settings dialog, the surface the camera was sized for, each local seat's touch d-pad. */
let settingsOpen = false;
let viewW = 0;
let viewH = 0;
const touchMove = [{ x: 0, y: 0 }, { x: 0, y: 0 }];
// The HUD draws the d-pads (one per local seat); the seats read what they hold.
useTouchControls((seat) => ({ x: touchMove[seat]?.x ?? 0, y: touchMove[seat]?.y ?? 0, lx: 0, ly: 0, buttons: 0 }));

/** A new dive: a fresh seed, the delvers whole, floor 1 carved on this tick. */
export function startDive(world: World, g: GameV): void {
  const t = lazyTuning(world);
  const rng = world.resource(og.Rng);
  g.seed = rng.ok ? rng.value.integer(1, 1_000_000_000) : 1;
  g.phase = Phase.Play;
  g.paused = false;
  g.won = false;
  g.floor = 1;
  g.deepest = 1;
  g.hpMax = t.heroHp;
  g.hp = t.heroHp;
  // Online before the room runs, seat 1 has nobody at it yet (the Hero system brings it in when the room starts).
  g.hpMax2 = SEATS > 1 && (!ONLINE || inRoom(world)) ? t.heroHp : 0;
  g.hp2 = g.hpMax2;
  g.attack = t.heroAttack;
  g.armour = 0;
  g.lamp = t.lamp;
  g.potions = 1;
  g.gold = 0;
  g.kills = 0;
  g.hits = 0;
  g.hurts = 0;
  g.finds = 0;
  g.turns = 0;
  g.killedBy = -1;
  g.overTicks = 0;
  g.foesTurn = false;
  g.newRecord = false;
  g.build = true;
  g.dives += 1;
  saveRecord(world, g);
}

/** The dive record ({dives, relics}) written at the revision it was read at. */
function saveRecord(world: World, g: GameV): void {
  if (inRoom(world)) return;
  const r = og.platform.storeSet(world, { key: "record", value: { dives: g.dives, relics: g.relics }, revision: g.recordRevision });
  if (r.ok) g.recordWrite = r.value;
}

/** The buses' volumes from this device's settings (og.Audio is each peer's own and in no snapshot, so a restored or joining world takes them again). */
function applyPrefs(world: World, p: typeof prefs): void {
  const audio = world.resource(og.Audio);
  if (!audio.ok) return;
  const a = audio.value;
  if (a.master !== p.master) a.master = p.master;
  if (a.music !== p.music) a.music = p.music;
  if (a.sfx !== p.effects) a.sfx = p.effects;
  if (a.ui !== p.effects) a.ui = p.effects;
}

export function setLanguage(world: World, index: number): void {
  prefs.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[prefs.language]!.code);
}

function prefsValue(p: typeof prefs) {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion };
}

/**
 * og.platform: the settings (this device's, src/device.ts) and the record
 * asked for on tick 0 and applied when they answer; a record write refused for a stale revision (another
 * device dived meanwhile) is read again, merged (the stored counts plus
 * what this device added since it last read or wrote them) and written at
 * the new revision.
 */
function platform(world: World, g: GameV, p: typeof prefs): void {
  if (world.tick === 0) {
    p.loadRequest = og.platform.loadSettings(world);
    const r = og.platform.storeGet(world, "record");
    g.recordRequest = r.ok ? r.value : 0;
    return;
  }
  if (p.loadRequest !== 0) {
    const r = og.platform.result(world, p.loadRequest);
    if (r !== null) {
      p.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") p.master = s.masterVolume;
        if (typeof s.musicVolume === "number") p.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, at);
      }
    }
  }
  // A peer's own answers come back on its own ticks: a running room keeps no record.
  if (inRoom(world)) return;
  if (g.recordRequest !== 0) {
    const r = og.platform.result(world, g.recordRequest);
    if (r !== null) {
      g.recordRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: { dives?: number; relics?: number } | null; revision?: number } | null;
      // The stored counts plus what this device added since it last read or wrote them.
      const dives = d?.value?.dives ?? 0;
      const relics = d?.value?.relics ?? 0;
      g.dives = dives + g.dives - g.recordDives;
      g.relics = relics + g.relics - g.recordRelics;
      g.recordDives = dives;
      g.recordRelics = relics;
      if (typeof d?.revision === "number") g.recordRevision = d.revision;
      if (g.recordMerge) {
        g.recordMerge = false;
        saveRecord(world, g);
      }
    }
  }
  if (g.recordWrite !== 0) {
    const r = og.platform.result(world, g.recordWrite);
    if (r !== null) {
      g.recordWrite = 0;
      if (r.ok) {
        g.recordRevision = (r.data as { revision: number }).revision;
        g.recordDives = g.dives;
        g.recordRelics = g.relics;
      }
      else if (r.error?.code === "conflict") {
        // Someone else wrote the record: read it again and write the merge.
        g.recordConflicts += 1;
        g.recordMerge = true;
        const again = og.platform.storeGet(world, "record");
        g.recordRequest = again.ok ? again.value : 0;
      }
    }
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/** A d-pad's held direction as a seat's move (right, up). */
function dpadMove(seat: number, size: number): void {
  const d = og.ui.dpad("Pad", { size });
  touchMove[seat] = { x: d.up ? 0 : d.right ? 1 : d.down ? 0 : d.left ? -1 : 0, y: d.up ? 1 : d.right ? 0 : d.down ? -1 : 0 };
}

function hud(world: World, g: GameV): void {
  const touch = usesTouch();
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Health", () => {
      og.ui.icon("Heart", "heart", { size: 22, tint: { token: g.hp * 3 <= g.hpMax ? "danger" : "ink" } });
      og.ui.meter("Bar", g.hp, g.hpMax, { kind: "health", label: `${g.hp}/${g.hpMax}`, width: px(touch ? 110 : 150) });
    });
    // The second delver's health, its heart in the second seat's ink.
    if (SEATS > 1) {
      capsule("Health2", () => {
        og.ui.icon("Heart", "heart", { size: 22, tint: { token: g.hp2 * 3 <= g.hpMax2 ? "danger" : "primary" } });
        og.ui.meter("Bar", g.hp2, g.hpMax2, { kind: "health", label: `${g.hp2}/${g.hpMax2}`, width: px(touch ? 110 : 150) });
      });
    }
    capsule("Stats", () => {
      og.ui.icon("Sword", "sword", { size: 20 });
      og.ui.counter("Attack", g.attack, { font: "mono", width: px(22) });
      og.ui.icon("Shield", "shield", { size: 20 });
      og.ui.counter("Armour", g.armour, { font: "mono", width: px(22) });
      og.ui.icon("Coin", "coin", { size: 20, tint: { token: "gold" } });
      og.ui.counter("Gold", g.gold, { font: "mono", width: px(40) });
    });
  });
  og.ui.anchor("Depth", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Floor", () => og.ui.label("Label", say(world, "hud.floor", { n: g.floor, total: FLOORS }), { step: "lg", nowrap: true }));
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
  });
  // A refused draught (the Hero system, after this one, refused it last tick) shakes the flask once, with the error sound (og-ui §6).
  const refused = (seat: number) => g.refusedTick === world.tick - 1 && g.refusedSeat === seat;
  if (LOCAL && touch) {
    // Two on one device: each seat's column in its own bottom corner, Wait and Drink over its d-pad.
    for (let seat = 0; seat < 2; seat++) {
      og.ui.anchor(`Seat${seat}`, { anchor: seat === 0 ? "bottomLeft" : "bottomRight", direction: "column", gap: S.sm, cross: "center" }, () => {
        og.ui.row("Actions", { gap: S.sm, cross: "center" }, () => {
          if (og.ui.actionButton("Wait", say(world, "hud.wait"), { icon: "clock", size: 48 }).clicked) press("wait", seat);
          if (og.ui.actionButton("Drink", say(world, "hud.drink"), { icon: "potion", size: 52 }).clicked) press("drink", seat);
          if (refused(seat)) og.ui.shake({ sound: true });
          if (seat === 0) og.ui.label("Count", `× ${g.potions}`, { font: "mono", nowrap: true });
        });
        dpadMove(seat, 120);
      });
    }
    return;
  }
  og.ui.anchor("Hands", { anchor: "bottomLeft", gap: S.md, cross: "end" }, () => {
    og.ui.column("Flask", { cross: "center", gap: S.xxs }, () => {
      if (og.ui.actionButton("Drink", say(world, "hud.drink"), { icon: "potion", size: 60 }).clicked) press("drink");
      if (refused(0)) og.ui.shake({ sound: true });
      og.ui.label("Count", `× ${g.potions}`, { font: "mono", nowrap: true });
    });
    if (touch && og.ui.actionButton("Wait", say(world, "hud.wait"), { icon: "clock", size: 52 }).clicked) press("wait");
  });
  if (touch) og.ui.anchor("Move", { anchor: "bottomRight" }, () => dpadMove(0, 150));
}

function settings(world: World, p: typeof prefs): void {
  let done = false;
  const m = og.ui.modal("Settings", settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.master = slider("Master", "settings.master", p.master);
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, r.selected);
    p.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (settingsOpen && (done || m.dismissed)) {
    settingsOpen = false;
    must(og.platform.saveSettings(world, prefsValue(p)));
    p.saves += 1;
  }
}

/** The run's summary: how it ended, how deep, how long, what fell, what was found. */
function summary(world: World, g: GameV): "again" | "menu" | null {
  let chose: "again" | "menu" | null = null;
  og.ui.modal("Summary", g.phase === Phase.Over, { title: g.won ? say(world, "over.won") : say(world, "over.lost", { n: g.floor }), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", g.won ? "crown" : "skull", { size: 48, tint: { token: g.won ? "primary" : "inkMuted" } }));
      if (g.won && g.overTicks === OVER_DELAY + 20) og.ui.burst({ count: 12 });
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Floor", say(world, "over.floor"), `${g.deepest}/${FLOORS}`, true);
      og.ui.keyValue("Turns", say(world, "over.turns"), `${g.turns}`, true);
      og.ui.keyValue("Kills", say(world, "over.kills"), `${g.kills}`, true);
      og.ui.keyValue("Gold", say(world, "over.gold"), `${g.gold}`, true);
      og.ui.keyValue("Finds", say(world, "over.finds"), `${g.finds}`, true);
      if (!g.won) og.ui.keyValue("By", say(world, "over.by"), say(world, g.killedBy >= 0 ? `foe.${FOES[g.killedBy]}` : "foe.none"), false);
    });
    og.ui.label("Record", say(world, "over.record", { dives: g.dives, relics: g.relics }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) chose = "menu";
    });
  });
  return chose;
}

/**
 * A screen document up while `shown`: og.ui.screen pushes it the first tick
 * and builds it with its viewmodel each tick; the tick it is not shown it is
 * popped (the top layer: the title and the pause never stack). Answers the
 * events it emitted (its buttons' `emit`, Back's).
 */
function layer(world: World, name: string, shown: boolean, vm: object = {}): string[] {
  if (!shown) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/** The screens, the dive's start and end, the camera, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Lockstep], writes: [Game, og.Camera, og.Audio, og.Rng] }, (_q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  const p = prefs;
  platform(world, g, p);
  applyPrefs(world, p);
  touchMove[0] = { x: 0, y: 0 };
  touchMove[1] = { x: 0, y: 0 };
  // The camera: 13 cells tall on a wide screen, 10 cells across a portrait one.
  const vp = og.ui.frameViewport();
  if (vp.width > 0 && (vp.width !== viewW || vp.height !== viewH)) {
    viewW = vp.width;
    viewH = vp.height;
    const cam = world.resource(og.Camera);
    if (cam.ok) cam.value.height = vp.width < vp.height ? (10 * vp.height) / vp.width : 13;
  }
  if (LOCAL && vp.width > 0 && g.phase === Phase.Play) {
    // Two on one device: wide and tall enough for both delvers and two cells round them, eased (each device's own view).
    const cam = world.resource(og.Camera);
    const base = vp.width < vp.height ? (10 * vp.height) / vp.width : 13;
    const want = Math.max(base, Math.abs(g.heroY - g.hero2Y) + 5, ((Math.abs(g.heroX - g.hero2X) + 5) * vp.height) / vp.width);
    if (cam.ok && Math.abs(cam.value.height - want) > 0.01) cam.value.height += (want - cam.value.height) * 0.12;
  }
  // The settings hold this device's time alone, so a room's settings hold nothing.
  const hold = g.phase === Phase.Play && (g.paused || (settingsOpen && !inRoom(world)));
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  // A beat after the lamp goes out or the relic is taken, the summary.
  if (g.phase === Phase.Play && (partyDown(g) || g.won)) {
    g.overTicks += 1;
    if (g.overTicks === OVER_DELAY) {
      g.phase = Phase.Over;
      if (g.won) g.relics += 1;
      saveRecord(world, g);
      must(og.platform.telemetry(world, "dive.over", { won: g.won, floor: g.deepest, turns: g.turns, kills: g.kills, gold: g.gold }));
    }
  } else if (g.phase === Phase.Over) g.overTicks += 1;

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "dissolve", durationMs: 600 }).shown;
  // The title and the pause are the game's screen documents (assets/ui/main-menu.ogt, pause.ogt).
  for (const e of layer(world, "main-menu", shown === "title")) {
    if (!settingsOpen && e === "play") press("dive");
    if (e === "settings") settingsOpen = true;
  }
  // Back from its Settings the pause keeps the focus on Settings, as the pause shipped.
  const fromSettings = og.ui.memory("pause.fromSettings", 0);
  const pauseUp = shown === "play" && g.paused && !settingsOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    if (e === "settings") {
      settingsOpen = true;
      og.ui.remember("pause.fromSettings", 1);
    }
    if (e === "quit") press("quit");
  }
  if (pauseUp && fromSettings === 1) {
    og.ui.setFocus("pause/Pause/Panel/Menu/Settings");
    og.ui.remember("pause.fromSettings", 0);
  }
  if (shown === "title") {
    og.ui.setNavigation(true);
  } else if (shown === "play") {
    hud(world, g);
  } else {
    og.ui.setNavigation(true);
    const c = summary(world, g);
    if (c === "again") press("dive");
    if (c === "menu") press("quit");
  }
  settings(world, p);
});
