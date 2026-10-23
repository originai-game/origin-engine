// Spark Orb's screens on og.ui and the skin in assets/skins/sparkorb (the
// engine's neon pack in the orb's gold on night violet): the title, the
// HUD (the sparks and their rate top left, stardust and pause top right,
// the four generators along the bottom, the supernova's progress), the
// pause menu, the settings dialog, the supernova's dialog and its run
// card, and the welcome back. Screens change through an iris (a fade with
// reduced motion). Every string is a key of assets/strings/*.ogt and
// every number goes through src/numbers.ts's `big`.
//
// The Ui system also keeps the save in og.platform: read on launch (and on
// `relaunch`), the time away (the answer's host time less the save's
// `at`, og.platform.now when it was written) credited, written every Tuning.saveEvery
// ticks, after a supernova, on pause and on Quit, at the revision read; a write
// refused as stale is read again and merged (the save with more lifetime
// sparks wins) and written at the new revision.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, GENERATORS, Phase, Prefs, Sound, Tuning } from "./components.ts";
import { big, duration, novaAt, novaGain, offlineGain, owned, price, totalRate } from "./numbers.ts";
import { counts, lazyTuning, must } from "./systems.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** The generators' icons in the skin's atlas. */
const ICONS = ["fire", "bolt", "gem", "star"] as const;

/** The tokens' spacing scale in CSS px (og.ui.frameTheme answers it too, but builds the whole theme each call). */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24 } as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** The save's value: everything a relaunch restores, and when it was written (the host's clock, UTC ms). */
function saveValue(world: World, g: GameV) {
  return { v: 1, sparks: g.sparks, runEarned: g.runEarned, lifetime: g.lifetime, owned: counts(g), dust: g.dust, novas: g.novas, runTicks: g.runTicks, taps: g.taps, at: og.platform.now(world) };
}

type Save = ReturnType<typeof saveValue>;

function restore(g: GameV, s: Save): void {
  g.sparks = s.sparks ?? 0;
  g.runEarned = s.runEarned ?? 0;
  g.lifetime = s.lifetime ?? 0;
  g.owned0 = s.owned?.[0] ?? 0;
  g.owned1 = s.owned?.[1] ?? 0;
  g.owned2 = s.owned?.[2] ?? 0;
  g.owned3 = s.owned?.[3] ?? 0;
  g.dust = s.dust ?? 0;
  g.novas = s.novas ?? 0;
  g.runTicks = s.runTicks ?? 0;
  g.taps = s.taps ?? 0;
}

function writeSave(world: World, g: GameV): void {
  const r = og.platform.storeSet(world, { key: "orb", value: saveValue(world, g), revision: g.saveRevision });
  if (r.ok) g.saveWrite = r.value;
  g.sinceSave = 0;
  g.saveNow = false;
}

/** A launch: the settings and the save asked for (tick 0, or `relaunch`). */
export function launch(world: World, g: GameV, p: PrefsV): void {
  p.loadRequest = og.platform.loadSettings(world);
  const r = og.platform.storeGet(world, "orb");
  g.saveRequest = r.ok ? r.value : 0;
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

/** og.platform's answers: the settings, the save (and the time away), the save's writes. */
function platform(world: World, g: GameV, p: PrefsV): void {
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
  if (g.saveRequest !== 0) {
    const r = og.platform.result(world, g.saveRequest);
    if (r !== null) {
      g.saveRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: Save | null; revision?: number } | null;
      if (typeof d?.revision === "number") g.saveRevision = d.revision;
      if (g.saveMerge) {
        // Another device wrote the save: the one with more lifetime sparks wins, then it is written here.
        g.saveMerge = false;
        if (d?.value && (d.value.lifetime ?? 0) > g.lifetime) restore(g, d.value);
        writeSave(world, g);
      } else if (!g.loaded) {
        g.loaded = true;
        if (d?.value) {
          restore(g, d.value);
          // The time away: the answer's host time less the save's (relaunch:<s> has set it instead).
          if (g.away === 0 && d.value.at > 0 && r.time > 0) g.away = Math.max(0, (r.time - d.value.at) / 1000);
          const t = lazyTuning(world);
          const gain = offlineGain(t, totalRate(t, counts(g), g.dust), g.away);
          if (gain > 0 && g.away >= 60) {
            g.offline = gain;
            g.sparks += gain;
            g.runEarned += gain;
            g.lifetime += gain;
            g.welcome = true;
          }
        }
      }
    }
  }
  if (g.saveWrite !== 0) {
    const r = og.platform.result(world, g.saveWrite);
    if (r !== null) {
      g.saveWrite = 0;
      if (r.ok) {
        g.saveRevision = (r.data as { revision: number }).revision;
        g.saves += 1;
      } else if (r.error?.code === "conflict") {
        g.saveConflicts += 1;
        g.saveMerge = true;
        const again = og.platform.storeGet(world, "orb");
        g.saveRequest = again.ok ? again.value : 0;
      }
    }
  }
  // Saving: every Tuning.saveEvery ticks of play, and when asked (a supernova, the pause, Quit, which has gone to the title by then).
  if (g.loaded && (g.phase !== Phase.Title || g.saveNow) && g.saveWrite === 0 && g.saveRequest === 0) {
    g.sinceSave += 1;
    if (g.saveNow || g.sinceSave >= lazyTuning(world).saveEvery) writeSave(world, g);
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "capsule", background: { token: "panel" } }, build);
}

function hud(world: World, g: GameV): void {
  const t = lazyTuning(world);
  const vp = og.ui.frameViewport();
  const phone = vp.width < vp.height;
  og.ui.setNavigation(false);
  og.ui.anchor("Wallet", { anchor: "topLeft", direction: "column", gap: S.xs }, () => {
    capsule("Sparks", () => {
      og.ui.icon("Icon", "star", { size: 24, tint: { token: "highlight" } });
      // The sparks change every tick, so they are a label in the locale's words, not a counter
      // (a counter re-counts from where it is on every change); counters show what jumps.
      og.ui.label("Amount", big(world, g.sparks), { step: "xl", font: "display", nowrap: true });
    });
    og.ui.row("RateRow", { padding: { left: S.sm, top: 0, right: 0, bottom: 0 } }, () => {
      og.ui.label("Rate", say(world, "hud.rate", { n: big(world, g.rate) }), { step: "body", color: { token: "inkMuted" }, nowrap: true });
    });
  });
  let paused = false;
  og.ui.anchor("Corner", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    if (g.dust > 0 || g.novas > 0) {
      capsule("Dust", () => {
        og.ui.icon("Icon", "gem", { size: 20, tint: { token: "info" } });
        og.ui.counter("Amount", g.dust, { font: "mono" });
      });
    }
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) paused = true;
  });
  if (paused) {
    g.paused = true;
    g.saveNow = true;
  }
  // The generators along the bottom: a button each (its icon), its name, how many, its price.
  let buy = -1;
  og.ui.anchor("Shop", { anchor: "bottom", gap: phone ? S.sm : S.xl, cross: "end" }, () => {
    GENERATORS.forEach((name, i) => {
      const n = owned(g, i);
      const cost = price(t, i, n);
      const can = g.sparks >= cost;
      og.ui.column(og.ui.idi("Gen", i), { cross: "center", gap: S.xxs, width: px(phone ? 82 : 96) }, () => {
        const r = og.ui.actionButton("Buy", say(world, "hud.buy", { name: say(world, `gen.${name}`) }), { icon: ICONS[i]!, size: phone ? 56 : 64, primary: false });
        if (r.clicked) buy = i;
        // A refused purchase shakes its card with the error sound; a milestone bursts from it (og-ui §6).
        if (g.refusedSlot === i && g.refusedTick === world.tick - 1) og.ui.shake({ sound: true });
        if (g.milestoneSlot === i && g.milestoneTick === world.tick - 1) og.ui.burst({ count: 10 });
        og.ui.row("Head", { gap: S.xxs, cross: "center" }, () => {
          og.ui.label("Name", say(world, `gen.${name}`), { step: "body", color: { token: g.selected === i ? "highlight" : "ink" }, nowrap: true });
          og.ui.counter("Owned", n, { font: "mono", prefix: "×", color: { token: "inkMuted" } });
        });
        og.ui.label("Cost", big(world, cost), { font: "mono", color: { token: can ? "highlight" : "inkMuted" }, nowrap: true });
      });
    });
  });
  if (buy >= 0) g.pendingBuy = buy;
  // The supernova: its progress until it gives stardust, then its button.
  const gain = novaGain(t, g.runEarned);
  og.ui.anchor("Nova", { anchor: phone ? "topRight" : "bottomRight", offsetY: phone ? 64 : 0, direction: "column", cross: "center", gap: S.xxs }, () => {
    if (gain >= 1) {
      if (og.ui.actionButton("Supernova", say(world, "hud.nova"), { icon: "bolt", size: 64, primary: true }).clicked) g.confirmNova = true;
      og.ui.label("Gain", say(world, "hud.novaGain", { n: gain }), { color: { token: "highlight" }, nowrap: true });
    } else if (g.runEarned > 0) {
      og.ui.label("At", say(world, "hud.novaAt", { n: big(world, novaAt(t)) }), { step: "caption", color: { token: "inkMuted" }, nowrap: true });
      og.ui.meter("Progress", Math.min(g.runEarned, novaAt(t)), novaAt(t), { width: px(phone ? 110 : 140) });
    }
  });
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

/** The dialogs over play: the supernova's question and the welcome back. */
function dialogs(world: World, g: GameV): void {
  const t = lazyTuning(world);
  const width = px(Math.min(420, og.ui.frameViewport().width - 32));
  const gain = novaGain(t, g.runEarned);
  let collapse = false;
  let notYet = false;
  const m = og.ui.modal("Confirm", g.confirmNova, { title: say(world, "confirm.title"), width }, () => {
    og.ui.label("Body", say(world, "confirm.body", { n: gain, total: g.dust + gain, bonus: Math.round((g.dust + gain) * t.dustBonus * 100) }), { width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm, align: "end" }, () => {
      if (og.ui.button("Later", say(world, "confirm.later")).clicked) notYet = true;
      if (og.ui.button("Collapse", say(world, "confirm.collapse"), { variant: "primary" }).clicked) collapse = true;
    });
  });
  if (g.confirmNova && (notYet || m.dismissed)) g.confirmNova = false;
  if (g.confirmNova && collapse) {
    g.confirmNova = false;
    g.pendingNova = true;
  }
  let collect = false;
  const w = og.ui.modal("Welcome", g.welcome, { title: say(world, "welcome.title"), width }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => og.ui.icon("Icon", "clock", { size: 40, tint: { token: "highlight" } }));
    og.ui.label("Body", say(world, "welcome.body", { time: duration(world, g.away), n: big(world, g.offline) }), { width: grow, align: "center" });
    og.ui.row("Actions", { width: grow, align: "center" }, () => {
      if (og.ui.button("Collect", say(world, "welcome.collect"), { variant: "primary" }).clicked) collect = true;
    });
  });
  if (g.welcome && (collect || w.dismissed)) {
    g.welcome = false;
    g.sfx |= Sound.Welcome;
  }
}

/** The run's card after a supernova. */
function novaCard(world: World, g: GameV): boolean {
  const t = lazyTuning(world);
  let again = false;
  og.ui.modal("Card", g.phase === Phase.Nova, { title: say(world, "nova.title"), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", "gem", { size: 48, tint: { token: "info" } }));
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Made", say(world, "nova.made"), big(world, g.lastRun), true);
      og.ui.keyValue("Time", say(world, "nova.time"), duration(world, g.lastRunTicks / 60), true);
      og.ui.keyValue("Gain", say(world, "nova.gain"), `+${g.lastGain}`, true);
      og.ui.keyValue("Dust", say(world, "nova.dust"), `${g.dust}`, true);
      og.ui.keyValue("Bonus", say(world, "nova.bonus"), say(world, "nova.percent", { n: Math.round(g.dust * t.dustBonus * 100) }), true);
    });
    og.ui.row("Actions", { width: grow }, () => {
      if (og.ui.button("Again", say(world, "nova.again"), { variant: "primary", width: grow }).clicked) again = true;
    });
  });
  return again;
}

/** The screens, the camera, the buses, the platform's answers and the save. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Input, og.Gamepad], writes: [Game, Prefs, og.Camera, og.Audio] }, (_q, time, world) => {
  const game = world.resource(Game);
  const prefs = world.resource(Prefs);
  if (!game.ok || !prefs.ok) return;
  const g = game.value;
  const p = prefs.value;
  if (time.tick === 0) {
    launch(world, g, p);
    applyPrefs(world, p);
  }
  platform(world, g, p);
  // The camera: 10 m tall on a wide screen; 7 m across a portrait one, the orb above the shop.
  const vp = og.ui.frameViewport();
  if (vp.width > 0 && (vp.width !== g.viewW || vp.height !== g.viewH)) {
    g.viewW = vp.width;
    g.viewH = vp.height;
    const portrait = vp.width < vp.height;
    const cam = world.resource(og.Camera);
    if (cam.ok) cam.value.height = portrait ? (7.4 * vp.height) / vp.width : 10;
    g.orbX = 0;
    g.orbY = portrait ? 1.4 : 0.35;
  }
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen || g.confirmNova || g.welcome);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  const input = world.resource(og.Input);
  const pad = world.resource(og.Gamepad);
  if (g.phase === Phase.Play && !g.held && input.ok && pad.ok) {
    if (input.value.pressed("Escape") || input.value.pressed("KeyP") || pad.value.pressed("start")) {
      g.paused = true;
      g.saveNow = true;
    }
    if ((input.value.pressed("KeyN") || pad.value.pressed("y")) && novaGain(lazyTuning(world), g.runEarned) >= 1) g.confirmNova = true;
  }
  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "nova";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "iris", durationMs: 600 }).shown;
  if (shown !== "title") g.leftTitle = true;
  if (shown === "title") {
    og.ui.setNavigation(true);
    const progress = g.lifetime > 0 || g.dust > 0;
    const r = og.ui.mainMenu("Title", {
      title: say(world, "title.name"),
      subtitle: say(world, "title.tagline"),
      items: [
        { id: "play", label: say(world, progress ? "title.continue" : "title.play"), primary: true },
        { id: "settings", label: say(world, "title.settings") },
      ],
      footer: say(world, "title.footer"),
      // The title the game opens on is built settled: the web draws it behind the play gate, where
      // the skin's entrance would stand part-way until the tap (og-platform-sdk §3). Later titles enter.
      instant: !g.leftTitle,
    });
    if (!g.settingsOpen && r.selected === "play") {
      g.phase = Phase.Play;
      g.playTick = world.tick;
    }
    if (r.selected === "settings") g.settingsOpen = true;
  } else if (shown === "play") {
    hud(world, g);
    dialogs(world, g);
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
      g.saveNow = true;
    }
  } else {
    og.ui.setNavigation(true);
    if (novaCard(world, g)) {
      g.phase = Phase.Play;
      g.playTick = world.tick;
    }
  }
  settings(world, g, p);
  if (g.language !== p.language) g.language = p.language;
});
