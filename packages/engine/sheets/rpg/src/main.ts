// The rpg components' review sheet (docs/ui.md §6.2): the inventory page (backpack,
// item card, character) and the journal page (quest log, map and place,
// quest detail, conversation, compass; on a phone, party and turn order over
// the map), dressed by a style pack. The pages use the built-in components
// by name, as a game does, with fixture stores; loading a page loads them.
//
// Commands: style:<fantasy>, page:<inventory|journal|equipment|explore|combat>,
// reduced:<on|off>.

import { og, type World } from "@origingame/engine";
import { CMD_NAMES, EQUIPMENT, localized, ITEM_NAMES, SKILL_NAMES, INK_NAMES, INVENTORY, JOURNAL, PICTURES } from "./fixtures.ts";

const PAGES = ["inventory", "journal", "equipment", "explore", "combat"] as const;
const STYLES = ["fantasy"] as const;

const DOCUMENTS = [
  "ui/inventory.svelte",
  "ui/journal.svelte",
  "ui/equipment.svelte",
  "ui/explore.svelte",
  "ui/combat.svelte",
  "ui/sheet.svelte",
];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The style and both icon sets in one configure, so neither resets the other. */
function setStyle(world: World, name: string): void {
  const style = must(og.load.style(world, `@styles/${name}`), name);
  const items = must(og.load.texture(world, "art/items.png"), "items");
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  og.ui.configure(world, {
    style,
    icons: { texture: items, columns: 8, rows: 6, names: [...ITEM_NAMES.map((n) => `item-${n}`), ...SKILL_NAMES.map((n) => `skill-${n}`), ...CMD_NAMES.map((n) => `cmd-${n}`)] },
    fallbackIcons: { texture: ink, columns: 8, rows: 4, names: INK_NAMES },
  });
}

/** Mounts the sheet, its stores the fixtures and the page shown. */
/** The pictures, loaded in start (ui only reads); handles are stable per world. */
let pictures: Record<string, number> = {};
/** The fixtures in the active locale, read in start (ui may not call og.i18n). */
let fixtures: Record<string, unknown> = {};

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const data = { ...fixtures, page: "inventory" };
  const stores = Object.fromEntries(Object.entries(data).map(([k, v]) =>
    [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
  must(scope.mount("sheet", "RpgSheet", stores), "mount");
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
    pictures = Object.fromEntries(PICTURES.map((p) => [p, must(og.load.texture(world, `art/${p}.png`), p) as number]));
    // The kit's strings in the review's locales, as a game ships them (the
    // kit's English comes with the kit), and the sheet's own fixture text.
    for (const locale of ["en", "ja", "zh-CN", "ar"]) must(og.i18n.load(world, `strings/${locale}.ogt`), locale);
    fixtures = localized({ ...INVENTORY(pictures), ...JOURNAL(pictures), ...EQUIPMENT() }, (key) => og.i18n.t(world, key));
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "inventory").set(PAGES.includes(args[0] as never) ? args[0]! : "inventory");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the rpg kit." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
