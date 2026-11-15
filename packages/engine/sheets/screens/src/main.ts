// The built-in screens' review sheet (docs/ui.md §6.2): every screen in
// src/ui/screens, each on a page of its own (a screen is the whole
// viewport, so no two share one), in each state src/fixtures.ts declares
// (the document's fixture and the sheet's empty, overflow, locked and the
// rest), dressed by a style pack or the default style. A layer (pause,
// results, the film screens) is pushed as a game pushes it, with the
// state's fixture as its viewmodel; a component screen (card, table,
// confirm) is mounted with the state's fixture as its stores.
//
// Commands: style:<fantasy|scifi|cartoon|default>, page:<screen>[:<state>],
// state:<state> (og ui review --states), reduced:<on|off>.

import { og, type UiReadable, type World } from "@origingame/engine";
import { FIXTURES, NAMES } from "./fixtures.ts";

const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

/** The screens that are layers (`<meta name="screen">`); the rest are components. */
const LAYERS = new Set([
  "calibration", "creator-tools", "credits", "film-chapters", "film-choices", "film-flowchart", "film-gallery", "game-over",
  "hud-editor", "leaderboard", "loading", "main-menu", "mode-select", "notifications", "pause", "results", "save-load",
  "settings", "splash", "subtitles", "title", "tutorial",
]);
/** Every built-in screen, by its id (literal, so the cook bundles each). */
const DOCUMENTS = [
  "@ui/screens/banner.svelte",
  "@ui/screens/calendar.svelte",
  "@ui/screens/calibration.svelte",
  "@ui/screens/card.svelte",
  "@ui/screens/confirm.svelte",
  "@ui/screens/creator-tools.svelte",
  "@ui/screens/credits.svelte",
  "@ui/screens/data-list.svelte",
  "@ui/screens/film-chapters.svelte",
  "@ui/screens/film-choices.svelte",
  "@ui/screens/film-flowchart.svelte",
  "@ui/screens/film-gallery.svelte",
  "@ui/screens/game-over.svelte",
  "@ui/screens/hud-editor.svelte",
  "@ui/screens/key-values.svelte",
  "@ui/screens/leaderboard.svelte",
  "@ui/screens/loading.svelte",
  "@ui/screens/main-menu.svelte",
  "@ui/screens/mode-select.svelte",
  "@ui/screens/notifications.svelte",
  "@ui/screens/objective-tracker.svelte",
  "@ui/screens/pagination.svelte",
  "@ui/screens/pause.svelte",
  "@ui/screens/results.svelte",
  "@ui/screens/save-load.svelte",
  "@ui/screens/settings.svelte",
  "@ui/screens/splash.svelte",
  "@ui/screens/star-rating.svelte",
  "@ui/screens/subtitles.svelte",
  "@ui/screens/table.svelte",
  "@ui/screens/title.svelte",
  "@ui/screens/tree.svelte",
  "@ui/screens/tutorial.svelte",
];
const MEMBERS = new Map(Object.entries(NAMES).map(([member, name]) => [name, member]));

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/**
 * The shared ink icon sheet's cells (templates/styles/shared/ink-icons.png),
 * and a row of chevrons (its `back` turned) for pagination, calendar and tree.
 */
const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
  "chevronLeft", "chevronRight", "chevronUp", "chevronDown",
];

/**
 * The style and the fallback icons in one configure, so neither resets the
 * other. A pack brings its own fonts; the default style has none, so it is
 * given the sheet's font (without one the UI draws no text).
 */
function setStyle(world: World, name: string): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const fallbackIcons = { texture: ink, columns: 8, rows: 5, names: INK_NAMES };
  if (name === "default") {
    const font = must(og.load.font(world, "fonts/InstrumentSans-Regular.ttf"), "font");
    og.ui.configure(world, { style: null, font, fallbackIcons });
  } else og.ui.configure(world, { style: must(og.load.style(world, `@styles/${name}`), name), font: null, fallbackIcons });
}

/** The page shown and its state. */
let shown = { screen: "", state: "default" };

/** The fixture of a screen's state (its default where it declares no such state). */
const fixture = (screen: string, state: string) => {
  const states = FIXTURES[MEMBERS.get(screen)!]!;
  return states[state] ?? states.default!;
};

/** A component screen's state as its stores: a writable per scalar prop, a keyed collection per list. */
function stores(world: World, screen: string, state: string): Record<string, UiReadable<unknown>> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, UiReadable<unknown>> = {};
  for (const [prop, value] of Object.entries(fixture(screen, state))) {
    const id = `${screen}_${state}_${prop}`;
    out[prop] = Array.isArray(value) ? scope.collection(id, value as { id: string }[], { key: "id" }) : scope.writable(id, value);
  }
  return out;
}

function show(world: World, screen: string, state: string): void {
  if (!MEMBERS.has(screen)) throw new Error(`no screen ${screen}: ${[...MEMBERS.keys()].join(", ")}`);
  const scope = og.ui.scope(world, "sheet");
  og.ui.clear(world);
  must(scope.unmount("page"), "unmount");
  shown = { screen, state };
  if (LAYERS.has(screen)) must(og.ui.push(world, screen as never, fixture(screen, state) as never), screen);
  else must(scope.mount("page", screen, stores(world, screen, state)), screen);
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
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (world, args) => show(world, args[0] ?? "pause", args[1] ?? "default"),
    state: (world, args) => show(world, shown.screen || "pause", args[0] ?? "default"),
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    page: { usage: "<screen>[:<state>]", doc: "A built-in screen, in its default state or the one named." },
    state: { usage: "<state>", doc: "The page's screen in a state of src/fixtures.ts (its default where it has none of the name)." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
