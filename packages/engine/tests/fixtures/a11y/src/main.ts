// The accessibility mirror's game (tests/web/a11y.ts): what a screen reader
// is given of a document's controls, on two dialogs (assets/ui/a11y.svelte).
// `show settings` opens Settings: Audio and Graphics tabs, a Master slider
// named by its label (and Music), Mute everything and Subtitles switches, a
// High contrast switch on Graphics and Done, which closes it with a
// "Settings saved" toast. `show pause` opens a pause dialog (it scales in,
// or fades under reduced motion); `show none` closes both.
import { og, defineSystem } from "@origingame/engine";
import type { UiWritable } from "@origingame/engine";

const SCREENS = ["none", "settings", "pause"] as const;

let settings: UiWritable<boolean> | undefined;
let pause: UiWritable<boolean> | undefined;

const Screens = defineSystem("A11yScreens", {}, (_q, _t, world) => {
  for (const e of og.ui.docEvents(world)) {
    if (e.kind !== "emit") continue;
    if (e.name === "done") {
      settings?.set(false);
      og.ui.toast(world, "Settings saved");
    } else if (e.name === "resume") pause?.set(false);
  }
});

export default og.game({
  tick: 60,
  systems: [Screens],
  start(world) {
    const loaded = og.load.document(world, "ui/a11y.svelte");
    if (!loaded.ok) throw new Error(loaded.reason);
  },
  ui(world) {
    const scope = og.ui.scope(world, "a11y");
    settings = scope.writable("settings", false);
    pause = scope.writable("pause", false);
    const inputs = {
      settings,
      pause,
      tab: scope.writable("tab", "audio"),
      master: scope.writable("master", 1),
      music: scope.writable("music", 0.8),
      muted: scope.writable("muted", false),
      subtitles: scope.writable("subtitles", true),
      contrast: scope.writable("contrast", false),
    };
    const mounted = scope.mount("screens", "a11y", inputs);
    if (!mounted.ok) throw new Error(mounted.reason);
  },
  commands: {
    show: (_world, args) => {
      const at = SCREENS.indexOf((args[0] ?? "none") as (typeof SCREENS)[number]);
      if (at < 0) return;
      settings?.set(at === 1);
      pause?.set(at === 2);
    },
  },
  commandHelp: {
    show: { usage: `<${SCREENS.join("|")}>`, doc: "Open a dialog, or none." },
  },
});
