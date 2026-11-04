// The shooter kit's review sheet (docs/ui.md §17.3): each page builds the
// kit's components from fixtures/ on a style pack's ground. `og ui sheet
// shooter` renders every page of ../project.ogt at desktop and phone, 100 %
// and 150 %. The kit is imported by its relative path, so the sheet always
// shows the copy it ships with: bundled, or a game's edited install.
//
// Commands: style:<fantasy|scifi|cartoon|pixel>, page:<hud|scoreboard|loadout>.

import { og, defineSystem, type World } from "@origingame/engine";
import { shooter } from "../../index.ts";
import { FEED, ammo, loadout, scoreboard } from "../../fixtures/match.ts";

const PAGES = ["hud", "scoreboard", "loadout"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "pixel"] as const;
const state = { page: "hud" as (typeof PAGES)[number], shownAt: -1 };

/** The page's ground: the pack's backdrop over the whole screen, inset by the safe area. */
function ground(build: () => void): void {
  const s = og.ui.frameViewport().safeArea;
  og.ui.column("Page", {
    width: { kind: "grow" }, height: { kind: "grow" }, background: { token: "backdrop" },
    padding: { left: s.left + 16, top: s.top + 16, right: s.right + 16, bottom: s.bottom + 16 }, gap: 12,
  }, build);
}

const phone = (): boolean => og.ui.frameViewport().breakpoint === "phonePortrait";

function hud(): void {
  // Hits a sixth of a second old on every frame, so the capture always holds them.
  const t = og.ui.tick() - 10;
  ground(() => {});
  shooter.lowHealthVignette("Vignette", { health: 0.28, threshold: 0.35, hitAt: t });
  shooter.damageDirection("Hits", { hits: [{ angle: 0.9, at: t, strength: 0.8 }, { angle: -2.2, at: t, strength: 0.45 }], durationMs: 1400 });
  shooter.crosshair("Crosshair", { style: "cross", spread: 0.3, dot: true, hitAt: t, hit: "headshot", onTarget: true });
  if (!phone()) shooter.killfeed("Feed", { entries: FEED.map((e, i) => ({ ...e, at: t - 60 * (i + 1) })), max: 4, durationMs: 60_000, anchor: "topRight" });
  shooter.ammoCounter("Ammo", ammo());
}

const BUILD: Record<(typeof PAGES)[number], () => void> = {
  hud,
  scoreboard: () => { ground(() => {}); shooter.scoreboard("Board", scoreboard()); },
  loadout: () => ground(() => shooter.loadout("Loadout", { ...loadout(), title: "Loadout" })),
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
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the shooter kit." },
  },
});
