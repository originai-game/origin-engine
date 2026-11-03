// Hollowbell's screens on og.ui with the style assets/styles/hollowbell (the
// engine's pixel style pack, recoloured; the look applies it): the title
// over the village (New game, Continue when a save answered, Settings), the
// name dialog of a new game (a text field kept in Prefs.name, a str field),
// the HUD (hearts and glimmers top-left; the three chimes, drawn as screen
// sprites from the items' texture array, and Pause top-right; a prompt by
// the mole or the bell), the mole's talk (the game's letterbox around
// og.ui.dialogue) and his wares, the pause menu, the settings (a modal of
// widgets), and the results under the dawn look. The title, the wares,
// pause and the results are the game's own screen documents
// (assets/ui/*.ogt, ejected with `og ui eject` from the library's
// main-menu, vendor, pause and results). Screens change
// through an iris (a fade with reduced motion). Every string is a key of
// assets/strings/*.ogt.
//
// The Ui system also starts, continues and ends runs, holds the world
// (time scale 0) while paused, talking or shopping, runs the gloam's slow
// motion, counts the run's clock, switches the looks, keeps the settings
// and the save in og.platform (src/save.ts) and counts the accessibility
// tree's unnamed controls for the playtests.

import { og, defineSystem } from "@origingame/engine";
import type { Look, Sound as SoundHandle, TextureArray, Value, World } from "@origingame/engine";
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

/** The UI's sounds and reduced motion: configure keeps the style the look applied. */
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

/** The settings as og.platform keeps them (`og.settings`). */
interface Settings {
  masterVolume?: number;
  musicVolume?: number;
  sfxVolume?: number;
  muted?: boolean;
  reducedMotion?: boolean;
  highContrast?: boolean;
  language?: string;
}

/** The game's screen documents (assets/ui/), loaded once at the start. */
export const SCREENS = ["main-menu", "pause", "shop", "results"] as const;

/**
 * A screen document shown while `on`: mounted (pushed when it is not on the layer stack) and built here with its
 * viewmodel, answering the events it emitted this tick (Back's too); taken off the stack when it is no longer wanted.
 */
function screen(world: World, name: (typeof SCREENS)[number], on: boolean, vm: Record<string, unknown> = {}): string[] {
  if (!on) {
    while (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

function settingsOf(p: PrefsV): Settings {
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

function takeSettings(world: World, p: PrefsV, s: Settings): void {
  if (typeof s.masterVolume === "number") p.master = s.masterVolume;
  if (typeof s.musicVolume === "number") p.music = s.musicVolume;
  if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
  if (typeof s.muted === "boolean") p.muted = s.muted;
  if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
  if (typeof s.highContrast === "boolean") p.highContrast = s.highContrast;
  if (typeof s.language === "string" && s.language !== LANGUAGES[p.language]!.code) setLanguage(world, p, s.language);
}

/** The title's world: the village at the spawn, the hollow as a fresh run has it. */
export function toTitle(world: World, g: GameV): void {
  resetProgress(g);
  g.phase = Phase.Title;
  g.naming = false;
  g.start = true;
  g.warp = true;
  g.warpX = g.spawnX;
  g.warpY = g.spawnY;
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
  g.warpY = g.spawnY;
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
  g.warpY = g.checkY;
  g.area = -1;
  g.loads += 1;
  og.ui.toast(world, say(world, "toast.welcome", { name: p.name === "" ? say(world, "name.placeholder") : p.name }), { tone: "primary" });
  return true;
}

/** The score: 10 a glimmer, 50 a foe, and up to 600 for a quick ring (2 a second under five minutes). */
export function score(g: GameV): number {
  return g.glimmers * 10 + g.kills * 50 + Math.max(0, Math.round((300 - g.time) * 2));
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

function hud(world: World, g: GameV): void {
  // The keys move Wren during play; the talk and the shop are walked by keys and pads like any screen.
  og.ui.setNavigation(g.talking || g.shopping);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "row", gap: S.sm, cross: "center" }, () => {
    capsule("Hearts", () => {
      for (let i = 0; i < HEARTS; i++) og.ui.icon(og.ui.idi("Heart", i), "heart", { size: 22, tint: { token: i < g.hearts ? "danger" : "controlStrong" } });
    });
    capsule("Glimmers", () => {
      og.ui.icon("Coin", "coin", { size: 20, tint: { token: "gold" } });
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
    // A backing in the style's surface colour, then the three chimes from the items' texture array.
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
        og.ui.icon("Glyph", prompt === "hud.ring" ? "music" : "bag", { size: 18, tint: { token: "primary" } });
        og.ui.label("Text", say(world, prompt), { nowrap: true });
      });
    });
  }
}

function talk(world: World, g: GameV, p: PrefsV): void {
  const keeper = p.name === "" ? say(world, "name.placeholder") : p.name;
  const mole = say(world, "talk.mole");
  const lines = [
    { speaker: mole, text: say(world, "talk.1"), portrait: "person" },
    { speaker: mole, text: say(world, "talk.2"), portrait: "person" },
    { speaker: mole, text: say(world, "talk.3"), portrait: "person" },
    { speaker: keeper, text: say(world, "talk.4"), portrait: "music" },
  ];
  if (!g.talking) return;
  const r = letterboxTalk(world, lines, Math.min(g.talkIndex, lines.length - 1));
  g.talkIndex = r.index;
  if (r.finished || r.skipped) {
    g.talking = false;
    g.shopping = true;
    g.shopSelected = 0;
  }
}

/**
 * A talk in the film's frame: a black bar a tenth of the picture top and bottom (run out under a notch, as a
 * film's are), the line in og.ui.dialogue a step above the bottom bar, typed out (a press or Activate shows it
 * whole, the next one goes on), and Skip at the top right, as Back is.
 */
function letterboxTalk(world: World, lines: { speaker: string; text: string; portrait: string }[], index: number): { index: number; finished: boolean; skipped: boolean } {
  const out = { index, finished: false, skipped: false };
  og.ui.setNavigation(true);
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const barH = Math.round((vp.height - sa.top - sa.bottom) * 0.1);
  const black = { color: { r: 0, g: 0, b: 0 } };
  for (const [name, anchor, y, h] of [["Top", "topLeft", -sa.top, barH + sa.top], ["Bottom", "bottomLeft", sa.bottom, barH + sa.bottom]] as const) {
    og.ui.anchor(`Talk.${name}`, { anchor, margin: 0, offsetX: -sa.left, offsetY: y, width: px(vp.width), height: px(h) }, () => {
      og.ui.row("Bar", { width: grow, height: grow, background: black }, () => {});
    });
  }
  const line = lines[index]!;
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  og.ui.anchor("Talk.box", { anchor: "bottom", width: phone ? grow : px(Math.min(760, vp.width - 64)), offsetY: -Math.max(0, barH + S.md - 16) }, () => {
    const r = og.ui.dialogue("Dialogue", { speaker: line.speaker, text: line.text, portraitIcon: line.portrait });
    if (r.advanced) {
      if (index + 1 >= lines.length) out.finished = true;
      else out.index = index + 1;
    }
  });
  og.ui.anchor("Talk.skip", { anchor: "topRight", offsetY: barH + S.sm }, () => {
    if (og.ui.button("Skip", say(world, "talk.skip"), { variant: "quiet", size: "small" }).clicked) out.skipped = true;
  });
  if (og.ui.back()) out.skipped = true;
  return out;
}

function shop(world: World, g: GameV): void {
  if (!g.shopping) return;
  const owned = (id: string) => (id === "wick" && g.wick) || (id === "cloak" && g.cloak);
  const wares = WARES.map((w) => ({ name: say(world, `shop.${w.id}`), doc: say(world, `shop.${w.id}Doc`), price: w.price, icon: w.icon, owned: owned(w.id) }));
  const at = Math.min(g.shopSelected, WARES.length - 1);
  const sel = wares[at]!;
  // assets/ui/shop.ogt: the wares, the selected one's line, Buy; short of the price its Buy shakes, refused.
  const asked = screen(world, "shop", true, { glimmers: g.glimmers, wares, selected: at, name: sel.name, doc: sel.doc, price: sel.price, owned: sel.owned, short: g.glimmers < sel.price, unit: say(world, "shop.currency") });
  if (asked.includes("select")) g.shopSelected = (og.ui.screenVm(world, "shop") as { selected?: number } | null)?.selected ?? at;
  const r = { bought: asked.includes("buy") ? WARES[at]!.id : null, closed: asked.includes("back") };
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
 * (the fixed settings template is gone since U8): it shows exactly the
 * settings Hollowbell has on one page (no Graphics or Controls tab, no
 * master volume) with its prefs and the mixed-script label.
 */
function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  const before = g.settingsOpen ? JSON.stringify(prefsValue(p)) : "";
  const m = og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(460, og.ui.frameViewport().width - 32)) }, () => {
    // A label mixing scripts: the Latin in the style's pixel face, the Chinese from the game's subset.
    og.ui.label("About", say(world, "settings.about"), { color: { token: "inkMuted" }, width: grow });
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
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
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

/** The results' count-up: 0.9 s from their opening (og.ui's frames), ease-out cubic. */
const COUNT_FRAMES = 54;

function results(world: World, g: GameV, p: PrefsV): "again" | "menu" | null {
  const stars = 1 + (g.hurts <= 3 ? 1 : 0) + (g.kills >= 6 ? 1 : 0);
  // When they opened, in og.ui's frames: the count-up and the stars run from there (dropped when the results go).
  let since = og.ui.memory("resultsSince", -1);
  if (since < 0) since = og.ui.tick();
  og.ui.remember("resultsSince", since);
  const k = Math.min(1, (og.ui.tick() - since) / COUNT_FRAMES);
  const asked = screen(world, "results", true, {
    title: say(world, "results.title"),
    score: k >= 1 ? g.score : Math.round(g.score * (1 - (1 - k) ** 3)),
    best: g.best > 0 ? say(world, "results.best", { best: String(g.best).replace(/\B(?=(\d{3})+(?!\d))/g, ",") }) : "",
    newBest: g.newBest,
    landed: k >= 1,
    stats: [
      { label: say(world, "results.keeper"), value: p.name === "" ? say(world, "results.nobody") : p.name },
      { label: say(world, "results.time"), value: seconds(g.time) },
      { label: say(world, "results.glimmers"), value: `${g.glimmers}` },
      { label: say(world, "results.beaten"), value: `${g.kills}` },
      { label: say(world, "results.hurts"), value: `${g.hurts}` },
    ],
  });
  if (og.ui.openSlot("results/Stars")) {
    og.ui.starRating("Rating", { earned: stars, max: 3, size: 44, revealAt: since, name: say(world, "results.stars", { earned: stars, max: 3 }) });
    og.ui.close();
  }
  return asked.includes("again") ? "again" : asked.includes("menu") ? "menu" : null;
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
      takeSettings(world, p, s as Settings);
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

    const want = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
    const shown = og.ui.screenTransition("Screens", want, { kind: p.reducedMotion ? "fade" : "iris", durationMs: 600 }).shown;
    if (shown !== "title") g.leftTitle = true;
    if (shown === "title") {
      og.ui.setNavigation(true);
      const saved = savedRecord(g);
      // assets/ui/main-menu.ogt: Continue (the primary with a save), New game, Settings. It has no entrance: the
      // title the game opens on is built settled, as the web draws it behind the play gate (og-platform-sdk §3).
      const asked = screen(world, "main-menu", true, { canContinue: saved !== null });
      if (saved !== null) {
        og.ui.anchor("Slot", { anchor: "bottomLeft" }, () => {
          capsule("Saved", () => {
            og.ui.icon("Glyph", "music", { size: 18, tint: { token: "primary" } });
            og.ui.label("Text", say(world, "title.saved", { name: saved.name === "" ? say(world, "name.placeholder") : saved.name, n: chimesIn(saved) }), { nowrap: true });
          });
        });
      }
      if (!g.settingsOpen && !g.naming && g.phase === Phase.Title) {
        if (asked.includes("new")) g.naming = true;
        if (asked.includes("continue")) continueRun(world, g, p);
      }
      if (asked.includes("settings")) g.settingsOpen = true;
      naming(world, g, p);
    } else if (shown === "play") {
      hud(world, g);
      if (g.phase === Phase.Play && !g.paused && !g.settingsOpen && !g.talking && !g.shopping && og.ui.back()) g.paused = true;
      // assets/ui/pause.ogt: Resume (and Back), Settings, Save and quit.
      const asked = screen(world, "pause", g.paused && !g.settingsOpen);
      if (asked.includes("resume")) g.paused = false;
      if (asked.includes("settings")) g.settingsOpen = true;
      if (asked.includes("quit")) {
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
    // The screen documents of the screens not shown leave the layer stack.
    if (shown !== "title") screen(world, "main-menu", false);
    if (shown !== "play") screen(world, "pause", false);
    if (shown !== "play" || !g.shopping) screen(world, "shop", false);
    if (shown === "title" || shown === "play") screen(world, "results", false);
    settings(world, g, p);

    // The accessibility tree, for the playtests: every half second, and every tick a dialog is open.
    if (!g.settingsOpen && !g.paused && !g.naming && !g.shopping && time.tick % 30 !== 0) return;
    const tree = og.ui.a11y();
    const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
    if (unnamed !== g.a11yUnnamed) g.a11yUnnamed = unnamed;
    if (tree.nodes.length !== g.a11yNodes) g.a11yNodes = tree.nodes.length;
  },
);
