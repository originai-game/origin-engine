// The accessibility mirror's game (tests/web/a11y.ts): what a screen reader
// is given of a document's controls, on two dialogs (assets/ui/a11y.tsx).
// `show settings` opens Settings: Audio and Graphics tabs, a Master slider
// named by its label (and Music), Mute everything and Subtitles switches, a
// High contrast switch on Graphics and Done, which closes it with a
// "Settings saved" polite status. `show pause` opens a pause dialog (it scales in,
// or fades under reduced motion); `show none` closes both.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { mount, useStore } from "@origingame/engine/ui";
import { A11y } from "../assets/ui/a11y.tsx";

const SCREENS = ["none", "settings", "pause"] as const;

// The UI's state: stores in the presentation checkpoint, which the TSX
// follows. Commands and handlers write them between ticks.
const store = (world: World) => {
  const scope = og.ui.scope(world, "a11y");
  return {
    settings: scope.writable("settings", false),
    pause: scope.writable("pause", false),
    tab: scope.writable<"audio" | "graphics">("tab", "audio"),
    master: scope.writable("master", 1),
    music: scope.writable("music", 0.8),
    muted: scope.writable("muted", false),
    subtitles: scope.writable("subtitles", true),
    contrast: scope.writable("contrast", false),
    saved: scope.writable("saved", ""),
  };
};

export default og.game({
  tick: 60,
  ui(world) {
    const s = store(world);
    mount(world, () => {
      const v = Object.fromEntries(Object.entries(s).map(([k, w]) => [k, useStore<unknown>(w)])) as { [K in keyof typeof s]: () => (typeof s)[K] extends { set(v: infer T): unknown } ? T : never };
      return A11y({
        get settings() { return v.settings(); },
        get pause() { return v.pause(); },
        get tab() { return v.tab(); },
        get master() { return v.master(); },
        get music() { return v.music(); },
        get muted() { return v.muted(); },
        get subtitles() { return v.subtitles(); },
        get contrast() { return v.contrast(); },
        get saved() { return v.saved(); },
        onTab: (_e, tab) => s.tab.set(tab),
        onVolume: (_e, which, value) => s[which].set(value),
        onSwitch: (_e, which) => s[which].update((on) => !on),
        onDone: () => { s.settings.set(false); s.saved.set("Settings saved"); },
        onResume: () => s.pause.set(false),
      });
    }, {}, { layer: "modal" });
  },
  commands: {
    show: (world, args) => {
      const at = SCREENS.indexOf((args[0] ?? "none") as (typeof SCREENS)[number]);
      if (at < 0) return;
      store(world).settings.set(at === 1);
      store(world).pause.set(at === 2);
    },
  },
  commandHelp: {
    show: { usage: `<${SCREENS.join("|")}>`, doc: "Open a dialog, or none." },
  },
});
