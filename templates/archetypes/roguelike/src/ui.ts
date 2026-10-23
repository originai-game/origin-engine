// Lamp Dive's screens on og.ui and the style in assets/styles/lampdive (the
// engine's pixel pack in tomb colours): the title, the HUD (health, strike,
// armour and gold in the top-left corner, the floor and pause in the
// top-right, the draught and, on a touch screen, Wait and a d-pad along the
// bottom), the pause menu, the settings dialog and the run's summary when
// the lamp goes out or the relic is taken. Screens change through a
// dissolve (a fade with reduced motion). Every string is a key of
// assets/strings/*.ogt.
//
// The Ui system also starts dives (a fresh seed from og.Rng), ends them a
// beat after the lamp goes out or the relic is taken, sizes the camera for
// the surface, sets the buses and keeps the settings and the dive record
// in og.platform (the record written at the revision read, and merged when
// another device wrote it first).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { FOES, Game, Phase, Prefs, Tuning } from "./components.ts";
import { lazyTuning, must, OVER_DELAY } from "./systems.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** The floors of a dive. */
export const FLOORS = 2;

/** The tokens' spacing scale in CSS px (og.ui.frameTheme answers it too, but builds the whole theme each call). */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** A new dive: a fresh seed, the lamp-bearer whole, floor 1 carved on this tick. */
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
  g.lock = 0;
  g.queued = 0;
  g.foesTurn = false;
  g.newRecord = false;
  g.build = true;
  g.dives += 1;
  saveRecord(world, g);
}

/** The dive record ({dives, relics}) written at the revision it was read at. */
function saveRecord(world: World, g: GameV): void {
  const r = og.platform.storeSet(world, { key: "record", value: { dives: g.dives, relics: g.relics }, revision: g.recordRevision });
  if (r.ok) g.recordWrite = r.value;
}

function applyPrefs(world: World, p: PrefsV): void {
  const audio = world.resource(og.Audio);
  if (audio.ok) {
    audio.value.master = p.master;
    audio.value.music = p.music;
    audio.value.sfx = p.effects;
    audio.value.ui = p.effects;
  }
}

function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function prefsValue(p: PrefsV) {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion };
}

/**
 * og.platform: the settings and the record asked for on tick 0 and applied
 * when they answer; a record write refused for a stale revision (another
 * device dived meanwhile) is read again, merged (the stored counts plus
 * what this device added since it last read or wrote them) and written at
 * the new revision.
 */
function platform(world: World, g: GameV, p: PrefsV): void {
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
        if (at >= 0) setLanguage(world, p, at);
        applyPrefs(world, p);
      }
    }
  }
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

function hud(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const touch = vp.coarsePointer;
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Health", () => {
      og.ui.icon("Heart", "heart", { size: 22, tint: { token: g.hp * 3 <= g.hpMax ? "danger" : "ink" } });
      og.ui.meter("Bar", g.hp, g.hpMax, { kind: "health", label: `${g.hp}/${g.hpMax}`, width: px(touch ? 110 : 150) });
    });
    capsule("Stats", () => {
      og.ui.icon("Sword", "sword", { size: 20 });
      og.ui.counter("Attack", g.attack, { font: "mono", width: px(22) });
      og.ui.icon("Shield", "shield", { size: 20 });
      og.ui.counter("Armour", g.armour, { font: "mono", width: px(22) });
      og.ui.icon("Coin", "coin", { size: 20, tint: { token: "gold" } });
      og.ui.counter("Gold", g.gold, { font: "mono", width: px(40) });
    });
  });
  let paused = false;
  og.ui.anchor("Depth", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Floor", () => og.ui.label("Label", say(world, "hud.floor", { n: g.floor, total: FLOORS }), { step: "lg", nowrap: true }));
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) paused = true;
  });
  if (paused) g.paused = true;
  og.ui.anchor("Hands", { anchor: "bottomLeft", gap: S.md, cross: "end" }, () => {
    og.ui.column("Flask", { cross: "center", gap: S.xxs }, () => {
      if (og.ui.actionButton("Drink", say(world, "hud.drink"), { icon: "potion", size: 60 }).clicked) g.padDrink = true;
      // A refused draught (the Hero system, after this one, refused it last tick) shakes the flask once, with the error sound (og-ui §6).
      if (g.refusedTick === world.tick - 1) og.ui.shake({ sound: true });
      og.ui.label("Count", `× ${g.potions}`, { font: "mono", nowrap: true });
    });
    if (touch && og.ui.actionButton("Wait", say(world, "hud.wait"), { icon: "clock", size: 52 }).clicked) g.padWait = true;
  });
  let dir = 0;
  if (touch) {
    og.ui.anchor("Move", { anchor: "bottomRight" }, () => {
      const d = og.ui.dpad("Pad", { size: 150 });
      dir = d.up ? 1 : d.right ? 2 : d.down ? 3 : d.left ? 4 : 0;
    });
  }
  if (dir !== g.padDir) g.padDir = dir;
}

function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  const before = JSON.stringify(prefsValue(p));
  const m = og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
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
    if (r.selected !== p.language) setLanguage(world, p, r.selected);
    p.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue(p)) !== before) applyPrefs(world, p);
  if (g.settingsOpen && (done || m.dismissed)) {
    g.settingsOpen = false;
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

/** The screens, the dive's start and end, the camera, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning], writes: [Game, Prefs, og.Camera, og.Audio, og.Rng] }, (_q, time, world) => {
  const game = world.resource(Game);
  const prefs = world.resource(Prefs);
  if (!game.ok || !prefs.ok) return;
  const g = game.value;
  const p = prefs.value;
  platform(world, g, p);
  if (time.tick === 0) applyPrefs(world, p);
  // The camera: 13 cells tall on a wide screen, 10 cells across a portrait one.
  const vp = og.ui.frameViewport();
  if (vp.width > 0 && (vp.width !== g.viewW || vp.height !== g.viewH)) {
    g.viewW = vp.width;
    g.viewH = vp.height;
    const cam = world.resource(og.Camera);
    if (cam.ok) cam.value.height = vp.width < vp.height ? (10 * vp.height) / vp.width : 13;
  }
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  // A beat after the lamp goes out or the relic is taken, the summary.
  if (g.phase === Phase.Play && (g.hp <= 0 || g.won)) {
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
  if (shown !== "title") g.leftTitle = true;
  if (shown === "title") {
    og.ui.setNavigation(true);
    const r = og.ui.mainMenu("Title", {
      title: say(world, "title.name"),
      subtitle: say(world, "title.tagline"),
      items: [
        { id: "play", label: say(world, "title.play"), primary: true },
        { id: "settings", label: say(world, "title.settings") },
      ],
      footer: say(world, "title.footer"),
      // The title the game opens on is built settled: the web draws it behind the play gate, where
      // the style's entrance would stand part-way until the tap (og-platform-sdk §3). Later titles enter.
      instant: !g.leftTitle,
    });
    if (!g.settingsOpen && r.selected === "play") startDive(world, g);
    if (r.selected === "settings") g.settingsOpen = true;
  } else if (shown === "play") {
    hud(world, g);
    const m = og.ui.pauseMenu("Pause", {
      open: g.paused && !g.settingsOpen,
      title: say(world, "pause.title"),
      items: [
        { id: "resume", label: say(world, "pause.resume"), primary: true },
        { id: "settings", label: say(world, "pause.settings") },
        { id: "quit", label: say(world, "pause.quit") },
      ],
    });
    if (m.selected === "resume") g.paused = false;
    if (m.selected === "settings") g.settingsOpen = true;
    if (m.selected === "quit") {
      g.paused = false;
      g.phase = Phase.Title;
    }
  } else {
    og.ui.setNavigation(true);
    const c = summary(world, g);
    if (c === "again") startDive(world, g);
    if (c === "menu") g.phase = Phase.Title;
  }
  settings(world, g, p);
  if (g.language !== p.language) g.language = p.language;
});
