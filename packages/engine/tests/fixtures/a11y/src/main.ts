// The accessibility mirror's game (tests/web/a11y.ts): what a screen reader
// is given of og.ui's controls, on two dialogs built from them. `show
// settings` opens Settings: Audio and Graphics tabs, a Master slider named
// by its row's label (and Music), Mute everything and Subtitles switches, a High contrast switch on
// Graphics and Done, which closes it with a "Settings saved" toast. `show
// pause` opens a pause dialog (it scales in, or fades under reduced
// motion); `show none` closes both.
import { og, defineResource, defineSystem, bool, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

const SCREENS = ["none", "settings", "pause"] as const;

const Ui = defineResource("A11yUi", { screen: u32(0), tab: u32(0), master: f32(1), music: f32(0.8), muted: bool(false), subtitles: bool(true), contrast: bool(false) });

/** A volume row: its label names the slider for a screen reader. */
function volume(id: string, label: string, value: number): number {
  let out = value;
  og.ui.row(id, { width: { kind: "grow" }, gap: 16, cross: "center" }, () => {
    og.ui.label("Label", label, { width: { kind: "fixed", px: 140 }, nowrap: true });
    out = og.ui.valueSlider("Slider", value, { step: 0.05, showValue: false, bubble: false }).value;
    og.ui.label("Value", `${Math.round(value * 100)}%`, { width: { kind: "fixed", px: 48 }, align: "end", nowrap: true });
  });
  return out;
}

const Screens = defineSystem("A11yScreens", { writes: [Ui] }, (_q, _t, world: World) => {
  const ui = world.resource(Ui);
  if (!ui.ok) return;
  const s = ui.value;
  const settings = og.ui.modal("Settings", s.screen === 1, { title: "Settings", width: { kind: "fixed", px: 600 } }, () => {
    s.tab = og.ui.segmented("Section", { items: [{ label: "Audio" }, { label: "Graphics" }], selected: s.tab, width: { kind: "grow" }, label: "Section" }).selected;
    if (s.tab === 0) {
      s.master = volume("Master", "Master", s.master);
      s.music = volume("Music", "Music", s.music);
      s.muted = og.ui.toggleSwitch("Mute", s.muted, { label: "Mute everything" }).on;
      s.subtitles = og.ui.toggleSwitch("Subtitles", s.subtitles, { label: "Subtitles" }).on;
    } else {
      s.contrast = og.ui.toggleSwitch("Contrast", s.contrast, { label: "High contrast" }).on;
    }
    og.ui.row("Footer", { width: { kind: "grow" }, align: "end" }, () => {
      if (og.ui.button("Done", "Done", { variant: "primary" }).clicked) {
        s.screen = 0;
        og.ui.toast(world, "Settings saved");
      }
    });
  });
  if (settings.dismissed) s.screen = 0;
  const pause = og.ui.modal("Pause", s.screen === 2, { title: "Paused" }, () => {
    if (og.ui.button("Resume", "Resume", { variant: "primary" }).clicked) s.screen = 0;
  });
  if (pause.dismissed) s.screen = 0;
});

export default og.game({
  tick: 60,
  resources: [Ui],
  systems: [Screens],
  commands: {
    show: (world, args) => {
      const ui = world.resource(Ui);
      const at = SCREENS.indexOf((args[0] ?? "none") as (typeof SCREENS)[number]);
      if (ui.ok && at >= 0) ui.value.screen = at;
    },
  },
  commandHelp: {
    show: { usage: `<${SCREENS.join("|")}>`, doc: "Open a dialog, or none." },
  },
});
