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
    kit.label("Value", shown, { width: px(48), align: "end", font: "mono", color: token("fgMuted"), nowrap: true });
  });
  return out;
}

export function settingsScreen(id: string, desc: Api.SettingsDesc = {}): Api.SettingsResult {
  const { t, phone } = frame();
  const before = settingsWithDefaults(desc.settings);
  const s = settingsWithDefaults(desc.settings);
  const languages = desc.languages ?? [];
  const tabs = ["Audio", "Graphics", "Controls", ...(languages.length > 0 ? ["Language"] : [])];
  let tab = Math.min(kit.memory(`${id}.tab`, 0), tabs.length - 1);
  let dirty = kit.memory(`${id}.dirty`, 0) !== 0;
  let closed = false;
  const width = phone ? grow() : px(Math.min(600, frame().viewport.width - 48));
  const m = kit.modal(id, desc.open ?? false, { title: "Settings", width, ignoreBackdrop: true }, () => {
    // Tabs on a wide surface, a select on a phone (one selection grammar).
    const pick = phone
      ? (kit.select("Section", tabs, tab, { width: grow() }) as Api.ChoiceResult)
      : (kit.tabs("Section", tabs, tab, { fill: true, width: grow() }) as Api.ChoiceResult);
    tab = pick.selected;
    kit.column("Body", { width: grow(), gap: t.sm, height: px(phone ? 300 : 260) }, () => {
      const pct = (v: number): string => `${Math.round(v * 100)}%`;
      switch (tabs[tab]) {
        case "Audio": {
          s.masterVolume = sliderRow("Master", "Master", s.masterVolume, { step: 0.05 }, pct(s.masterVolume));
          s.musicVolume = sliderRow("Music", "Music", s.musicVolume, { step: 0.05 }, pct(s.musicVolume));
          s.sfxVolume = sliderRow("Effects", "Effects", s.sfxVolume, { step: 0.05 }, pct(s.sfxVolume));
          s.muted = (kit.toggle("Mute", s.muted, "Mute everything") as Api.ToggleResult).on;
          break;
        }
        case "Graphics": {
          kit.label("QualityLabel", "Quality", { step: "meta", color: token("fgMuted") });
          const q = kit.radio("Quality", ["Low", "Medium", "High"], QUALITIES.indexOf(s.quality), { horizontal: !phone });
          s.quality = QUALITIES[q.selected] ?? "medium";
          s.showFps = (kit.toggle("Fps", s.showFps, "Show frame rate") as Api.ToggleResult).on;
          s.reducedMotion = (kit.toggle("Motion", s.reducedMotion, "Reduce motion") as Api.ToggleResult).on;
          s.highContrast = (kit.toggle("Contrast", s.highContrast, "High contrast") as Api.ToggleResult).on;
          break;
        }
        case "Controls": {
          s.sensitivity = sliderRow("Sensitivity", "Sensitivity", s.sensitivity, { min: 0.1, max: 3, step: 0.1 }, s.sensitivity.toFixed(1));
          s.invertY = kit.checkbox("Invert", s.invertY, "Invert vertical look").on;
          s.vibration = kit.checkbox("Vibration", s.vibration, "Vibration").on;
          if (s.bindings.length > 0) {
            kit.divider("Rule", false);
            kit.scroll("Bindings", { width: grow(), height: grow() }, () => {
              s.bindings.forEach((b, i) => kit.keyValue(kit.idi("Binding", i), b.label ?? b.action ?? "", b.key ?? "", true));
            });
          }
          break;
        }
        case "Language": {
          const at = Math.max(0, languages.findIndex((l) => l.code === s.language));
          const r = kit.radio("Languages", languages.map((l) => l.label ?? l.code ?? ""), at, {});
          s.language = languages[r.selected]?.code ?? s.language;
          break;
        }
      }
    });
    kit.row("Footer", { width: grow(), align: "end" }, () => {
      if ((kit.button("Done", "Done", { variant: "primary" }) as Api.Response).clicked) closed = true;
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
