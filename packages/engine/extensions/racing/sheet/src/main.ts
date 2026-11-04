// The racing kit's review sheet (docs/ui.md §17.3): each page builds the
// kit's components from fixtures/ on a style pack's ground. `og ui sheet
// racing` renders every page of ../project.ogt at desktop and phone, 100 %
// and 150 %. The kit is imported by its relative path, so the sheet always
// shows the copy it ships with: bundled, or a game's edited install.
//
// The results page mounts the kit's race-results document as og ui sheet
// installs it, at ui-kits/racing/, with the document's own fixture.
//
// Commands: style:<fantasy|scifi|cartoon|pixel>, page:<hud|arcade|garage|results>.

import { og, defineSystem, type World } from "@origingame/engine";
import { racing } from "../../index.ts";
import { checkpoint, garage, lap, standings, tach } from "../../fixtures/race.ts";

const PAGES = ["hud", "arcade", "garage", "results"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "pixel"] as const;
const state = { page: "hud" as (typeof PAGES)[number], shownAt: -1 };

function ground(build: () => void): void {
  const vp = og.ui.frameViewport();
  const s = vp.safeArea;
  og.ui.column("Page", {
    width: { kind: "grow" }, height: { kind: "grow" }, background: { token: "backdrop" },
    padding: { left: s.left + 16, top: s.top + 16, right: s.right + 16, bottom: s.bottom + 16 }, gap: 12,
  }, build);
}

function hud(): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait";
  const k = phone ? 0.75 : 1;
  if (state.shownAt < 0) state.shownAt = og.ui.tick();
  ground(() => {
    og.ui.column("Lead", { gap: 8 }, () => {
      racing.lapTimer("Head", lap("head"));
      racing.lapTimer("Times", lap("times"));
      racing.standingsTower("Standings", standings());
    });
  });
  og.ui.anchor("Instrument", { anchor: phone ? "bottom" : "bottomRight", margin: 16 }, () => racing.speedometer("Speed", tach(Math.round(220 * k))));
  // A moment half a second old on every frame, so the capture always holds it.
  racing.raceBanner("Banner", { ...checkpoint(og.ui.tick() - 30), below: "" });
}

function arcade(): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait";
  ground(() => {
    racing.lapTimer("Head", { ...lap("head"), placeBadge: true, positionText: "4th" });
  });
  og.ui.anchor("ItemHud", { anchor: "topRight", margin: 16 }, () => racing.itemSlot("Item", { label: "Boost", size: phone ? 64 : 84, anchor: null }));
  racing.driftScore("Drift", { score: 2450, multiplier: 3, total: 18_200, label: "Drift", at: { x: Math.round(vp.width * 0.62), y: Math.round(vp.height * 0.42) } });
  og.ui.anchor("Instrument", { anchor: "bottomRight", margin: 16 }, () => racing.speedometer("Speed", { style: "arcade", speed: 148, maxSpeed: 200, unit: "km/h", boost: 0.7, boostSegments: 4, size: phone ? 150 : 200, anchor: null }));
}

const Sheet = defineSystem("Sheet", { reads: [og.Camera] }, () => {
  switch (state.page) {
    case "hud": hud(); break;
    case "arcade": arcade(); break;
    case "garage": ground(() => racing.garage("Garage", { ...garage(), title: "Garage" })); break;
    case "results": og.ui.screen("race-results", null); break;
  }
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
    const doc = og.load.document(world, "ui-kits/racing/race-results.ogt");
    if (!doc.ok) throw new Error(doc.reason);
  },
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "scifi"),
    page: (_world, args) => {
      state.page = PAGES.includes(args[0] as never) ? (args[0] as (typeof PAGES)[number]) : "hud";
      state.shownAt = -1;
    },
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the racing kit." },
  },
});
