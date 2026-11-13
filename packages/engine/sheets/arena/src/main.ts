// The arena components' review sheet (docs/ui.md §6.2): the select page
// (roster, draft and lobby: the normal pre-match flow), the match page
// (the stress page: the round HUD with its won banner over a scene, a
// ten-player scoreboard and the five-a-side versus, all with long names)
// and the states page (every member in every fixture state it declares),
// dressed by a style pack or the default style. The pages use the built-in
// components by name, as a game does, with fixture stores; portraits are
// the sample art under assets/art, matched to a fixture row by name.
//
// Commands: style:<fantasy|default>, page:<select|match|states>,
// reduced:<on|off>.

import { og, type World } from "@origingame/engine";
import { FIXTURES, NAMES } from "./fixtures.ts";

const PAGES = ["select", "match", "states"] as const;
const STYLES = ["fantasy", "default"] as const;
const PICTURES = ["brenna", "idris", "maren", "sela", "ysolde", "lurker", "oarsman"];

const DOCUMENTS = ["ui/select.svelte", "ui/match.svelte", "ui/states.svelte", "ui/sheet.svelte"];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason}`);
  return r.value;
}

/** The shared ink icon sheet's cells (templates/styles/shared/ink-icons.png): the roster's and lobby's glyphs. */
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

/** The pictures, loaded in start (ui only reads); handles are stable per world. */
let pictures: Record<string, number> = {};

/** A row's portrait from the sample art when its character (or name) has one; other rows keep the icon. */
function withArt(value: unknown): unknown {
  if (!Array.isArray(value)) return value;
  return value.map((row: Record<string, unknown>) => {
    if (!("portrait" in row)) return row;
    const who = String(row.character || row.name || "").split(/[ ,]/)[0]!.toLowerCase();
    return pictures[who] ? { ...row, portrait: pictures[who] } : row;
  });
}

/** Each fixture as a store of its scalars (`<member>_<state>`) and a keyed collection per list (`<member>_<state>_<prop>`). */
function stores(world: World): Record<string, unknown> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, unknown> = { page: scope.writable("page", "select") };
  const captions: Record<string, string> = {};
  for (const [member, states] of Object.entries(FIXTURES)) {
    for (const [state, fixture] of Object.entries(states)) {
      const name = `${member}_${state}`;
      captions[name] = `${NAMES[member]} · ${state}`;
      out[name] = scope.writable(name, fixture);
      for (const [prop, value] of Object.entries(fixture)) {
        if (Array.isArray(value)) out[`${name}_${prop}`] = scope.collection(`${name}_${prop}`, withArt(value) as { id: string }[], { key: "id" });
      }
    }
  }
  out.captions = scope.writable("captions", captions);
  return out;
}

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  must(scope.mount("sheet", "ArenaSheet", stores(world)), "mount");
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
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "select").set(PAGES.includes(args[0] as never) ? args[0]! : "select");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the arena kit." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
