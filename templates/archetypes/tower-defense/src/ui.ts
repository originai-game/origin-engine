// Path Guard's screens on og.ui and the skin in assets/skins/pathguard:
// the title, the HUD (the light and the coins in the top-left corner, the
// wave and pause in the top-right, the plant tray on the right on a wide
// screen and along the bottom on a phone), the pause menu, the settings
// dialog and the end card. Screens change through an iris (a fade with
// reduced motion). Every string is a key of assets/strings/*.ogt.
//
// The Ui system also frames the camera on the field for the surface (the
// field beside the tray on a wide screen, between the HUD rows on a
// phone), sets the buses from the settings and reads and writes the
// settings and the best score through og.platform.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Game, Phase, Plant, PLANTS, Prefs, Tuning } from "./components.ts";
import { COLS, cost, lazyTuning, ROWS, WAVES } from "./systems.ts";

/** The languages the settings offer, each named in its own language. */
export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;
type PrefsV = Value<typeof Prefs>;
/**
 * The tokens' spacing scale in CSS px (the same at every density). og.ui's
 * frameTheme answers it too, but builds the whole theme each call (about a
 * millisecond), so the screens read it from here.
 */
const SPACE = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24 } as const;
type Theme = { readonly space: typeof SPACE };
const THEME: Theme = { space: SPACE };

/** A string of the active locale (og.i18n). */
export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

/** The phone layouts: a portrait or landscape phone. */
function phone(): boolean {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
}

/** The sheet frame of a plant, for the tray's pictures. */
const PICTURE = [8, 11, 14] as const;
function uvOf(frame: number): [number, number, number, number] {
  return [(frame % 8) / 8, Math.floor(frame / 8) / 3, ((frame % 8) + 1) / 8, (Math.floor(frame / 8) + 1) / 3];
}

/** A new round: the field cleared, the purse and the light full, the first wave three seconds off. */
export function startRound(world: World, g: GameV, t: TuneV): void {
  g.phase = Phase.Play;
  g.paused = false;
  g.won = false;
  g.rounds += 1;
  g.lives = t.startLives;
  g.gold = t.startGold;
  g.wave = 0;
  g.inWave = false;
  g.toSpawn = 0;
  g.spawned = 0;
  g.breakTicks = 180;
  g.callNext = false;
  g.kills = 0;
  g.leaks = 0;
  g.placed = 0;
  g.refused = 0;
  g.shots = 0;
  g.score = 0;
  g.newBest = false;
  g.selected = Plant.Thorn;
  g.cursorX = 4;
  g.cursorY = 9;
  const grid = g.grid as unknown as never;
  for (let y = 1; y < ROWS - 1; y++) {
    for (let x = 1; x < COLS - 1; x++) {
      must(og.tilemap.setTile(world, { entity: g.map as unknown as Entity, layer: "pads", x, y, tile: -1 }));
      must(og.nav.setCost(world, grid, x, ROWS - 1 - y, 1));
    }
  }
  // The Plants and Visitors systems clear their rows on their next run.
  g.reset = true;
  g.pendingPlants = 0;
}

/** Frames the camera on the field for this surface: beside the tray on a wide screen, between the HUD rows on a phone. */
function frameCamera(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  if (vp.width <= 0 || vp.height <= 0 || (vp.width === g.viewW && vp.height === g.viewH && vp.safeArea.top === g.viewTop)) return;
  g.viewW = vp.width;
  g.viewH = vp.height;
  g.viewTop = vp.safeArea.top;
  const cam = world.resource(og.Camera);
  if (!cam.ok) return;
  const portrait = vp.width < vp.height;
  // The rectangle the field may use, in CSS pixels: room for the HUD's rows and the tray.
  const top = vp.safeArea.top + (portrait ? 84 : 76);
  const bottom = vp.safeArea.bottom + (portrait ? 176 : 24);
  const right = vp.safeArea.right + (portrait ? 0 : 200);
  const left = vp.safeArea.left + (portrait ? 0 : 200);
  const w = vp.width - left - right;
  const h = vp.height - top - bottom;
  const scale = Math.min(w / (COLS + 0.4), h / (ROWS + 0.4));
  const cx = left + w / 2;
  const cy = top + h / 2;
  const c = cam.value;
  c.orthographic = true;
  c.height = vp.height / scale;
  c.position = { x: COLS / 2 - (cx - vp.width / 2) / scale, y: ROWS / 2 + (cy - vp.height / 2) / scale, z: 50 };
}

/** The settings applied: the buses, the language, the motion. */
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

/** The settings as og.platform.saveSettings keeps them. */
function prefsValue(p: PrefsV) {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion };
}

/** The first ticks: ask for the settings and the best score; apply each when it answers. A best written at a stale revision is read again and the higher written back. */
function platform(world: World, g: GameV, p: PrefsV): void {
  if (world.tick === 0) {
    p.loadRequest = og.platform.loadSettings(world);
    const b = og.platform.storeGet(world, "best");
    g.bestRequest = b.ok ? b.value : 0;
    return;
  }
  if (p.loadRequest !== 0) {
    const r = og.platform.result(world, p.loadRequest);
    if (r !== null) {
      p.loadRequest = 0;
      p.loaded = true;
      const v = (r.ok ? r.data : null) as { value?: Record<string, unknown> } | null;
      const s = v?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") p.master = s.masterVolume;
        if (typeof s.musicVolume === "number") p.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, p, at);
      }
    }
  }
  if (g.bestRequest !== 0) {
    const r = og.platform.result(world, g.bestRequest);
    if (r !== null) {
      g.bestRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: unknown; revision?: number } | null;
      if (d && typeof d.value === "number") g.best = Math.max(g.best, d.value);
      if (d && typeof d.revision === "number") g.bestRevision = d.revision;
      // A best this device holds above the stored one (a write refused as stale) is written again at the revision just read.
      if (d && g.best > (typeof d.value === "number" ? d.value : 0)) {
        const w = og.platform.storeSet(world, { key: "best", value: g.best, revision: g.bestRevision });
        if (w.ok) g.bestWrite = w.value;
      }
    }
  }
  if (g.bestWrite !== 0) {
    const r = og.platform.result(world, g.bestWrite);
    if (r !== null) {
      g.bestWrite = 0;
      if (r.ok) g.bestRevision = (r.data as { revision: number }).revision;
      else if (r.error?.code === "conflict") {
        // Another device wrote `best` meanwhile: read it again; the answer keeps the higher and writes it back.
        const again = og.platform.storeGet(world, "best");
        g.bestRequest = again.ok ? again.value : 0;
      }
    }
  }
}

/** One HUD capsule: an icon and a number counting to its value. */
function capsule(t: Theme, id: string, icon: string, value: number, tone: "accent" | "danger" | "fg"): void {
  og.ui.row(id, { gap: t.space.xs, cross: "center", padding: { left: t.space.sm, top: t.space.xs, right: t.space.md, bottom: t.space.xs }, frame: "capsule", background: { token: "surface" }, radius: 999 }, () => {
    og.ui.icon("Icon", icon, { size: 26, tint: { token: tone === "fg" ? "fg" : tone } });
    og.ui.counter("Value", value, { step: "xl", font: "display", durationMs: 300, width: px(56), align: "center" });
  });
}

function hud(world: World, theme: Theme, g: GameV, t: TuneV): void {
  const small = phone();
  og.ui.setNavigation(false);
  og.ui.anchor("Purse", { anchor: "topLeft", gap: theme.space.sm }, () => {
    capsule(theme, "Light", "heart", g.lives, g.lives <= 3 ? "danger" : "fg");
    capsule(theme, "Coins", "coin", g.gold, "accent");
    // A refused placement shakes the purse once, with the error sound (og-ui §6).
    if (g.refusedTick === world.tick) og.ui.shake({ sound: true });
  });
  let paused = false;
  og.ui.anchor("Waves", { anchor: "topRight", gap: theme.space.sm, cross: "center" }, () => {
    og.ui.row("Wave", { cross: "center", padding: { left: theme.space.md, top: theme.space.xs, right: theme.space.md, bottom: theme.space.xs }, frame: "capsule", background: { token: "surface" }, radius: 999 }, () => {
      og.ui.label("Label", say(world, "hud.wave", { n: Math.max(1, g.wave), total: WAVES.length }), { step: "lg", font: "display", nowrap: true });
    });
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) paused = true;
  });
  if (paused) g.paused = true;
  // The tray: the three plants, the selected one in the accent, an unaffordable one disabled; and the next wave.
  og.ui.anchor("Tray", { anchor: small ? "bottom" : "right", direction: small ? "row" : "column", gap: theme.space.sm, cross: "center" }, () => {
    PLANTS.forEach((name, kind) => {
      const price = cost(t, kind);
      const chosen = g.selected === kind;
      og.ui.column(og.ui.idi("Plant", kind), { cross: "center", gap: theme.space.xxs, padding: { left: theme.space.sm, top: theme.space.xs, right: theme.space.sm, bottom: theme.space.sm }, frame: "card", background: { token: "surface" }, radius: 18 }, () => {
        og.ui.row("Top", { cross: "center", gap: theme.space.xs }, () => {
          og.ui.image("Picture", { texture: g.sheet as unknown as never, uv: uvOf(PICTURE[kind]!), width: px(small ? 32 : 44), height: px(small ? 32 : 44) });
          // The price: muted when the purse covers it, in danger's ink when it does not.
          og.ui.label("Price", `${price}`, { step: "lg", font: "display", color: { token: g.gold >= price ? "fgMuted" : "danger" }, nowrap: true, width: px(40), align: "center" });
        });
        if (og.ui.button("Pick", say(world, `tower.${name}`), { variant: chosen ? "primary" : "secondary", size: small ? "medium" : "small", width: px(small ? 86 : 104) }).clicked) g.selected = kind;
      });
    });
    if (!small) callButton(world, g, small);
  });
  // On a phone the call for the next wave sits over the tray, in the thumb's reach.
  if (small) og.ui.anchor("Call", { anchor: "bottom", offsetY: -112 }, () => callButton(world, g, small));
  if (g.wave === 0 && !small) {
    og.ui.anchor("Hint", { anchor: "bottom", padding: { left: theme.space.md, top: theme.space.xs, right: theme.space.md, bottom: theme.space.xs }, frame: "capsule", background: { token: "surface" }, radius: 999 }, () => {
      og.ui.label("Text", say(world, "hud.hint"), { nowrap: true });
    });
  }
}

/** The next wave, early: shown in the break, with the seconds left (an early call pays them as coins). */
function callButton(world: World, g: GameV, small: boolean): void {
  if (g.inWave || g.wave >= WAVES.length) return;
  const secs = Math.ceil(g.breakTicks / 60);
  if (og.ui.button("Next", `${say(world, "hud.next")} ${secs}`, { variant: "secondary", size: small ? "medium" : "medium" }).clicked) g.callNext = true;
}

/** The settings dialog: volumes, language, motion; saved through og.platform when it closes. */
function settings(world: World, theme: Theme, g: GameV, p: PrefsV): void {
  let done = false;
  const before = JSON.stringify(prefsValue(p));
  const m = og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: theme.space.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.master = slider("Master", "settings.master", p.master);
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "fgMuted" } });
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

/** The end card: won or lost, stars by the light left, the score counting up, the best, the round's numbers. */
function endCard(world: World, theme: Theme, g: GameV, t: TuneV): "retry" | "menu" | null {
  let chose: "retry" | "menu" | null = null;
  const stars = !g.won ? 0 : g.lives >= t.startLives - 1 ? 3 : g.lives * 2 >= t.startLives ? 2 : 1;
  const since = world.tick - g.overTick;
  og.ui.modal("End", g.phase === Phase.Over, { title: say(world, g.won ? "over.won" : "over.lost"), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Stars", { width: grow, align: "center", gap: theme.space.sm }, () => {
      for (let k = 0; k < 3; k++) {
        const lit = k < stars && since >= 20 + k * 12;
        og.ui.transition(og.ui.idi("Star", k), true, { kind: "scale" }, () => {
          og.ui.icon("Mark", "star", { size: 40, tint: { token: lit ? "accent" : "fillStrong" } });
        });
        if (lit && since === 20 + k * 12) og.ui.burst({ count: 10 });
      }
    });
    og.ui.counter("Score", g.score, { from: 0, durationMs: 900, step: "displayMd", font: "display", align: "center", width: grow });
    og.ui.row("BestRow", { width: grow, align: "center", gap: theme.space.sm, cross: "center" }, () => {
      og.ui.label("Best", say(world, "over.best", { n: g.best }), { color: { token: "fgMuted" }, nowrap: true });
      if (g.newBest) og.ui.transition("NewBest", since > 50, { kind: "pop" }, () => og.ui.badge("Badge", say(world, "over.newBest"), "accent"));
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Waves", say(world, "over.waves"), `${g.won ? g.wave : Math.max(0, g.wave - 1)}/${WAVES.length}`, true);
      og.ui.keyValue("Kills", say(world, "over.kills"), `${g.kills}`, true);
      og.ui.keyValue("Lives", say(world, "over.lives"), `${g.lives}/${t.startLives}`, true);
    });
    og.ui.row("Actions", { width: grow, gap: theme.space.sm }, () => {
      if (og.ui.button("Retry", say(world, "over.retry"), { variant: "primary", width: grow }).clicked) chose = "retry";
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) chose = "menu";
    });
  });
  return chose;
}

/** The screens, the camera's framing, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning], writes: [Game, Prefs, og.Camera, og.Audio], after: ["Visitors"] }, (_q, time, world) => {
  const game = world.resource(Game);
  const prefs = world.resource(Prefs);
  if (!game.ok || !prefs.ok) return;
  const g = game.value;
  const t = lazyTuning(world);
  const p = prefs.value;
  platform(world, g, p);
  if (time.tick === 0) applyPrefs(world, p);
  frameCamera(world, g);
  // Paused: the garden holds (time scale 0); playing: it runs at 1 (og-feel §3: never left below 1).
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }

  const theme = THEME;
  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "iris", durationMs: 600 }).shown;
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
    });
    if (!g.settingsOpen && r.selected === "play") startRound(world, g, t);
    if (r.selected === "settings") g.settingsOpen = true;
  } else if (shown === "play") {
    hud(world, theme, g, t);
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
    const c = endCard(world, theme, g, t);
    if (c === "retry") startRound(world, g, t);
    if (c === "menu") g.phase = Phase.Title;
  }
  settings(world, theme, g, p);
  if (g.language !== p.language) g.language = p.language;
});
