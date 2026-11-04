// The RPG kit's review sheet (docs/ui.md §17.3): each page builds the
// kit's components from fixtures/ on a style pack's ground. `og ui sheet
// rpg` renders every page of ../project.ogt at desktop and phone, 100 %
// and 150 %. The kit is imported by its relative path, so the sheet always
// shows the copy it ships with: bundled, or a game's edited install.
//
// Commands: style:<fantasy|scifi|cartoon|pixel>, page:<inventory|quests|dialogue|frames>.

import { og, defineSystem, type World } from "@origingame/engine";
import { rpg } from "../../index.ts";
import { QUESTS, bag, cast, conversation, gear, hero, party, raid, target, turns } from "../../fixtures/party.ts";

const PAGES = ["inventory", "quests", "dialogue", "frames"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "pixel"] as const;
const state = { page: "inventory" as (typeof PAGES)[number], shownAt: -1 };

/** The page's ground: the pack's backdrop over the whole screen, inset by the safe area. */
function ground(build: () => void): void {
  const s = og.ui.frameViewport().safeArea;
  og.ui.column("Page", {
    width: { kind: "grow" }, height: { kind: "grow" }, background: { token: "backdrop" },
    padding: { left: s.left + 16, top: s.top + 16, right: s.right + 16, bottom: s.bottom + 16 }, gap: 12,
  }, build);
}

const phone = (): boolean => og.ui.frameViewport().breakpoint === "phonePortrait";

function inventory(): void {
  ground(() => {
    // A phone stacks the gear over the bag and scrolls them, as a game's inventory screen does.
    const body = () => og.ui.row("Body", { width: { kind: "grow" }, gap: 16, wrap: phone() }, () => {
      rpg.paperdoll("Gear", { ...gear(), width: phone() ? { kind: "grow" } : { kind: "fixed", px: 420 } }, () => {});
      rpg.inventoryGrid("Bag", bag());
    });
    if (phone()) og.ui.scrollArea("Scroll", { width: { kind: "grow" }, height: { kind: "grow" } }, body);
    else body();
  });
}

function frames(): void {
  ground(() => {
    og.ui.row("Top", { width: { kind: "grow" }, align: "spaceBetween", wrap: phone(), gap: 12 }, () => {
      rpg.unitFrame("Hero", hero());
      rpg.unitFrame("Target", target());
    });
    rpg.turnOrder("Turns", turns());
    rpg.castBar("Cast", cast());
    og.ui.row("Groups", { gap: 16, wrap: true }, () => {
      rpg.partyFrames("Party", party());
      if (!phone()) rpg.raidFrames("Raid", raid());
    });
  });
  // The tracked quests over the game, as a HUD shows them.
  rpg.questTracker("Tracker", { quests: QUESTS.filter((q) => q.status === "active"), max: 2, anchor: phone() ? "bottomRight" : "right" });
}

const BUILD: Record<(typeof PAGES)[number], () => void> = {
  inventory,
  quests: () => ground(() => rpg.questLog("Log", { quests: QUESTS, selected: "q1", title: "Journal" })),
  dialogue: () => ground(() => rpg.conversation("Talk", { view: conversation(), charsPerSecond: 0 })),
  frames,
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
    setStyle(world, "fantasy");
  },
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (_world, args) => {
      state.page = PAGES.includes(args[0] as never) ? (args[0] as (typeof PAGES)[number]) : PAGES[0];
      state.shownAt = -1;
    },
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the RPG kit." },
  },
});
