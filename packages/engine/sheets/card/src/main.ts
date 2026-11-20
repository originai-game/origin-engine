// The card components' review sheet (docs/ui.md §6.2): the table page (the
// hand between the draw pile and the discard and exhaust piles), the builder
// page (collection and deck list, the deck over its limit) and the run page
// (run map and a card reward), dressed by a style pack. The pages use the
// built-in components by name, as a game does, with fixture stores; loading
// a page loads them.
//
// Commands: style:<fantasy>, page:<table|builder|run>, reduced:<on|off>.

import { og, type World } from "@origingame/engine";
import { ART_NAMES, BUILDER, INK_NAMES, RUN, TABLE, localized } from "./fixtures.ts";

const PAGES = ["table", "builder", "run"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

let runArt = 0;

const DOCUMENTS = [
  "ui/table.svelte",
  "ui/builder.svelte",
  "ui/run.svelte",
  "ui/sheet.svelte",
];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The style and both icon sets in one configure, so neither resets the other:
 * the sheet's painted card art (`art-<name>`, sheet only) and the ink icons. */
function setStyle(world: World, name: string): void {
  const art = must(og.load.texture(world, "art/card-art.png"), "card art");
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const icons = { texture: art, columns: 4, rows: 3, pictures: true, names: ART_NAMES.map((n) => `art-${n}`) };
  const fallbackIcons = { texture: ink, columns: 8, rows: 4, names: INK_NAMES };
  const style = name === "default" ? null : must(og.load.style(world, `@styles/${name}`), name);
  og.ui.configure(world, { style, font: null, icons, fallbackIcons });
}

/** The fixture text in the active locale, read in start (ui may not call og.i18n). */
let text: Record<string, unknown> = {};

/** Mounts the sheet, its stores the fixtures and the page shown. */
function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const stores = Object.fromEntries(Object.entries({ ...text, runArt, page: "table" }).map(([k, v]) =>
    [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
  must(scope.mount("sheet", "CardSheet", stores), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.04, g: 0.03, b: 0.02 };
    runArt = must(og.load.texture(world, "art/runmap.jpg"), "run art") as number;
    setStyle(world, "fantasy");
    // The kit's strings in the review's CJK and right-to-left locales, as a
    // game ships them (the kit's English comes with the kit), and the
    // sheet's own fixture text in English and Chinese.
    for (const locale of ["en", "zh-CN", "ja", "ar"]) must(og.i18n.load(world, `strings/${locale}.ogt`), locale);
    text = localized({ ...TABLE, ...BUILDER, ...RUN }, (key) => og.i18n.t(world, key));
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "light" ? "light" : "dark" }),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "table").set(PAGES.includes(args[0] as never) ? args[0]! : "table");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style pack the pages are dressed in." },
    scheme: { usage: "<light|dark>", doc: "The colour scheme the style is drawn in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the card kit." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
