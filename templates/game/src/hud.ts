// The screens: the title's menu panel (the scene's opening, as it always
// was), the run's HUD, the pause, the settings and the summary, and the
// touch controls. Every button that changes the world goes through press()
// (src/seats.ts), so a room sees it on the same tick on every peer: the
// title's Start and the summary's Play again press `start`, the HUD's Pause
// and the pause's Resume press `pause`. The settings are this device's own
// (prefs, below): the volumes, the language and reduced motion, loaded
// from og.platform's settings on the first tick and saved by Done, never in
// the world.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Ending, Health, Mission, Phase, Run } from "./components.ts";
import { CELLS, heroOf, missionOf, must, SUMMARY_READY } from "./play.ts";
import { bit, LOCAL, press, SEATS, setLookLive, useTouchControls, usesTouch } from "./seats.ts";
import type { TouchInput } from "./seats.ts";

/** The languages the settings offer, each a table under assets/strings/. */
export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** This device's settings, never in the world. */
export const prefs = { master: 1, effects: 1, language: 0, reducedMotion: false, loadRequest: 0, saves: 0, settingsOpen: false };

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;
const say = (world: World, key: string, args?: Record<string, string | number>) => og.i18n.t(world, key, args);

function applyPrefs(world: World): void {
  const audio = world.resource(og.Audio);
  if (audio.ok) {
    audio.value.master = prefs.master;
    audio.value.sfx = prefs.effects;
    audio.value.ui = prefs.effects;
  }
}

export function setLanguage(world: World, index: number): void {
  prefs.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[prefs.language]!.code);
}

function prefsValue() {
  return { masterVolume: prefs.master, sfxVolume: prefs.effects, language: LANGUAGES[prefs.language]!.code, reducedMotion: prefs.reducedMotion };
}

/** og.platform's settings: asked for on tick 0, applied when they answer. */
function loadPrefs(world: World): void {
  if (world.tick === 0) {
    prefs.loadRequest = og.platform.loadSettings(world);
    return;
  }
  if (prefs.loadRequest === 0) return;
  const r = og.platform.result(world, prefs.loadRequest);
  if (r === null) return;
  prefs.loadRequest = 0;
  const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
  if (!s || typeof s !== "object") return;
  if (typeof s.masterVolume === "number") prefs.master = s.masterVolume;
  if (typeof s.sfxVolume === "number") prefs.effects = s.sfxVolume;
  if (typeof s.reducedMotion === "boolean") prefs.reducedMotion = s.reducedMotion;
  const at = LANGUAGES.findIndex((l) => l.code === s.language);
  if (at >= 0) setLanguage(world, at);
  applyPrefs(world);
}

/** What the touch controls show next tick (Seats draws them): only in a run, not paused. */
const touchHud = { shown: false, jump: "Jump", run: "Run" };

/**
 * The touch controls (where the player chose touch or the pointer is a
 * finger), only during a run: the move stick bottom left, the look stick
 * and Jump and Run bottom right; two seats on one device each a column in
 * its own bottom corner.
 */
function touch(seat: number): TouchInput {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  if (!touchHud.shown) return out;
  const size = LOCAL ? 112 : 132;
  const move = () => {
    const s = og.ui.joystick("Move", { size });
    out.x = s.x;
    out.y = -s.y;
  };
  const buttons = () => {
    if (og.ui.actionButton("run", touchHud.run, { size: LOCAL ? 52 : 60 }).pressed) out.buttons |= bit("run");
    if (og.ui.actionButton("jump", touchHud.jump, { size: LOCAL ? 64 : 76, primary: true }).pressed) out.buttons |= bit("jump");
  };
  const look = () => {
    const s = og.ui.joystick("Look", { size: LOCAL ? 96 : 112 });
    out.lx = s.x * 2.5;
    out.ly = -s.y * 2.5;
  };
  if (LOCAL) {
    og.ui.anchor(`TouchMove${seat}`, { anchor: seat === 0 ? "bottomLeft" : "bottomRight", direction: "column", gap: 12, cross: "center" }, () => {
      og.ui.row("Actions", { gap: 8, cross: "end" }, () => {
        buttons();
        look();
      });
      move();
    });
    return out;
  }
  // A phone in portrait is narrow: the buttons stand over the look stick, the move stick has the other corner.
  og.ui.anchor(`TouchMove${seat}`, { anchor: "bottomLeft", direction: "column", gap: 12, cross: "center" }, move);
  og.ui.anchor(`TouchActions${seat}`, { anchor: "bottomRight", direction: "column", gap: 12, cross: "end" }, () => {
    og.ui.row("Buttons", { gap: 12, cross: "end" }, buttons);
    look();
  });
  return out;
}
useTouchControls(touch);

/**
 * The menu panel: the title, the prompt and the Start button. A click on
 * Start presses `start` (as S, Enter or a pad's A do); src/play.ts's Flow
 * then hands the trickster to the player.
 */
function title(world: World): void {
  og.ui.column("Hud", { padding: { left: 24, top: 24, right: 24, bottom: 24 } }, () => og.ui.panel("Menu", { padding: { left: 20, top: 16, right: 20, bottom: 16 } }, () => {
    og.ui.label("Title", say(world, "menu.title"), { step: "displaySm" });
    og.ui.label("Status", say(world, "menu.prompt"), { color: { token: "inkMuted" } });
    const start = og.ui.button("Start", say(world, "menu.start"), { variant: "primary", size: "large", disabled: false });
    if (start.clicked) press("start");
  }));
}

function clock(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** The run's HUD: the cells, each hero's health and the clock top left, Pause top right, the goal below. */
function hud(world: World, m: ReturnType<typeof missionOf>): void {
  og.ui.anchor("Run", { anchor: "topLeft", direction: "column", gap: 8, padding: { left: 16, top: 16, right: 16, bottom: 16 } }, () => {
    og.ui.panel("Status", { padding: { left: 16, top: 12, right: 16, bottom: 12 }, gap: 6 }, () => {
      og.ui.row("Cells", { gap: 10, cross: "center" }, () => {
        og.ui.label("Label", say(world, "hud.cells", { n: m.cells, total: CELLS }), { step: "lg", nowrap: true });
        og.ui.label("Clock", clock(m.time), { font: "mono", color: { token: "inkMuted" }, nowrap: true });
      });
      og.ui.progressBar("CellBar", { value: m.cells / CELLS, width: px(180), tone: m.cells >= CELLS ? "success" : "primary" });
      for (let seat = 0; seat < SEATS; seat++) {
        const hp = must(world.get(heroOf(world, seat), Health));
        og.ui.row(`Seat${seat}`, { gap: 8, cross: "center" }, () => {
          og.ui.label("Who", say(world, "hud.seat", { n: seat + 1 }), { step: "caption", width: px(64), nowrap: true });
          og.ui.progressBar("Health", { value: hp.hp / hp.max, width: px(108), tone: hp.hp / hp.max < 0.4 ? "danger" : "success" });
        });
      }
    });
  });
  og.ui.anchor("Bar", { anchor: "topRight", padding: { left: 16, top: 16, right: 16, bottom: 16 } }, () => {
    if (og.ui.button("Pause", say(world, "hud.pause"), { variant: "secondary" }).clicked) press("pause");
  });
  const goal = m.cells >= CELLS ? "hud.ready" : m.time < 6 ? "hud.goal" : "";
  if (goal !== "") {
    og.ui.anchor("Goal", { anchor: "top", padding: { left: 16, top: usesTouch() ? 132 : 20, right: 16, bottom: 16 } }, () => {
      og.ui.panel("Card", { padding: { left: 14, top: 8, right: 14, bottom: 8 } }, () => og.ui.label("Text", say(world, goal), { color: { token: m.cells >= CELLS ? "highlight" : "ink" } }));
    });
  }
}

function settings(world: World): void {
  let done = false;
  const before = JSON.stringify(prefsValue());
  og.ui.modal("Settings", prefs.settingsOpen, { title: say(world, "settings.title"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: 12, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    prefs.master = slider("Master", "settings.master", prefs.master);
    prefs.effects = slider("Effects", "settings.effects", prefs.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), prefs.language, { horizontal: true });
    if (r.selected !== prefs.language) setLanguage(world, r.selected);
    prefs.reducedMotion = og.ui.toggle("Motion", prefs.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue()) !== before) applyPrefs(world);
  if (prefs.settingsOpen && done) {
    prefs.settingsOpen = false;
    must(og.platform.saveSettings(world, prefsValue()));
    prefs.saves += 1;
  }
}

function pause(world: World, m: ReturnType<typeof missionOf>): void {
  og.ui.modal("Pause", m.phase === Phase.Play && m.paused && !prefs.settingsOpen, { title: say(world, "pause.title"), locked: true, width: px(Math.min(320, og.ui.frameViewport().width - 32)) }, () => {
    if (og.ui.button("Resume", say(world, "pause.resume"), { variant: "primary", width: grow, autofocus: true }).clicked) press("pause");
    if (og.ui.button("Settings", say(world, "pause.settings"), { variant: "secondary", width: grow }).clicked) prefs.settingsOpen = true;
  });
}

/** The summary: how the run ended, the cells, the time, the best and the grabs; Play again. */
function summary(world: World, m: ReturnType<typeof missionOf>): void {
  const won = m.ending === Ending.Escaped;
  og.ui.modal("Summary", m.phase === Phase.Over, { title: say(world, won ? "over.escaped" : "over.caught"), locked: true, width: px(Math.min(360, og.ui.frameViewport().width - 32)) }, () => {
    const row = (id: string, key: string, value: string) =>
      og.ui.row(id, { width: grow, gap: 12 }, () => {
        og.ui.label("Key", say(world, key), { width: grow, color: { token: "inkMuted" } });
        og.ui.label("Value", value, { font: "mono", nowrap: true });
      });
    row("Cells", "over.cells", `${m.cells}/${CELLS}`);
    row("Time", "over.time", clock(m.time));
    if (m.best > 0) row("Best", "over.best", clock(m.best));
    row("Grabs", "over.grabs", `${m.catches}`);
    const again = og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow, disabled: m.overTicks < SUMMARY_READY });
    if (again.clicked) press("start");
  });
}

/** Draws the screen the run is on, and turns the mouse-look on only in a run. */
export const Hud = defineSystem("Hud", { reads: [Run, Mission, Health], writes: [og.Audio] }, (_q, _time, world) => {
  loadPrefs(world);
  if (world.tick === 0) applyPrefs(world);
  const m = missionOf(world);
  touchHud.jump = say(world, "hud.jump");
  touchHud.run = say(world, "hud.run");
  touchHud.shown = m.phase === Phase.Play && !m.paused;
  setLookLive(m.phase === Phase.Play && !m.paused);
  if (m.phase === Phase.Title) {
    title(world);
    return;
  }
  hud(world, m);
  pause(world, m);
  settings(world);
  summary(world, m);
  // In a run the keys and the pad play the hero (A is Jump, not a press of the focused Pause); the pause and the summary are navigated.
  og.ui.setNavigation(m.phase === Phase.Over || m.paused);
});
