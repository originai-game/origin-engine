// Ember Vale's screens on og.ui and the style in assets/styles/ember (the
// engine's fantasy pack): the title over an orbit of the vale, the HUD
// (hearts and beacons top-left, the dusk clock and pause top-right, the
// kindling prompt and its progress at the bottom; on a touch screen a
// joystick, Jump and Kindle, and a drag on the right half turning the
// view), the pause menu, the settings dialog and
// the run's summary. Screens change through a dissolve (a fade with
// reduced motion). Every string is a key of assets/strings/*.ogt.
//
// The Ui system also starts runs, moves the clock (og.DayNight, held by
// the game so a pause holds it), ends runs (every beacon lit, the hearts
// gone, or night), holds the world while paused, sets the buses and keeps
// the settings and the record in og.platform (the record written at the
// revision read, and merged when another device wrote it first).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { BEACONS, Ending, Game, HEARTS, Phase, Prefs, Sound, Tuning } from "./components.ts";
import { lazyTuning } from "./hero.ts";
import { HOURS, must, setHour } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks between a run's end and its summary: the moment plays out first. */
export const OVER_DELAY = 90;

/** The tokens' spacing scale in CSS px. */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** A new run: the hero at the start, hearts full, the beacons dark, the clock at mid-afternoon. */
export function startRun(world: World, g: GameV): void {
  g.phase = Phase.Play;
  g.ending = Ending.None;
  g.paused = false;
  g.hearts = HEARTS;
  g.hurts = 0;
  g.lit = 0;
  g.litMask = 0;
  g.time = 0;
  g.hour = HOURS.start;
  g.kindle = 0;
  g.near = -1;
  g.jumps = 0;
  g.splashes = 0;
  g.grabs = 0;
  g.alarms = 0;
  g.overTicks = 0;
  g.newBest = false;
  g.start = true;
  g.runs += 1;
  setHour(world, g.hour);
  saveRecord(world, g);
}

/** Back to the title: the vale reset behind it, the beacons burning. */
export function toTitle(world: World, g: GameV): void {
  g.phase = Phase.Title;
  g.paused = false;
  g.ending = Ending.None;
  g.start = true;
  g.hour = HOURS.title;
  setHour(world, g.hour);
}

/**
 * The record ({runs, wins, best}) written at the revision last read or
 * written. One call at a time: while a read or a write is in flight the
 * newer record waits (recordDirty) and goes out when that answers, at the
 * revision the answer gives, so a late answer never stands for a newer
 * local record.
 */
function saveRecord(world: World, g: GameV): void {
  if (g.recordRequest !== 0 || g.recordWrite !== 0) {
    g.recordDirty = true;
    return;
  }
  g.recordDirty = false;
  const r = og.platform.storeSet(world, { key: "record", value: { runs: g.runs, wins: g.wins, best: g.best }, revision: g.recordRevision });
  if (!r.ok) return;
  g.recordWrite = r.value;
  g.recordSentRuns = g.runs;
  g.recordSentWins = g.wins;
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
 * device played meanwhile) is read again, merged (the stored counts plus
 * what this device added since it last read or wrote them, the better of
 * the two best times) and written at the new revision.
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
      const d = (r.ok ? r.data : null) as { value?: { runs?: number; wins?: number; best?: number } | null; revision?: number } | null;
      const runs = d?.value?.runs ?? 0;
      const wins = d?.value?.wins ?? 0;
      const best = d?.value?.best ?? 0;
      g.runs = runs + g.runs - g.recordRuns;
      g.wins = wins + g.wins - g.recordWins;
      if (best > 0 && (g.best === 0 || best < g.best)) g.best = best;
      g.recordRuns = runs;
      g.recordWins = wins;
      if (typeof d?.revision === "number") g.recordRevision = d.revision;
      if (g.recordMerge || g.recordDirty) {
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
        // The store now holds what was sent, at the revision it answers.
        g.recordRevision = (r.data as { revision: number }).revision;
        g.recordRuns = g.recordSentRuns;
        g.recordWins = g.recordSentWins;
        if (g.recordDirty) saveRecord(world, g);
      } else if (r.error?.code === "conflict") {
        // Someone else wrote the record: read it again and write the merge.
        g.recordConflicts += 1;
        g.recordMerge = true;
        const again = og.platform.storeGet(world, "record");
        g.recordRequest = again.ok ? again.value : 0;
      } else if (g.recordDirty) saveRecord(world, g);
    }
  }
}

/** The clock and the endings: night at HOURS.night, the hearts gone, or (the Beacons system) every beacon lit. */
function clock(world: World, g: GameV, dt: number): void {
  const t = lazyTuning(world);
  // The summary's moments (its burst) count on from the run's end.
  if (g.phase === Phase.Over) g.overTicks += 1;
  if (g.phase !== Phase.Play) return;
  if (g.ending === Ending.None) {
    g.time += dt;
    g.hour = Math.min(HOURS.night, HOURS.start + (g.time / t.duskSeconds) * (HOURS.night - HOURS.start));
    if (g.hearts <= 0) g.ending = Ending.Caught;
    else if (g.hour >= HOURS.night) {
      g.ending = Ending.Night;
      g.sfx |= Sound.Night;
    }
  }
  setHour(world, g.hour);
  if (g.ending === Ending.None) return;
  g.overTicks += 1;
  if (g.overTicks === OVER_DELAY) {
    g.phase = Phase.Over;
    if (g.ending === Ending.Dawn) {
      g.wins += 1;
      g.newBest = g.best === 0 || g.time < g.best;
      if (g.newBest) g.best = g.time;
    }
    saveRecord(world, g);
    must(og.platform.telemetry(world, "run.over", { ending: ["none", "dawn", "night", "caught"][g.ending]!, time: Math.round(g.time), lit: g.lit, hurts: g.hurts }));
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

function clockText(g: GameV): string {
  const h = Math.floor(g.hour);
  const m = Math.floor((g.hour - h) * 60);
  return `${h}:${m < 10 ? "0" : ""}${m}`;
}

function hud(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const touch = vp.coarsePointer;
  const t = lazyTuning(world);
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Hearts", () => {
      for (let i = 0; i < HEARTS; i++) og.ui.icon(`Heart${i}`, "heart", { size: 22, tint: { token: i < g.hearts ? "danger" : "controlStrong" } });
    });
    capsule("Beacons", () => {
      og.ui.icon("Fire", "fire", { size: 22, tint: { token: "primary" } });
      og.ui.label("Count", say(world, "hud.beacons", { n: g.lit, total: BEACONS }), { step: "lg", font: "mono", nowrap: true });
    });
  });
  let paused = false;
  og.ui.anchor("Dusk", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Clock", () => {
      const left = 1 - (g.hour - HOURS.start) / (HOURS.night - HOURS.start);
      og.ui.icon("Glyph", "clock", { size: 20, tint: { token: left < 0.25 ? "danger" : "ink" } });
      og.ui.label("Hour", clockText(g), { font: "mono", nowrap: true, width: px(48) });
      og.ui.meter("Left", Math.max(0, left), 1, { kind: "primary", width: px(touch ? 70 : 110) });
    });
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) paused = true;
  });
  if (paused) g.paused = true;
  // The prompt by an unlit beacon, then the kindling's progress.
  if (g.near >= 0 && g.ending === Ending.None) {
    og.ui.anchor("Prompt", { anchor: "bottom", offsetY: touch ? -150 : -24, direction: "column", cross: "center", gap: S.xs }, () => {
      capsule("Hint", () => {
        og.ui.icon("Fire", "fire", { size: 18, tint: { token: "primary" } });
        const pad = must(world.resource(og.Gamepad)).connected();
        og.ui.label("Text", say(world, touch ? "hud.hint.touch" : pad ? "hud.hint.pad" : "hud.hint.keys"), { nowrap: true });
      });
      if (g.kindle > 0) og.ui.progress("Kindle", g.kindle / t.kindleSeconds, { width: px(200) });
    });
  }
  let stickX = 0;
  let stickY = 0;
  let kindle = false;
  if (touch) {
    og.ui.anchor("Move", { anchor: "bottomLeft" }, () => {
      const s = og.ui.joystick("Stick", { size: 140, deadZone: 0.12 });
      // A thumb landing on the stick answers on the ui bus too (Sound.Tap).
      const held = og.ui.memory("Held", 0);
      if (s.active && held === 0) g.sfx |= Sound.Tap;
      if ((s.active ? 1 : 0) !== held) og.ui.remember("Held", s.active ? 1 : 0);
      if (s.active) {
        stickX = s.x;
        stickY = s.y;
      }
    });
    og.ui.anchor("Hands", { anchor: "bottomRight", gap: S.md, cross: "end" }, () => {
      kindle = og.ui.actionButton("Kindle", say(world, "hud.kindle"), { icon: "fire", size: 64, primary: g.near >= 0 }).pressed;
      // Each answers on the ui bus as it is touched (Sound.Tap), then acts.
      if (kindle && !g.padKindle) g.sfx |= Sound.Tap;
      const jump = og.ui.actionButton("Jump", say(world, "hud.jump"), { icon: "play", size: 72 });
      if (jump.clicked) {
        g.padJump = true;
        g.sfx |= Sound.Tap;
      }
    });
  }
  if (stickX !== g.stickX) g.stickX = stickX;
  if (stickY !== g.stickY) g.stickY = stickY;
  if (kindle !== g.padKindle) g.padKindle = kindle;
  look(world, g, touch ? vp : null);
}

/** Radians the view turns per CSS px a finger drags across the look zone. */
const LOOK_RAD_PER_PX = 0.008;

/**
 * The touch view turn: a finger that lands in the look zone (the right
 * half, below the clock and above Jump and Kindle) turns the view as it
 * drags sideways, alongside the stick's thumb, until it lifts. The turn
 * goes to View through Game.lookTurn.
 */
function look(world: World, g: GameV, vp: ReturnType<typeof og.ui.frameViewport> | null): void {
  let turn = 0;
  let id = g.lookId;
  if (vp === null) id = -1;
  else {
    const ptr = must(world.resource(og.Pointers));
    if (id >= 0 && !ptr.down(id)) id = -1;
    if (id >= 0) {
      const x = ptr.position(id).x;
      turn = (x - g.lookX) * LOOK_RAD_PER_PX;
      if (x !== g.lookX) g.lookX = x;
    } else {
      const top = vp.safeArea.top + 96;
      const bottom = vp.height - vp.safeArea.bottom - 220;
      for (let n = 0; n < ptr.count(); n++) {
        if (!ptr.pressed(n)) continue;
        const at = ptr.start(n);
        if (at.x < vp.width / 2 || at.y < top || at.y > bottom) continue;
        id = n;
        g.lookX = ptr.position(n).x;
        break;
      }
    }
  }
  if (id !== g.lookId) g.lookId = id;
  if (turn !== g.lookTurn) g.lookTurn = turn;
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

function seconds(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** The run's summary: how it ended, how long it took, what was lit and lost, the record. */
function summary(world: World, g: GameV): "again" | "menu" | null {
  let chose: "again" | "menu" | null = null;
  const won = g.ending === Ending.Dawn;
  const title = won ? "over.won" : g.ending === Ending.Caught ? "over.caught" : "over.night";
  og.ui.modal("Summary", g.phase === Phase.Over, { title: say(world, title), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", won ? "fire" : g.ending === Ending.Caught ? "skull" : "clock", { size: 48, tint: { token: won ? "primary" : "inkMuted" } }));
      // From a third of a second into the summary, every frame: a burst plays only while it is called.
      if (won && g.overTicks >= OVER_DELAY + 20) og.ui.burst({ count: 14 });
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Time", say(world, "over.time"), seconds(g.time), true);
      og.ui.keyValue("Beacons", say(world, "over.beacons"), `${g.lit}/${BEACONS}`, true);
      og.ui.keyValue("Hurts", say(world, "over.hurts"), `${g.hurts}`, true);
      og.ui.keyValue("Best", say(world, "over.best"), g.best > 0 ? seconds(g.best) + (g.newBest ? " ★" : "") : say(world, "over.none"), false);
    });
    og.ui.label("Record", say(world, "over.record", { runs: g.runs, wins: g.wins }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) chose = "menu";
    });
  });
  return chose;
}

/** The screens, the run's start, clock and end, the hold, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Gamepad, og.Pointers], writes: [Game, Prefs, og.Audio, og.DayNight] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const p = must(world.resource(Prefs));
  platform(world, g, p);
  if (time.tick === 0) applyPrefs(world, p);
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  clock(world, g, time.scaledDt);

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
      footer: say(world, og.ui.frameViewport().coarsePointer ? "title.footer.touch" : "title.footer"),
      // The title the game opens on is built settled: the web draws it behind the play gate, where
      // the style's entrance would stand part-way until the tap (og-platform-sdk §3). Later titles enter.
      instant: !g.leftTitle,
    });
    if (!g.settingsOpen && r.selected === "play" && g.phase === Phase.Title) startRun(world, g);
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
    if (m.selected === "quit") toTitle(world, g);
  } else {
    og.ui.setNavigation(true);
    const c = summary(world, g);
    if (c === "again") startRun(world, g);
    if (c === "menu") toTitle(world, g);
  }
  settings(world, g, p);
  if (g.language !== p.language) g.language = p.language;
});
