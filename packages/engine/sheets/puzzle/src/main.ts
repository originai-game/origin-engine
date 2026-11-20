// The puzzle components' review sheet (docs/ui.md §6.2): the play page (goals
// and moves over the board, the booster tray with one armed and one
// unavailable), the map page (a two-chapter level path beside a won level's
// summary with a new best) and the states page (every member in every
// fixture state it declares), dressed by a style pack or the default style.
// The pages use the built-in components by name, as a game does, with
// fixture stores.
//
// Commands: style:<fantasy|default>, page:<play|map|states>, reduced:<on|off>.

import { og, type UiReadable, type World } from "@origingame/engine";
import { FIXTURES, NAMES, localized } from "./fixtures.ts";

/** The sheet's painted art (art/pieces.png, 4 × 3, sheet only), drawn for the icons
 * its fixtures name; the ink icons stand in for the rest. */
const ART_NAMES = ["apple", "drop", "leaf", "gem", "coin", "bomb", "bolt", "plus", "fire", "key"];

/** The fixtures in the active locale, read in start (ui may not call og.i18n). */
let text = FIXTURES;

const PAGES = ["play", "map", "states"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

/** The map page's sample level map art (sheet only, never installed). */
let meadow = 0;

const DOCUMENTS = ["ui/play.svelte", "ui/map.svelte", "ui/states.svelte", "ui/sheet.svelte"];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The shared ink icon sheet's cells (templates/styles/shared/ink-icons.png): the goals' and boosters' icons and the lock. */
const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

/** The style and the fallback icons in one configure, so neither resets the other. */
function setStyle(world: World, name: string): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const art = must(og.load.texture(world, "art/pieces.png"), "painted art");
  const style = name === "default" ? null : must(og.load.style(world, `@styles/${name}`), name);
  og.ui.configure(world, { style, font: null, icons: { texture: art, columns: 4, rows: 3, pictures: true, names: ART_NAMES }, fallbackIcons: { texture: ink, columns: 8, rows: 4, names: INK_NAMES } });
}

/** Each fixture as a store of its scalars (`<member>_<state>`) and a keyed collection per list (`<member>_<state>_<prop>`). */
function stores(world: World): Record<string, UiReadable<unknown>> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, UiReadable<unknown>> = { page: scope.writable("page", "play"), member: scope.writable("member", ""), meadow: scope.writable("meadow", meadow) };
  const captions: Record<string, string> = {};
  for (const [member, states] of Object.entries(text)) {
    for (const [state, fixture] of Object.entries(states)) {
      const name = `${member}_${state}`;
      captions[name] = `${NAMES[member]} · ${state}`;
      out[name] = scope.writable(name, fixture);
      for (const [prop, value] of Object.entries(fixture)) {
        if (Array.isArray(value)) out[`${name}_${prop}`] = scope.collection(`${name}_${prop}`, value as { id: string }[], { key: "id" });
      }
    }
  }
  out.captions = scope.writable("captions", captions);
  return out;
}

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  must(scope.mount("sheet", "PuzzleSheet", stores(world)), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.05, g: 0.04, b: 0.05 };
    meadow = must(og.load.texture(world, "art/meadow.jpg"), "meadow") as number;
    setStyle(world, "fantasy");
    // The kit's strings in the review's CJK and right-to-left locales, as a
    // game ships them (English comes with the kit), and the fixture words
    // in Chinese.
    for (const locale of ["zh-CN", "ja", "ar"]) must(og.i18n.load(world, `strings/${locale}.ogt`), locale);
    text = localized(FIXTURES, (key) => (og.i18n.has(world, key) ? og.i18n.t(world, key) : key));
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "light" ? "light" : "dark" }),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "play").set(PAGES.includes(args[0] as never) ? args[0]! : "play");
    },
    member: (world, args) => og.ui.scope(world, "sheet").writable("member", "").set(args[0] ?? ""),
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    scheme: { usage: "<light|dark>", doc: "The colour scheme the style is drawn in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the puzzle kit." },
    member: { usage: "<goals|boosters|level-path|summary>", doc: "Show one component's fixtures on the states page; empty shows all." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
