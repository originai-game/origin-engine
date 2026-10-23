// The settings screen: Audio, Graphics, Controls and Language over the
// game's GameSettings. The game owns the values (a resource, a save); the
// screen answers the new ones every tick and calls the hooks: onChange on
// every change (apply it at once), onSave when it closes after changes.

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { frame, grow, px, token } from "./util.ts";

type Settings = {
  -readonly [K in keyof Required<Api.GameSettings>]: K extends "bindings" ? Api.KeyBinding[] : Required<Api.GameSettings>[K];
};

/** Every field of the settings, defaults filled in. */
export function settingsWithDefaults(s: Api.GameSettings = {}): Settings {
  return {
    masterVolume: s.masterVolume ?? 1,
    musicVolume: s.musicVolume ?? 0.8,
    sfxVolume: s.sfxVolume ?? 1,
    muted: s.muted ?? false,
    quality: s.quality ?? "medium",
    showFps: s.showFps ?? false,
    reducedMotion: s.reducedMotion ?? false,
    highContrast: s.highContrast ?? false,
    vibration: s.vibration ?? true,
    invertY: s.invertY ?? false,
    sensitivity: s.sensitivity ?? 1,
    language: s.language ?? "en",
    bindings: [...(s.bindings ?? [])],
  };
}

const QUALITIES: readonly Api.GraphicsQuality[] = ["low", "medium", "high"];

/** A labelled slider row with its value at the end. */
function sliderRow(id: string, label: string, value: number, desc: Api.SliderDesc, shown: string): number {
  const { t, phone } = frame();
  let out = value;
  kit.row(id, { width: grow(), gap: t.md, cross: "center" }, () => {
    kit.label("Label", label, { width: px(phone ? 96 : 140), nowrap: true });
    const r = kit.slider("Slider", value, desc) as Api.SliderResult;
    out = r.value;
    kit.label("Value", shown, { width: px(48), align: "end", font: "mono", color: token("inkMuted"), nowrap: true });
  });
  return out;
}

export function settingsScreen(id: string, desc: Api.SettingsDesc = {}): Api.SettingsResult {
  const { t, phone } = frame();
  const before = settingsWithDefaults(desc.settings);
  const s = settingsWithDefaults(desc.settings);
  const languages = desc.languages ?? [];
  // Every word is a field of the desc with its English default (og-i18n §2).
  const tabs: Array<[key: string, label: string]> = [
    ["audio", desc.audioLabel ?? "Audio"],
    ["graphics", desc.graphicsLabel ?? "Graphics"],
    ["controls", desc.controlsLabel ?? "Controls"],
    ...(languages.length > 0 ? [["language", desc.languageLabel ?? "Language"] as [string, string]] : []),
  ];
  let tab = Math.min(kit.memory(`${id}.tab`, 0), tabs.length - 1);
  let dirty = kit.memory(`${id}.dirty`, 0) !== 0;
  let closed = false;
  const width = phone ? grow() : px(Math.min(600, frame().viewport.width - 48));
  const m = kit.modal(id, desc.open ?? false, { title: desc.title ?? "Settings", width, ignoreBackdrop: true }, () => {
    // Tabs on a wide surface, a select on a phone (one selection grammar).
    // The groups' names are the desc's too: a screen reader hears them in the game's language.
    const labels = tabs.map(([, label]) => label);
    const section = desc.sectionLabel ?? "Section";
    const pick = phone
      ? (kit.select("Section", labels, tab, { width: grow(), label: section }) as Api.ChoiceResult)
      : (kit.tabs("Section", labels, tab, { fill: true, width: grow(), label: section }) as Api.ChoiceResult);
    tab = pick.selected;
    kit.column("Body", { width: grow(), gap: t.sm, height: px(phone ? 300 : 260) }, () => {
      const pct = (v: number): string => `${Math.round(v * 100)}%`;
      switch (tabs[tab]?.[0]) {
        case "audio": {
          s.masterVolume = sliderRow("Master", desc.masterLabel ?? "Master", s.masterVolume, { step: 0.05 }, pct(s.masterVolume));
          s.musicVolume = sliderRow("Music", desc.musicLabel ?? "Music", s.musicVolume, { step: 0.05 }, pct(s.musicVolume));
          s.sfxVolume = sliderRow("Effects", desc.effectsLabel ?? "Effects", s.sfxVolume, { step: 0.05 }, pct(s.sfxVolume));
          s.muted = (kit.toggle("Mute", s.muted, desc.muteLabel ?? "Mute everything") as Api.ToggleResult).on;
          break;
        }
        case "graphics": {
          const quality = desc.qualityLabel ?? "Quality";
          kit.label("QualityLabel", quality, { step: "meta", color: token("inkMuted") });
          const levels = [desc.lowLabel ?? "Low", desc.mediumLabel ?? "Medium", desc.highLabel ?? "High"];
          const q = kit.radio("Quality", levels, QUALITIES.indexOf(s.quality), { horizontal: !phone, label: quality });
          s.quality = QUALITIES[q.selected] ?? "medium";
          s.showFps = (kit.toggle("Fps", s.showFps, desc.fpsLabel ?? "Show frame rate") as Api.ToggleResult).on;
          s.reducedMotion = (kit.toggle("Motion", s.reducedMotion, desc.motionLabel ?? "Reduce motion") as Api.ToggleResult).on;
          s.highContrast = (kit.toggle("Contrast", s.highContrast, desc.contrastLabel ?? "High contrast") as Api.ToggleResult).on;
          break;
        }
        case "controls": {
          s.sensitivity = sliderRow("Sensitivity", desc.sensitivityLabel ?? "Sensitivity", s.sensitivity, { min: 0.1, max: 3, step: 0.1 }, s.sensitivity.toFixed(1));
          s.invertY = kit.checkbox("Invert", s.invertY, desc.invertLabel ?? "Invert vertical look").on;
          s.vibration = kit.checkbox("Vibration", s.vibration, desc.vibrationLabel ?? "Vibration").on;
          if (s.bindings.length > 0) {
            kit.divider("Rule", false);
            kit.scroll("Bindings", { width: grow(), height: grow() }, () => {
              s.bindings.forEach((b, i) => kit.keyValue(kit.idi("Binding", i), b.label ?? b.action ?? "", b.key ?? "", true));
            });
          }
          break;
        }
        case "language": {
          const at = Math.max(0, languages.findIndex((l) => l.code === s.language));
          const r = kit.radio("Languages", languages.map((l) => l.label ?? l.code ?? ""), at, { label: desc.languagesLabel ?? "Languages" });
          s.language = languages[r.selected]?.code ?? s.language;
          break;
        }
      }
    });
    kit.row("Footer", { width: grow(), align: "end" }, () => {
      if ((kit.button("Done", desc.doneLabel ?? "Done", { variant: "primary" }) as Api.Response).clicked) closed = true;
    });
  });
  if (m.dismissed) closed = true;
  const changed = JSON.stringify(s) !== JSON.stringify(before);
  if (changed) {
    dirty = true;
    desc.onChange?.(s);
  }
  if (closed && dirty) {
    desc.onSave?.(s);
    dirty = false;
  }
  kit.remember(`${id}.tab`, tab);
  kit.remember(`${id}.dirty`, dirty ? 1 : 0);
  return { settings: s, changed, closed };
}
