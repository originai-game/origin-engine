// Hollowbell's screens on og.ui with the skin assets/skins/hollowbell (the
// engine's pixel style pack, recoloured; the look applies it): the title
// over the village (New game, Continue when a save answered, Settings), the
// name dialog of a new game (a text field kept in Prefs.name, a str field),
// the HUD (hearts and glimmers top-left; the three chimes, drawn as screen
// sprites from the items' texture array, and Pause top-right; a prompt by
// the mole or the bell), the mole's talk (og.ui.cutscene) and his wares
// (og.ui.shopScreen), the pause menu, the settings (a modal of widgets),
// and the results (og.ui.resultsScreen) under the dawn look. Screens change
// through an iris (a fade with reduced motion). Every string is a key of
// assets/strings/*.ogt.
//
// The Ui system also starts, continues and ends runs, holds the world
// (time scale 0) while paused, talking or shopping, runs the gloam's slow
// motion, counts the run's clock, switches the looks, keeps the settings
// and the save in og.platform (src/save.ts) and counts the accessibility
// tree's unnamed controls for the playtests.

import { og, defineSystem } from "@origingame/engine";
import type { GameSettings, Look, Sound as SoundHandle, TextureArray, Value, World } from "@origingame/engine";
import { CHIMES, Game, HEARTS, Layer, NAME_CHARS, Phase, Prefs, Sound } from "./components.ts";
import { say } from "./feel.ts";
import { apply, askRecords, chimesIn, followRecords, resetProgress, savedRecord, writeBest, writeSave } from "./save.ts";
import { must } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks from the bell's first toll to the results. */
export const RING_TICKS = 150;

const S = { xs: 4, sm: 8, md: 12, lg: 16 } as const;
const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

/** The shop's wares (the names and lines are string keys). */
export const WARES = [
  { id: "cake", price: 6, icon: "heart" },
  { id: "wick", price: 10, icon: "fire" },
  { id: "cloak", price: 16, icon: "leaf" },
] as const;

/** A name as Prefs keeps it: at most NAME_CHARS characters. */
export function setName(p: PrefsV, name: string): void {
  p.name = [...name].slice(0, NAME_CHARS).join("");
}

/** The UI's sounds and reduced motion: configure keeps the skin the look applied. */
export function configureUi(world: World, p: PrefsV): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, {
    // Toasts at the top centre: the bottom is the talk's dialogue box and the prompts'.
    toastAnchor: "top",
    reducedMotion: p.reducedMotion,
    contrast: p.highContrast ? "high" : "standard",
    sounds: { press: sound("audio/ui-click.ogt"), success: sound("audio/ui-select.ogt"), open: sound("audio/ui-select.ogt"), close: sound("audio/ui-back.ogt"), error: sound("audio/ui-error.ogt") },
  }));
}

function applyPrefs(world: World, p: PrefsV): void {
  const audio = must(world.resource(og.Audio));
  audio.master = p.muted ? 0 : p.master;
  audio.music = p.music;
  audio.sfx = p.effects;
  audio.ui = p.effects;
  configureUi(world, p);
}

export function setLanguage(world: World, p: PrefsV, code: string): void {
  const at = LANGUAGES.findIndex((l) => l.code === code);
  p.language = at < 0 ? 0 : at;
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function settingsOf(p: PrefsV): GameSettings {
  return {
    masterVolume: p.master,
    musicVolume: p.music,
    sfxVolume: p.effects,
    muted: p.muted,
    reducedMotion: p.reducedMotion,
    highContrast: p.highContrast,
    language: LANGUAGES[p.language]!.code,
  };
}

function prefsValue(p: PrefsV) {
  return { ...settingsOf(p), name: p.name };
}

function takeSettings(world: World, p: PrefsV, s: GameSettings): void {
  if (typeof s.masterVolume === "number") p.master = s.masterVolume;
  if (typeof s.musicVolume === "number") p.music = s.musicVolume;
  if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
  if (typeof s.muted === "boolean") p.muted = s.muted;
  if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
  if (typeof s.highContrast === "boolean") p.highContrast = s.highContrast;
  if (typeof s.language === "string" && s.language !== LANGUAGES[p.language]!.code) setLanguage(world, p, s.language);
}

/** The title's world: the village at the spawn, the hollow as a fresh run has it. */
/**
 * Where a warp puts Wren's feet: 5 cm over the floor it names. Put exactly on
 * a tilemap's floor, the controller sinks 7 mm into the tiles' colliders at
 * some places and cannot run sideways until it jumps (FINDINGS.md #15);
 * from 5 cm up it lands on its first step.
 */
export const WARP_LIFT = 0.05;

export function toTitle(world: World, g: GameV): void {
  resetProgress(g);
  g.phase = Phase.Title;
  g.naming = false;
  g.start = true;
  g.warp = true;
  g.warpX = g.spawnX;
  g.warpY = g.spawnY + WARP_LIFT;
  void world;
}

/** A new run from the spawn. */
export function newRun(world: World, g: GameV): void {
  resetProgress(g);
  g.phase = Phase.Play;
  g.naming = false;
  g.start = true;
  g.warp = true;
  g.warpX = g.spawnX;
  g.warpY = g.spawnY + WARP_LIFT;
  g.area = -1;
  void world;
}

/** A run continued from the save: by its last lantern, with what it had. */
export function continueRun(world: World, g: GameV, p: PrefsV): boolean {
  const r = savedRecord(g);
  if (r === null) return false;
  apply(g, p, r);
  g.phase = Phase.Play;
  g.naming = false;
  g.start = true;
  g.warp = true;
  g.warpX = g.checkX;
  g.warpY = g.checkY + WARP_LIFT;
  g.area = -1;
  g.loads += 1;
  og.ui.toast(world, say(world, "toast.welcome", { name: p.name === "" ? say(world, "name.placeholder") : p.name }), { tone: "accent" });
  return true;
}

/** The score: 10 a glimmer, 50 a foe, and up to 600 for a quick ring (2 a second under five minutes). */
export function score(g: GameV): number {
  return g.glimmers * 10 + g.kills * 50 + Math.max(0, Math.round((300 - g.time) * 2));
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "capsule", background: { token: "surface" } }, build);
}

function hud(world: World, g: GameV): void {
  // The keys move Wren during play; the talk and the shop are walked by keys and pads like any screen.
  og.ui.setNavigation(g.talking || g.shopping);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "row", gap: S.sm, cross: "center" }, () => {
    capsule("Hearts", () => {
      for (let i = 0; i < HEARTS; i++) og.ui.icon(og.ui.idi("Heart", i), "heart", { size: 22, tint: { token: i < g.hearts ? "danger" : "fillStrong" } });
    });
    capsule("Glimmers", () => {
      og.ui.icon("Coin", "coin", { size: 20, tint: { token: "accent" } });
      og.ui.label("Count", `${g.glimmers}`, { step: "lg", font: "mono", nowrap: true, width: px(34) });
    });
  });
  let paused = false;
  og.ui.anchor("Bell", { anchor: "topRight", direction: "row", gap: S.sm, cross: "center" }, () => {
    // Room for the chimes, drawn below as screen sprites: a plain row, no frame, because og.ui draws
    // over every screen sprite whatever its layer (FINDINGS.md #8).
    og.ui.row("Chimes", { width: px(CHIMES * 26 + 16), height: px(34) }, () => {});
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 44 }).clicked) paused = true;
  });
  if (paused) g.paused = true;
  // The chimes: screen sprites from the items' texture array over the capsule's slots (one draw, whichever layer each shows).
  const slots = og.ui.bounds("Bell/Chimes");
  if (slots !== null) {
    // A backing in the skin's surface colour, then the three chimes from the items' texture array.
    og.render.sprite(world, { screen: true, position: { x: slots.x, y: slots.y, z: 0 }, anchor: { x: 0, y: 0 }, width: slots.width, height: slots.height, color: { r: 0.024, g: 0.019, b: 0.05 }, opacity: 0.9, layer: 0 });
    for (let k = 0; k < CHIMES; k++) {
      const have = ((g.chimeMask >>> k) & 1) === 1;
      og.render.sprite(world, {
        array: g.items as unknown as TextureArray,
        arrayLayer: Layer.Chime + (have ? Math.floor(g.time * 6 + k) % 4 : 0),
        screen: true,
        position: { x: slots.x + 8 + k * 26 + 13, y: slots.y + slots.height / 2, z: 0 },
        width: 24,
        height: 24,
        anchor: { x: 0.5, y: 0.5 },
        color: have ? { r: 1, g: 1, b: 1 } : { r: 0.25, g: 0.22, b: 0.35 },
        opacity: have ? 1 : 0.8,
        layer: 1,
      });
    }
  }
  const prompt = g.nearMole && !g.talking && !g.shopping ? "hud.talk" : g.nearBell && g.ringing === 0 ? (g.chimes >= CHIMES ? "hud.ring" : "") : "";
  if (prompt !== "" && g.respawn === 0) {
    og.ui.anchor("Prompt", { anchor: "bottom", offsetY: -24 }, () => {
      capsule("Hint", () => {
        og.ui.icon("Glyph", prompt === "hud.ring" ? "music" : "bag", { size: 18, tint: { token: "accent" } });
        og.ui.label("Text", say(world, prompt), { nowrap: true });
      });
    });
  }
}

function talk(world: World, g: GameV, p: PrefsV): void {
  const keeper = p.name === "" ? say(world, "name.placeholder") : p.name;
  const mole = say(world, "talk.mole");
  const r = og.ui.cutscene("Talk", {
    open: g.talking,
    index: g.talkIndex,
    lines: [
      { speaker: mole, text: say(world, "talk.1"), portrait: "person" },
      { speaker: mole, text: say(world, "talk.2"), portrait: "person" },
      { speaker: mole, text: say(world, "talk.3"), portrait: "person" },
      { speaker: keeper, text: say(world, "talk.4"), portrait: "music" },
    ],
  });
  if (!g.talking) return;
  // The talk's box does not take the focus when it opens, so Enter, Space and the pad's A would do
  // nothing until Tab (FINDINGS.md #10): move the focus to it.
  if (!og.ui.focus(0).endsWith("Dialogue")) og.ui.navigate(world, "next");
  g.talkIndex = r.index;
  if (r.finished || r.skipped) {
    g.talking = false;
    g.shopping = true;
  }
}

function shop(world: World, g: GameV): void {
  if (!g.shopping) return;
  const owned = (id: string) => (id === "wick" && g.wick) || (id === "cloak" && g.cloak);
  const r = og.ui.shopScreen("Shop", {
    title: say(world, "shop.title"),
    currency: say(world, "shop.currency"),
    currencyIcon: "coin",
    balance: g.glimmers,
    items: WARES.map((w) => ({ id: w.id, name: say(world, `shop.${w.id}`), description: say(world, `shop.${w.id}Doc`), price: w.price, icon: w.icon, owned: owned(w.id) })),
  });
  if (r.bought !== null) {
    const w = WARES.find((x) => x.id === r.bought);
    if (w !== undefined && g.glimmers >= w.price && !owned(w.id)) {
      if (w.id === "cake" && g.hearts >= HEARTS) {
        og.ui.toast(world, say(world, "toast.full"), { tone: "warn" });
      } else {
        g.glimmers -= w.price;
        g.bought += 1;
        g.sfx |= Sound.Buy;
        if (w.id === "cake") g.hearts = Math.min(HEARTS, g.hearts + 1);
        if (w.id === "wick") g.wick = true;
        if (w.id === "cloak") g.cloak = true;
        og.ui.toast(world, say(world, "toast.bought", { item: say(world, `shop.${w.id}`) }), { tone: "success" });
      }
    }
  }
  if (r.closed) g.shopping = false;
}

function naming(world: World, g: GameV, p: PrefsV): void {
  let begin = false;
  let back = false;
  const m = og.ui.modal("Naming", g.naming, { title: say(world, "name.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("NameRow", { width: grow, gap: S.md, cross: "center" }, () => {
      og.ui.label("Label", say(world, "name.label"), { nowrap: true });
      const f = og.ui.textField("Name", p.name, { placeholder: say(world, "name.placeholder"), maxChars: NAME_CHARS });
      if (f.value !== p.name) setName(p, f.value);
      if (f.response.submitted) begin = true;
    });
    og.ui.row("Footer", { width: grow, gap: S.sm, align: "end" }, () => {
      if (og.ui.button("Back", say(world, "name.back")).clicked) back = true;
      if (og.ui.button("Begin", say(world, "name.begin"), { variant: "primary" }).clicked) begin = true;
    });
  });
  if (!g.naming) return;
  if (begin) {
    newRun(world, g);
    must(og.platform.saveSettings(world, prefsValue(p)));
  } else if (back || m.dismissed) {
    g.naming = false;
  }
}

/**
 * The settings: the music and effects sliders, the language, reduced motion
 * and high contrast, and a label mixing Latin and Chinese. Built from widgets
 * rather than og.ui.settingsScreen, whose words (Settings, Audio, Graphics,
 * Master, Mute everything, Done...) stay English in every locale: SettingsDesc
 * has no field for them (FINDINGS.md #11).
 */
function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  const before = g.settingsOpen ? JSON.stringify(prefsValue(p)) : "";
  const m = og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(460, og.ui.frameViewport().width - 32)) }, () => {
    // A label mixing scripts: the Latin in the skin's pixel face, the Chinese from the game's subset.
    og.ui.label("About", say(world, "settings.about"), { color: { token: "fgMuted" }, width: grow });
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(120), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "fgMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, p, LANGUAGES[r.selected]!.code);
    const motion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    if (motion !== p.reducedMotion) p.reducedMotion = motion;
    const contrast = og.ui.toggle("Contrast", p.highContrast, say(world, "settings.contrast")).on;
    if (contrast !== p.highContrast) p.highContrast = contrast;
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

function results(world: World, g: GameV, p: PrefsV): "again" | "menu" | null {
  const stars = 1 + (g.hurts <= 3 ? 1 : 0) + (g.kills >= 6 ? 1 : 0);
  const r = og.ui.resultsScreen("Results", {
    open: g.phase === Phase.Over,
    title: say(world, "results.title"),
    score: g.score,
    best: g.best,
    newBest: g.newBest,
    bestLabel: say(world, "results.best"),
    newBestLabel: say(world, "results.newBest"),
    stars,
    maxStars: 3,
    stats: [
      { label: say(world, "results.keeper"), value: p.name === "" ? say(world, "results.nobody") : p.name },
      { label: say(world, "results.time"), value: seconds(g.time) },
      { label: say(world, "results.glimmers"), value: `${g.glimmers}` },
      { label: say(world, "results.beaten"), value: `${g.kills}` },
      { label: say(world, "results.hurts"), value: `${g.hurts}` },
    ],
    actions: [
      { id: "again", label: say(world, "results.again"), primary: true },
      { id: "menu", label: say(world, "results.menu") },
    ],
  });
  return r.selected === "again" ? "again" : r.selected === "menu" ? "menu" : null;
}

/** The bell rung and the tolls heard: the score, the best, the results under the dawn look. */
function ring(world: World, g: GameV): void {
  if (g.ringing === 0 || g.phase !== Phase.Play || g.held) return;
  g.ringing += 1;
  if (g.ringing < RING_TICKS) return;
  g.phase = Phase.Over;
  g.score = score(g);
  if (g.score > g.best) {
    g.best = g.score;
    g.newBest = true;
    writeBest(world, g);
  }
  // A finished run's save starts over: the bell is whole again.
  g.saveJson = "";
  g.hasSave = false;
  must(og.platform.storeSet(world, { key: "save", value: null }));
}

function looks(world: World, g: GameV): void {
  const want = g.phase === Phase.Over ? 1 : 0;
  if (want === g.look) return;
  must(og.render.applyLook(world, (want === 1 ? g.dawnLook : g.mainLook) as unknown as Look));
  g.look = want;
}

export const Ui = defineSystem(
  "Ui",
  { reads: [og.Gamepad, og.Time], writes: [Game, Prefs, og.Audio, og.RenderSettings, og.PostEffects, og.Stylise, og.MaterialStyle, og.Fog, og.Sky, og.Environment] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const p = must(world.resource(Prefs));
    if (time.tick === 0) {
      askRecords(world, g, p);
      applyPrefs(world, p);
    }
    const s = followRecords(world, g, p);
    if (s !== null) {
      takeSettings(world, p, s as GameSettings);
      if (typeof s.name === "string" && p.name === "") setName(p, s.name);
      applyPrefs(world, p);
    }

    // The world holds still under a dialog; the gloam's fall runs slow for 40 real ticks.
    const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen || g.talking || g.shopping);
    if (g.slowTicks > 0 && !hold) g.slowTicks -= 1;
    const scale = hold ? 0 : g.slowTicks > 0 ? 0.3 : 1;
    const now = must(world.resource(og.Time)).scale;
    if (scale !== now) must(og.time.setScale(world, scale));
    g.held = hold;
    if (g.phase === Phase.Play && !hold) g.time += time.scaledDt;
    ring(world, g);
    looks(world, g);
    if (g.musicChunks === 0 && g.music !== 0 && time.tick % 30 === 0) {
      const info = og.load.soundInfo(world, g.music as unknown as SoundHandle);
      if (info.ok && info.value.chunks !== 0) g.musicChunks = info.value.chunks;
    }

    const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
    const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "iris", durationMs: 600 }).shown;
    if (shown === "title") {
      og.ui.setNavigation(true);
      const saved = savedRecord(g);
      const r = og.ui.mainMenu("Title", {
        title: say(world, "title.name"),
        subtitle: say(world, "title.tagline"),
        items: [
          { id: "continue", label: say(world, "title.continue"), primary: saved !== null, disabled: saved === null },
          { id: "new", label: say(world, "title.new"), primary: saved === null },
          { id: "settings", label: say(world, "title.settings") },
        ],
        footer: say(world, "title.footer"),
      });
      if (saved !== null) {
        og.ui.anchor("Slot", { anchor: "bottomLeft" }, () => {
          capsule("Saved", () => {
            og.ui.icon("Glyph", "music", { size: 18, tint: { token: "accent" } });
            og.ui.label("Text", say(world, "title.saved", { name: saved.name === "" ? say(world, "name.placeholder") : saved.name, n: chimesIn(saved) }), { nowrap: true });
          });
        });
      }
      if (!g.settingsOpen && !g.naming && g.phase === Phase.Title) {
        if (r.selected === "new") g.naming = true;
        if (r.selected === "continue") continueRun(world, g, p);
      }
      if (r.selected === "settings") g.settingsOpen = true;
      naming(world, g, p);
    } else if (shown === "play") {
      hud(world, g);
      if (g.phase === Phase.Play && !g.paused && !g.settingsOpen && !g.talking && !g.shopping && og.ui.back()) g.paused = true;
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
        writeSave(world, g, p);
        toTitle(world, g);
      }
      talk(world, g, p);
      shop(world, g);
    } else {
      og.ui.setNavigation(true);
      const c = results(world, g, p);
      if (c === "again") newRun(world, g);
      if (c === "menu") toTitle(world, g);
    }
    settings(world, g, p);

    // The accessibility tree, for the playtests: every half second, and every tick a dialog is open.
    if (!g.settingsOpen && !g.paused && !g.naming && !g.shopping && time.tick % 30 !== 0) return;
    const tree = og.ui.a11y();
    const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
    if (unnamed !== g.a11yUnnamed) g.a11yUnnamed = unnamed;
    if (tree.nodes.length !== g.a11yNodes) g.a11yNodes = tree.nodes.length;
  },
);
