// The ungrouped built-in components' review sheet (docs/ui.md §6.2): the
// hud page (the normal page: health, mana and stamina, buffs, the ability
// bar, the minimap, compass and round timer, action hints, over a bright sky
// and a dark floor), the panels page (the stress page: a full inventory
// grid, a long tooltip, a menu with its submenu, a chart, a skill graph, the
// carousel, breadcrumbs and the overlay bar, with long names) and the states
// page (every member in every state: the documents' fixtures and the
// sheet's empty, overflow, disabled and tone states, src/fixtures.ts),
// dressed by a style pack or the default style. The pages use the built-in
// components by name, as a game does, with fixture stores. The world tags
// and ground marks are the world sheet's (sheets/world).
//
// Commands: style:<fantasy|scifi|cartoon|default>, page:<hud|panels|states>,
// reduced:<on|off>.

import { og, type UiReadable, type World } from "@origingame/engine";
import { FIXTURES, NAMES } from "./fixtures.ts";
import { sample } from "./sample.ts";

const PAGES = ["hud", "panels", "states"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

const DOCUMENTS = ["ui/hud.svelte", "ui/panels.svelte", "ui/states.svelte", "ui/sheet.svelte", "ui/sample.svelte"];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The shared ink icon sheet's cells (templates/styles/shared/ink-icons.png). */
const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

/**
 * The style and the fallback icons in one configure, so neither resets the
 * other. A pack brings its own fonts; the default style has none, so it is
 * given the sheet's font (without one the UI draws no text).
 */
function setStyle(world: World, name: string): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const fallbackIcons = { texture: ink, columns: 8, rows: 4, names: INK_NAMES };
  if (name === "default") {
    const font = must(og.load.font(world, "fonts/InstrumentSans-Regular.ttf"), "font");
    og.ui.configure(world, { style: null, font, fallbackIcons });
  } else og.ui.configure(world, { style: must(og.load.style(world, `@styles/${name}`), name), font: null, fallbackIcons });
}

/** The list's key: `id`, else `action` (an action hint's). */
const keyOf = (rows: unknown[]) => (rows.length && !("id" in (rows[0] as object)) ? "action" : "id");

/** Each fixture as a store of its scalars (`<member>_<state>`) and a keyed collection per list (`<member>_<state>_<prop>`). */
function stores(world: World): Record<string, UiReadable<unknown>> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, UiReadable<unknown>> = { page: scope.writable("page", "hud") };
  const captions: Record<string, string> = {};
  for (const [member, states] of Object.entries(FIXTURES)) {
    for (const [state, fixture] of Object.entries(states)) {
      const name = `${member}_${state}`;
      captions[name] = `${NAMES[member]} · ${state}`;
      out[name] = scope.writable(name, fixture);
      for (const [prop, value] of Object.entries(fixture)) {
        if (Array.isArray(value)) out[`${name}_${prop}`] = scope.collection(`${name}_${prop}`, value as { id: string }[], { key: keyOf(value) });
      }
    }
  }
  out.captions = scope.writable("captions", captions);
  return out;
}

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  must(scope.mount("sheet", "UngroupedSheet", stores(world)), "mount");
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
    sample,
    page: (world, args) => {
      show(world);
      og.ui.scope(world, "sheet").writable("page", "hud").set(PAGES.includes(args[0] as never) ? args[0]! : "hud");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    sample: { usage: "<member> [fixture]", doc: "One isolated panel or feedback fixture, such as tooltipCard overflow or starBar compact." },
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the ungrouped components." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
