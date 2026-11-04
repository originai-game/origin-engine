// The strategy kit's review sheet (docs/ui.md §17.3): each page builds the
// kit's components from fixtures/ on a style pack's ground. `og ui sheet
// strategy` renders every page of ../project.ogt at desktop and phone, 100 %
// and 150 %. The kit is imported by its relative path, so the sheet always
// shows the copy it ships with: bundled, or a game's edited install.
//
// Commands: style:<fantasy|scifi|cartoon|pixel>, page:<console|defence>.

import { og, defineSystem, type World } from "@origingame/engine";
import { strategy } from "../../index.ts";
import { commands, resources, waves } from "../../fixtures/base.ts";

const PAGES = ["console", "defence"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "pixel"] as const;
const state = { page: "console" as (typeof PAGES)[number], shownAt: -1 };

/** The page's ground: the pack's backdrop over the whole screen, inset by the safe area. */
function ground(build: () => void): void {
  const s = og.ui.frameViewport().safeArea;
  og.ui.column("Page", {
    width: { kind: "grow" }, height: { kind: "grow" }, background: { token: "backdrop" },
    padding: { left: s.left + 16, top: s.top + 16, right: s.right + 16, bottom: s.bottom + 16 }, gap: 12,
  }, build);
}

const phone = (): boolean => og.ui.frameViewport().breakpoint === "phonePortrait";

const BUILD: Record<(typeof PAGES)[number], () => void> = {
  console: () => {
    ground(() => {});
    // On a phone the console's minimap takes the top end: the resources keep to the start, clear of it.
    if (phone()) {
      const w = og.ui.frameViewport().width;
      og.ui.anchor("Top", { anchor: "topLeft", margin: 12 }, () => og.ui.column("Room", { width: { kind: "fixed", px: Math.round(w * 0.52) } }, () => strategy.resourceBar("Resources", resources())));
    } else og.ui.anchor("Top", { anchor: "top", margin: 12 }, () => strategy.resourceBar("Resources", resources()));
    strategy.rtsCommands("Rts", commands());
  },
  defence: () => {
    ground(() => {});
    og.ui.anchor("Top", { anchor: "top", margin: 12 }, () => strategy.resourceBar("Purse", { ...resources(), clock: null, rates: false }));
    og.ui.anchor("Waves", { anchor: "right", margin: 12 }, () => strategy.wavePreview("Waves", waves()));
  },
};

const Sheet = defineSystem("Sheet", { reads: [og.Camera] }, () => {
  if (state.shownAt < 0) state.shownAt = og.ui.tick();
  BUILD[state.page]();
});

function setStyle(world: World, name: string): void {
  const loaded = og.load.style(world, `@styles/${name}/style.ogt`);
  if (!loaded.ok) throw new Error(loaded.reason);
  og.ui.configure(world, { style: loaded.value });
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Sheet],
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.08, g: 0.09, b: 0.11 };
    setStyle(world, "scifi");
  },
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "scifi"),
    page: (_world, args) => {
      state.page = PAGES.includes(args[0] as never) ? (args[0] as (typeof PAGES)[number]) : PAGES[0];
      state.shownAt = -1;
    },
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the strategy kit." },
  },
});
