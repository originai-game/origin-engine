// Lantern Vale's screens on og.ui with the skin assets/styles/lantern (the
// engine's fantasy pack, applied by the look): the title over a slow pass
// above the valley, the HUD (hearts and lanterns top-left, the dusk clock
// and Pause top-right, the lighting prompt and its progress at the
// bottom), the pause menu, the settings (music and effects sliders, the
// lamplighter's name in a text field, the language, reduced motion, and a
// label mixing Latin and Chinese), and the results on og.ui.resultsScreen
// under a second look (looks/dawn.ogt). Screens change through a dissolve
// (a fade with reduced motion). Every string is a key of
// assets/strings/*.ogt.
//
// The Ui system also starts runs, moves the dusk clock, ends runs (every
// lantern lit, the hearts gone, or night), holds the world while paused,
// switches the looks, follows the far side's streaming group, keeps the
// settings in og.platform and counts the accessibility tree's unnamed
// controls for the playtests.

import { og, defineSystem, STREAM_RESIDENT } from "@origingame/engine";
import type { Look, Sound as SoundHandle, Value, World } from "@origingame/engine";
import { Ending, Game, HEARTS, LANTERN_COUNT, NAME_CHARS, Phase, Prefs, Sound, Tuning } from "./components.ts";
import { HOURS, must, setHour } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks between a run's end and its results: the moment plays out first. */
export const OVER_DELAY = 75;

/** The tokens' spacing scale in CSS px. */
const S = { xs: 4, sm: 8, md: 12, lg: 16 } as const;
const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

/** A name as Prefs keeps it: at most NAME_CHARS characters (a saved one from elsewhere may be longer). */
function setName(p: PrefsV, name: string): void {
  p.name = [...name].slice(0, NAME_CHARS).join("");
}

/** A new run: the hero at the start, hearts full, the lanterns dark, the clock at sunset. */
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
  g.steps = 0;
  g.overTicks = 0;
  g.newBest = false;
  g.start = true;
  setHour(world, g.hour);
}

/** Back to the title: the vale reset behind it, the lanterns glowing. */
export function toTitle(world: World, g: GameV): void {
  g.phase = Phase.Title;
  g.paused = false;
  g.ending = Ending.None;
  g.start = true;
  g.hour = HOURS.title;
  setHour(world, g.hour);
}

/**
 * The UI's sounds and reduced motion: og.ui.configure keeps every field it
 * is not given, so the skin the look applied stays.
 */
export function configureUi(world: World, reducedMotion: boolean): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, {
    reducedMotion,
    sounds: { press: sound("audio/ui-click.ogt"), success: sound("audio/ui-select.ogt"), open: sound("audio/ui-select.ogt"), close: sound("audio/ui-back.ogt"), error: sound("audio/ui-error.ogt") },
  }));
}

function applyPrefs(world: World, p: PrefsV): void {
  const audio = must(world.resource(og.Audio));
  audio.music = p.music;
  audio.sfx = p.effects;
  audio.ui = p.effects;
  configureUi(world, p.reducedMotion);
}

function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function prefsValue(p: PrefsV) {
  return { musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion, name: p.name };
}

/** og.platform: the settings asked for on tick 0 and applied when they answer. */
function platform(world: World, p: PrefsV): void {
  if (world.tick === 0) {
    p.loadRequest = og.platform.loadSettings(world);
    return;
  }
  if (p.loadRequest === 0) return;
  const r = og.platform.result(world, p.loadRequest);
  if (r === null) return;
  p.loadRequest = 0;
  const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
  if (!s || typeof s !== "object") return;
  if (typeof s.musicVolume === "number") p.music = s.musicVolume;
  if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
  if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
  if (typeof s.name === "string") setName(p, s.name);
  const at = LANGUAGES.findIndex((l) => l.code === s.language);
  if (at >= 0) setLanguage(world, p, at);
  applyPrefs(world, p);
}

/** The clock and the endings: night at HOURS.night, the hearts gone, or (src/lanterns.ts) all four lit. */
function clock(world: World, g: GameV, dt: number): void {
  const t = must(world.resource(Tuning));
  if (g.phase !== Phase.Play) return;
  if (g.ending === Ending.None) {
    g.time += dt;
    g.hour = Math.min(HOURS.night, HOURS.start + (g.time / t.duskSeconds) * (HOURS.night - HOURS.start));
    if (g.hearts <= 0) {
      g.ending = Ending.Caught;
      g.sfx |= Sound.Lose;
    } else if (g.hour >= HOURS.night) {
      g.ending = Ending.Night;
      g.sfx |= Sound.Lose;
    }
  }
  setHour(world, g.hour);
  if (g.ending === Ending.None) return;
  g.overTicks += 1;
  if (g.overTicks >= OVER_DELAY) {
    g.phase = Phase.Over;
    if (g.ending === Ending.Lit && (g.best === 0 || score(world, g) > g.best)) {
      g.newBest = true;
      g.best = score(world, g);
    }
    // The results are the next dawn: the night has passed.
    g.hour = HOURS.dawn;
    setHour(world, g.hour);
  }
}

/** The results look on the results screen, the dusk look everywhere else. */
function looks(world: World, g: GameV): void {
  const want = g.phase === Phase.Over ? 1 : 0;
  if (want === g.look) return;
  must(og.render.applyLook(world, (want === 1 ? g.dawn : g.dusk) as unknown as Look));
  g.look = want;
}

const STREAM_STATES = ["unloaded", "queued", "loading", "resident", "failed"] as const;

/**
 * The far side's streaming group: og.StreamEvent says when it changes (a
 * toast when it becomes resident in a run), og.stream.status where it is
 * and og.stream.budget what the groups take, every tenth tick.
 */
function stream(world: World, g: GameV): void {
  for (const e of world.events(og.StreamEvent)) {
    if (e.group !== "far-side" || e.state !== STREAM_RESIDENT) continue;
    g.farLoads += 1;
    if (g.phase === Phase.Play) og.ui.toast(world, say(world, "hud.far"));
  }
  const status = og.stream.status(world, "far-side");
  if (status.ok) {
    const state = STREAM_STATES.indexOf(status.value.state);
    if (state !== g.far) g.far = state;
  }
  if (world.tick % 10 !== 0) return;
  const b = og.stream.budget(world);
  if (b.ok && b.value.used !== g.farUsed) g.farUsed = b.value.used;
  if (g.musicChunks === 0 && g.music !== 0) {
    const info = og.load.soundInfo(world, g.music as unknown as SoundHandle);
    if (info.ok && info.value.chunks !== 0) g.musicChunks = info.value.chunks;
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

function clockText(hour: number): string {
  const h = Math.floor(hour);
  const m = Math.floor((hour - h) * 60);
  return `${h}:${m < 10 ? "0" : ""}${m}`;
}

function hud(world: World, g: GameV): void {
  const t = must(world.resource(Tuning));
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Hearts", () => {
      for (let i = 0; i < HEARTS; i++) og.ui.icon(`Heart${i}`, "heart", { size: 22, tint: { token: i < g.hearts ? "danger" : "controlStrong" } });
    });
    capsule("Lanterns", () => {
      og.ui.icon("Fire", "fire", { size: 22, tint: { token: "primary" } });
      og.ui.label("Count", say(world, "hud.lanterns", { n: g.lit, total: LANTERN_COUNT }), { step: "lg", font: "mono", nowrap: true });
    });
  });
  let paused = false;
  og.ui.anchor("Dusk", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Clock", () => {
      const left = 1 - (g.hour - HOURS.start) / (HOURS.night - HOURS.start);
      og.ui.icon("Glyph", "clock", { size: 20, tint: { token: left < 0.25 ? "danger" : "ink" } });
      og.ui.label("Hour", clockText(g.hour), { font: "mono", nowrap: true, width: px(52) });
      og.ui.meter("Left", Math.max(0, left), 1, { kind: "primary", width: px(110) });
    });
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) paused = true;
  });
  if (paused) g.paused = true;
  if (g.near >= 0 && g.ending === Ending.None) {
    og.ui.anchor("Prompt", { anchor: "bottom", offsetY: -24, direction: "column", cross: "center", gap: S.xs }, () => {
      capsule("Hint", () => {
        og.ui.icon("Fire", "fire", { size: 18, tint: { token: "primary" } });
        og.ui.label("Text", say(world, "hud.hint"), { nowrap: true });
      });
      if (g.kindle > 0) og.ui.progress("Kindle", g.kindle / t.kindleSeconds, { width: px(200) });
    });
  }
}

function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  const before = g.settingsOpen ? JSON.stringify(prefsValue(p)) : "";
  const m = og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(440, og.ui.frameViewport().width - 32)) }, () => {
    // A label mixing scripts: the Latin in the skin's face, the Chinese from the fallback.
    og.ui.label("About", say(world, "settings.about"), { color: { token: "inkMuted" }, width: grow });
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    og.ui.row("NameRow", { width: grow, gap: S.md, cross: "center" }, () => {
      og.ui.label("Label", say(world, "settings.name"), { width: px(110), nowrap: true });
      const f = og.ui.textField("Name", p.name, { placeholder: say(world, "settings.namePlaceholder"), maxChars: NAME_CHARS });
      if (f.value !== p.name) setName(p, f.value);
    });
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, p, r.selected);
    const motion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    if (motion !== p.reducedMotion) p.reducedMotion = motion;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (g.settingsOpen && JSON.stringify(prefsValue(p)) !== before) applyPrefs(world, p);
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

/** The score: 250 a lantern, and on a win a second for each second of dusk left. */
export function score(world: World, g: GameV): number {
  const t = must(world.resource(Tuning));
  return g.lit * 250 + (g.ending === Ending.Lit ? Math.max(0, Math.round(t.duskSeconds - g.time)) : 0);
}

function results(world: World, g: GameV, p: PrefsV): "again" | "menu" | null {
  const won = g.ending === Ending.Lit;
  const stars = won ? Math.max(1, 3 - g.hurts) : 0;
  const name = p.name;
  const r = og.ui.resultsScreen("Results", {
    open: g.phase === Phase.Over,
    title: say(world, won ? "results.won" : g.ending === Ending.Caught ? "results.caught" : "results.night"),
    score: score(world, g),
    best: g.best,
    newBest: g.newBest,
    // og.ui's own words, from the tables too: "{best}" is the kit's to fill.
    bestLabel: say(world, "results.best"),
    newBestLabel: say(world, "results.newBest"),
    stars,
    maxStars: 3,
    stats: [
      { label: say(world, "results.player"), value: name === "" ? say(world, "results.nobody") : name },
      { label: say(world, "results.lanterns"), value: `${g.lit}/${LANTERN_COUNT}` },
      { label: say(world, "results.time"), value: seconds(g.time) },
      { label: say(world, "results.hurts"), value: `${g.hurts}` },
    ],
    actions: [
      { id: "again", label: say(world, "results.again"), primary: true },
      { id: "menu", label: say(world, "results.menu") },
    ],
  });
  return r.selected === "again" ? "again" : r.selected === "menu" ? "menu" : null;
}

/** The screens, the run's start, clock and end, the hold, the looks, the stream, the platform's answers. */
export const Ui = defineSystem(
  "Ui",
  { reads: [Tuning, og.Gamepad, og.StreamEvent], writes: [Game, Prefs, og.Audio, og.DayNight, og.RenderSettings, og.PostEffects, og.Stylise, og.MaterialStyle, og.Fog, og.Sky, og.Environment] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const p = must(world.resource(Prefs));
    platform(world, p);
    if (time.tick === 0) applyPrefs(world, p);
    const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
    if (hold !== g.held) {
      must(og.time.setScale(world, hold ? 0 : 1));
      g.held = hold;
    }
    clock(world, g, time.scaledDt);
    looks(world, g);
    stream(world, g);

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
        // the skin's entrance would stand part-way until the tap (og-platform-sdk §3). Later titles enter.
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
      const c = results(world, g, p);
      if (c === "again") startRun(world, g);
      if (c === "menu") toTitle(world, g);
    }
    settings(world, g, p);
    // The accessibility tree, for the playtests: every half second, and on every tick a dialog is open.
    if (!g.settingsOpen && !g.paused && time.tick % 30 !== 0) return;
    const tree = og.ui.a11y();
    const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
    if (unnamed !== g.a11yUnnamed) g.a11yUnnamed = unnamed;
    if (tree.nodes.length !== g.a11yNodes) g.a11yNodes = tree.nodes.length;
  },
);
