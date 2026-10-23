// The player's settings: what the settings screen shows and answers
// (GameSettings), what Prefs keeps, and how they apply (the buses' volumes,
// the UI's reduced motion and contrast and its sounds, the language).

import { og } from "@origingame/engine";
import type { GameSettings, Value, World } from "@origingame/engine";
import { Prefs } from "./components.ts";
import { must } from "./level.ts";

type PrefsV = Value<typeof Prefs>;

/** The languages offered: their codes and the key of each one's name in itself. */
export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "es", key: "lang.es" },
] as const;

/** The UI's sounds and reduced motion; configure keeps the style the look applied (og-ui §6). */
export function configureUi(world: World, p: PrefsV): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, {
    reducedMotion: p.reducedMotion,
    contrast: p.highContrast ? "high" : "standard",
    sounds: {
      hover: sound("audio/ui-hover.ogt"),
      press: sound("audio/ui-click.ogt"),
      success: sound("audio/ui-select.ogt"),
      open: sound("audio/ui-select.ogt"),
      close: sound("audio/ui-back.ogt"),
      error: sound("audio/ui-error.ogt"),
    },
  }));
}

export function applyPrefs(world: World, p: PrefsV): void {
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

export function settingsOf(p: PrefsV): GameSettings {
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

/** Takes what the settings screen (or the saved og.settings) answered, and applies it. */
export function takeSettings(world: World, p: PrefsV, s: Partial<GameSettings>): void {
  if (typeof s.masterVolume === "number") p.master = s.masterVolume;
  if (typeof s.musicVolume === "number") p.music = s.musicVolume;
  if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
  if (typeof s.muted === "boolean") p.muted = s.muted;
  if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
  if (typeof s.highContrast === "boolean") p.highContrast = s.highContrast;
  // Asked for even when it matches: the host's own locale may be another (og-platform-sdk §3).
  if (typeof s.language === "string") setLanguage(world, p, s.language);
  applyPrefs(world, p);
}
