// The shooter kit's review sheet (docs/ui.md §6.2): the normal page (combat
// HUD over a bright sky and a dark ground) and the stress page (a loadout
// with long names, locked and equipped slots, and a dense killfeed), in a
// style pack. The kit is installed into assets/ui-kits/shooter as a game
// installs it; the pages mount its documents with fixture stores.
//
// Commands: style:<clean>, page:<combat|loadout>.

import { og, type World } from "@origingame/engine";
import { COMBAT, FEED, LOADOUT } from "./fixtures.ts";

const PAGES = ["combat", "loadout"] as const;
const STYLES = ["clean"] as const;

const DOCUMENTS = [
  "ui/stat-bar.svelte",
  "ui-kits/shooter/crosshair.svelte",
  "ui-kits/shooter/ammo-counter.svelte",
  "ui-kits/shooter/damage-direction.svelte",
  "ui-kits/shooter/killfeed.svelte",
  "ui-kits/shooter/loadout.svelte",
  "ui-kits/shooter/combat-hud.svelte",
  "ui/combat.svelte",
  "ui/loadout.svelte",
  "ui/sheet.svelte",
];

function setStyle(world: World, name: string): void {
  const loaded = og.load.style(world, `@styles/${name}/style.ogt`);
  if (!loaded.ok) throw new Error(loaded.reason);
  og.ui.configure(world, { style: loaded.value });
}

/** Mounts the sheet, its stores the fixtures and the page shown. */
function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const stores = Object.fromEntries(Object.entries({ ...COMBAT, feed: FEED, ...LOADOUT, page: "combat" }).map(([k, v]) =>
    [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
  const mounted = scope.mount("sheet", "ShooterSheet", stores);
  if (!mounted.ok) throw new Error(mounted.reason);
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.05, g: 0.06, b: 0.08 };
    setStyle(world, "clean");
    for (const path of DOCUMENTS) {
      const loaded = og.load.document(world, path);
      if (!loaded.ok) throw new Error(`${path}: ${loaded.reason}`);
    }
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "clean"),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "combat").set(PAGES.includes(args[0] as never) ? args[0]! : "combat");
    },
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the shooter kit." },
  },
});
