// The settings screen: Audio, Graphics, Controls and Language over the
// game's GameSettings. The game owns the values (a resource, a save); the
// screen answers the new ones every tick and calls the hooks: onChange on
// every change (apply it at once), onSave when it closes after changes.
// A new graphics preset is applied to SettingsDesc.world by the screen
// itself (og.render.applyQuality) before onChange, and a new text size
// (og.ui.configure's textScale).
//
// With GameSettings.display (og.render.display(world).settings) the Graphics
// tab is the PC graphics page: the preset, render scale, frame-rate limit,
// V-Sync (fixed on: the browser's), adaptive quality, battery saver,
// fullscreen and the GPU's power preference (applies after a restart, as
// og.render.display's restartNeeded says), each change applied with
// og.render.setDisplay, which never writes the world. The Controls tab's
// raw mouse input goes to og.input.setMouse, and with SettingsDesc.lookAction
// its sensitivity and inversion to og.input.setSensitivity for the mouse.

import type * as Api from "@origingame/engine";
import { render } from "../runtime.ts";
import { input } from "../input.ts";
import { kit } from "./bind.ts";
import { fit, frame, grow, px, token } from "./util.ts";

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
    textScale: Math.min(2, Math.max(1, s.textScale ?? 1)),
    vibration: s.vibration ?? true,
    invertY: s.invertY ?? false,
    sensitivity: s.sensitivity ?? 1,
    language: s.language ?? "en",
    bindings: [...(s.bindings ?? [])],
    rawMouse: s.rawMouse ?? true,
    display: s.display ? { ...s.display } : null,
  };
}

const QUALITIES: readonly Api.GraphicsQuality[] = ["low", "medium", "high"];
const FRAME_CAPS: readonly Api.FrameCap[] = ["auto", "display", "fps30", "fps60", "fps120", "fps144"];
const POWER: readonly Api.PowerPreference[] = ["highPerformance", "lowPower"];

type World = Parameters<typeof render.setDisplay>[0];

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
    // The body sizes to its tab's rows, at least the old fixed height where
    // the screen has room (the dialog keeps its size between short tabs), and
    // scrolls once the dialog reaches the safe area: the modal's backdrop
    // shrinks it, a scroll's height going down to nothing. Its right padding
    // keeps the rows' values clear of the scroll bar (xs wide, xxs in).
    const floor = frame().viewport.height >= 600 ? (phone ? 300 : 260) : 0;
    const bar = t.xs + 2 * t.xxs;
    kit.scroll("Body", { width: grow(), height: fit(floor), gap: t.sm, padding: { left: 0, top: 0, bottom: 0, right: bar } }, () => {
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
          if (s.display) {
            graphicsPage(s, s.display, desc, phone);
            break;
          }
          const quality = desc.qualityLabel ?? "Quality";
          kit.label("QualityLabel", quality, { step: "meta", color: token("inkMuted") });
          const levels = [desc.lowLabel ?? "Low", desc.mediumLabel ?? "Medium", desc.highLabel ?? "High"];
          const q = kit.radio("Quality", levels, QUALITIES.indexOf(s.quality), { horizontal: !phone, label: quality });
          s.quality = QUALITIES[q.selected] ?? "medium";
          s.showFps = (kit.toggle("Fps", s.showFps, desc.fpsLabel ?? "Show frame rate") as Api.ToggleResult).on;
          s.reducedMotion = (kit.toggle("Motion", s.reducedMotion, desc.motionLabel ?? "Reduce motion") as Api.ToggleResult).on;
          s.highContrast = (kit.toggle("Contrast", s.highContrast, desc.contrastLabel ?? "High contrast") as Api.ToggleResult).on;
          s.textScale = sliderRow("TextSize", desc.textSizeLabel ?? "Text size", s.textScale, { min: 1, max: 2, step: 0.25 }, pct(s.textScale));
          break;
        }
        case "controls": {
          s.sensitivity = sliderRow("Sensitivity", desc.sensitivityLabel ?? "Sensitivity", s.sensitivity, { min: 0.1, max: 3, step: 0.1 }, s.sensitivity.toFixed(1));
          s.invertY = kit.checkbox("Invert", s.invertY, desc.invertLabel ?? "Invert vertical look").on;
          s.rawMouse = (kit.toggle("Raw", s.rawMouse, desc.rawMouseLabel ?? "Raw mouse input") as Api.ToggleResult).on;
          const mouse = desc.world ? (input.mouse(desc.world as unknown as World) as Api.MouseInfo) : null;
          if (mouse?.rawUnsupported) kit.label("RawNote", desc.rawUnavailableLabel ?? "Not available in this browser", { step: "meta", color: token("inkMuted") });
          s.vibration = kit.checkbox("Vibration", s.vibration, desc.vibrationLabel ?? "Vibration").on;
          if (s.bindings.length > 0) {
            kit.divider("Rule", false);
            kit.column("Bindings", { width: grow() }, () => {
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
    const world = desc.world as unknown as World | undefined;
    // The graphics page's settings: presentation only (og.render.setDisplay), drawn from the next frame.
    if (s.display && world && JSON.stringify(s.display) !== JSON.stringify(before.display)) {
      const r = render.setDisplay(world, s.display) as Api.Result<void>;
      if (!r.ok) throw new Error(`og.ui.settingsScreen: ${r.reason}`);
    }
    if (world && s.rawMouse !== before.rawMouse) input.setMouse(world, { raw: s.rawMouse });
    if (world && desc.lookAction && (s.sensitivity !== before.sensitivity || s.invertY !== before.invertY)) {
      const r = input.setSensitivity(world, { action: desc.lookAction, device: "keyboardMouse", scale: s.sensitivity, invertY: s.invertY }) as Api.Result<void>;
      if (!r.ok) throw new Error(`og.ui.settingsScreen: ${r.reason}`);
    }
    // The preset is RenderSettings' quality fields (og.render.applyQuality):
    // the next frame draws it, and the probe reads what is drawn. With the
    // graphics page it is the display's preset instead (above).
    if (s.quality !== before.quality && world && !s.display) {
      const r = render.applyQuality(desc.world as unknown as Parameters<typeof render.applyQuality>[0], s.quality) as Api.Result<void>;
      if (!r.ok) throw new Error(`og.ui.settingsScreen: ${r.reason}`);
    }
    // The vibration switch is the player's haptic strength (og.ui.setHaptics):
    // off silences the style's patterns on the pad and the phone.
    if (s.vibration !== before.vibration) kit.setHaptics(s.vibration ? 1 : 0);
    // The text size is the UI's (og.ui.configure keeps every other field);
    // without a world the game applies it in onChange.
    if (s.textScale !== before.textScale && desc.world) {
      const r = kit.configure(desc.world as never, { textScale: s.textScale }) as Api.Result<void>;
      if (!r.ok) throw new Error(`og.ui.settingsScreen: ${r.reason}`);
    }
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

/**
 * The PC graphics page over GameSettings.display: what og.render.display
 * answers for this device (V-Sync, the GPU the page loaded with) beside the
 * player's choices. Its rows are the body's, which scrolls past the safe area.
 */
function graphicsPage(s: Settings, d: Api.DisplaySettings, desc: Api.SettingsDesc, phone: boolean): void {
  const { t } = frame();
  const info = desc.world ? (render.display(desc.world as unknown as World) as Api.DisplayInfo) : null;
  const display = d as { -readonly [K in keyof Api.DisplaySettings]: Api.DisplaySettings[K] };
  kit.column("Page", { width: grow(), gap: t.sm }, () => {
    const quality = desc.qualityLabel ?? "Quality";
    kit.label("QualityLabel", quality, { step: "meta", color: token("inkMuted") });
    const levels = [desc.lowLabel ?? "Low", desc.mediumLabel ?? "Medium", desc.highLabel ?? "High"];
    // No preset of the player's is the game's own: shown as GameSettings.quality's.
    const shown = QUALITIES.indexOf(display.quality ?? s.quality);
    const q = kit.radio("Quality", levels, shown, { horizontal: !phone, label: quality });
    if (q.selected !== shown) {
      display.quality = QUALITIES[q.selected] ?? "medium";
      s.quality = display.quality;
    }
    const scale = display.renderScale ?? 1;
    display.renderScale = sliderRow("Scale", desc.renderScaleLabel ?? "Render scale", scale, { min: 0.5, max: 2, step: 0.05 }, `${Math.round(scale * 100)}%`);
    kit.row("Cap", { width: grow(), gap: t.md, cross: "center" }, () => {
      const label = desc.frameCapLabel ?? "Frame rate limit";
      kit.label("Label", label, { width: px(phone ? 96 : 140), nowrap: true });
      const caps = [desc.frameCapAutoLabel ?? "Automatic", desc.frameCapDisplayLabel ?? "Display's rate", "30", "60", "120", "144"];
      const at = Math.max(0, FRAME_CAPS.indexOf(display.frameCap ?? "auto"));
      const r = kit.select("Select", caps, at, { width: grow(), label }) as Api.ChoiceResult;
      display.frameCap = FRAME_CAPS[r.selected] ?? "auto";
    });
    kit.keyValue("Vsync", desc.vsyncLabel ?? "V-Sync", desc.vsyncValueLabel ?? "On (set by the browser)", false);
    display.autoQuality = (kit.toggle("Auto", display.autoQuality ?? true, desc.autoQualityLabel ?? "Adapt quality to hold the frame rate") as Api.ToggleResult).on;
    display.batterySaver = (kit.toggle("Battery", display.batterySaver ?? false, desc.batteryLabel ?? "Battery saver") as Api.ToggleResult).on;
    // The page leaving fullscreen (Escape) turns the setting off: show what the page is.
    const full = info ? info.settings.fullscreen ?? false : display.fullscreen ?? false;
    const f = (kit.toggle("Fullscreen", full, desc.fullscreenLabel ?? "Fullscreen") as Api.ToggleResult).on;
    if (f !== full || info) display.fullscreen = f;
    kit.row("Power", { width: grow(), gap: t.md, cross: "center" }, () => {
      const label = desc.powerLabel ?? "Graphics processor";
      kit.label("Label", label, { width: px(phone ? 96 : 140), nowrap: true });
      const at = Math.max(0, POWER.indexOf(display.powerPreference ?? "highPerformance"));
      const r = kit.select("Select", [desc.highPerformanceLabel ?? "High performance", desc.lowPowerLabel ?? "Low power"], at, { width: grow(), label }) as Api.ChoiceResult;
      display.powerPreference = POWER[r.selected] ?? "highPerformance";
    });
    // The adapter is chosen at the boot: another one waits for the next load.
    const restart = info ? info.restartNeeded || (info.presenting && display.powerPreference !== info.powerPreference) : false;
    if (restart) kit.label("Restart", desc.restartLabel ?? "Applies after a restart", { step: "meta", color: token("warn") });
    s.showFps = (kit.toggle("Fps", s.showFps, desc.fpsLabel ?? "Show frame rate") as Api.ToggleResult).on;
    s.reducedMotion = (kit.toggle("Motion", s.reducedMotion, desc.motionLabel ?? "Reduce motion") as Api.ToggleResult).on;
    s.highContrast = (kit.toggle("Contrast", s.highContrast, desc.contrastLabel ?? "High contrast") as Api.ToggleResult).on;
    s.textScale = sliderRow("TextSize", desc.textSizeLabel ?? "Text size", s.textScale, { min: 1, max: 2, step: 0.25 }, `${Math.round(s.textScale * 100)}%`);
  });
}
