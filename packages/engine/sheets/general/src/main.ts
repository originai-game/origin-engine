// The general components' review sheet (docs/ui.md §6.2): the store page
// (shop, purchase confirmation, login calendar), the progress page
// (achievements, daily quests), the social page (chat, friends) and the
// states page (each member empty, loading, failed or refused), dressed by a
// style pack. The pages use the built-in components by name, as a game
// does, with fixture stores; loading a page loads them.
//
// Commands: style:<fantasy>, page:<store|progress|social|states>,
// reduced:<on|off>.

import { og, type World } from "@origingame/engine";
import { INK_NAMES, PROGRESS, SOCIAL, STATES, STORE } from "./fixtures.ts";

const PAGES = ["store", "progress", "social", "states"] as const;
const STYLES = ["fantasy"] as const;

const DOCUMENTS = [
  "ui/store.svelte",
  "ui/progress.svelte",
  "ui/social.svelte",
  "ui/states.svelte",
  "ui/sheet.svelte",
];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The style and the icon set in one configure, so neither resets the other. */
function setStyle(world: World, name: string): void {
  const style = must(og.load.style(world, `@styles/${name}`), name);
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  og.ui.configure(world, { style, fallbackIcons: { texture: ink, columns: 8, rows: 4, names: INK_NAMES } });
}

/** Mounts the sheet, its stores the fixtures and the page shown. */
function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const data = { ...STORE, ...PROGRESS, ...SOCIAL, ...STATES, page: "store" };
  const stores = Object.fromEntries(Object.entries(data).map(([k, v]) =>
    [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
  must(scope.mount("sheet", "GeneralSheet", stores), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.04, g: 0.03, b: 0.02 };
    setStyle(world, "fantasy");
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "store").set(PAGES.includes(args[0] as never) ? args[0]! : "store");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the general kit." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
