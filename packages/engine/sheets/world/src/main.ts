// The world-anchored components' review sheet (docs/ui.md §6.2): the world
// tags (nameplate, loot label, interaction prompt, waypoint, speech bubble),
// the ground marks (selection ring, telegraph, path line, placement ghost)
// and the mass elements (overhead bars, damage numbers), anchored over a lit
// 3D scene as a game anchors them: the field page (the normal page: every
// kind and state spread over the ground) and the crowd page (the stress
// page: bunched units, long names, twenty-four bars and ten hits at once),
// dressed by a style pack or the default style. The units are boxes at the
// points the fixtures name (src/fixtures.ts).
//
// Damage numbers rise and fade within a second of their hit, so each screen
// sends `hits` just before its capture: every hit again, as new rows.
//
// Commands: style:<fantasy|scifi|cartoon|default>, page:<field|crowd>, hits,
// reduced:<on|off>.

import { og, defineSystem, type UiReadable, type World } from "@origingame/engine";
import { FIXTURES, NAMES } from "./fixtures.ts";

const PAGES = ["field", "crowd"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

const DOCUMENTS = ["ui/field.svelte", "ui/crowd.svelte", "ui/sheet.svelte"];

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

/** The units the tags stand over: x z on the ground, height, colour. */
const UNITS: Array<[number, number, number, { r: number; g: number; b: number }]> = [
  [-4, -1, 2.0, { r: 0.45, g: 0.16, b: 0.12 }], // the boss
  [3, -2, 1.7, { r: 0.2, g: 0.42, b: 0.22 }], // the friend
  [0, 2, 1.7, { r: 0.22, g: 0.32, b: 0.55 }], // the player
  [7, -4, 1.6, { r: 0.5, g: 0.44, b: 0.3 }], // the merchant
  [-2, -3, 1.8, { r: 0.4, g: 0.2, b: 0.15 }], // the crowd
  [-1.4, -3.2, 1.6, { r: 0.35, g: 0.18, b: 0.14 }],
  [2, 0, 0.8, { r: 0.42, g: 0.3, b: 0.16 }], // the chest
  [4, -0.5, 1.2, { r: 0.3, g: 0.3, b: 0.32 }], // the vault
];

function scene(world: World): void {
  const box = must(og.render.boxGeometry(world, {}), "box");
  const solid = (x: number, y: number, z: number, size: { x: number; y: number; z: number }, color: { r: number; g: number; b: number }) => {
    const e = must(world.spawn({ position: { x, y, z } }), "spawn");
    must(world.set(e, og.Transform, { scale: size }), "scale");
    must(world.insert(e, og.Mesh, { geometry: box, material: must(og.render.standardMaterial(world, { color, roughness: 0.85 }), "material"), castShadow: true }), "mesh");
  };
  // Grass and a stone road: a bright and a dark ground under the tags.
  solid(0, -0.25, -10, { x: 80, y: 0.5, z: 60 }, { r: 0.24, g: 0.36, b: 0.16 });
  solid(0, -0.2, -6, { x: 5, y: 0.5, z: 40 }, { r: 0.4, g: 0.38, b: 0.34 });
  for (const [x, z, h, color] of UNITS) solid(x, h / 2, z, { x: 0.8, y: h, z: 0.8 }, color);
  const sun = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.5 }) }), "sun");
  must(world.insert(sun, og.DirectionalLight, { color: { r: 1, g: 0.95, b: 0.85 }, intensity: 2.5, castShadow: true }), "light");
  const cam = must(world.resource(og.Camera), "camera");
  cam.background = { r: 0.55, g: 0.7, b: 0.85 };
  frame(world);
}

/** Where the camera looks, and from where at 16:9. */
const TARGET = { x: 0, y: 0.5, z: -3 };
const FROM = { x: 0, y: 8, z: 11 };
/** The 16:9 view's half-width over its distance (fov 55°). */
const HALF_WIDTH = Math.tan((55 / 2) * (Math.PI / 180)) * (16 / 9);
/** How much of that width a narrower viewport keeps: the loot nearest the camera stays on the screen and inside its labels' 30 m reach. */
const KEEP = 0.85;

/**
 * The camera framed for the viewport, as a game frames a portrait phone: the
 * field keeps the width its units stand in (those at its sides stay on the
 * screen, with their tags), the field of view widening to 80° and the camera
 * drawing back for the rest.
 */
function frame(world: World): void {
  const cam = must(world.resource(og.Camera), "camera");
  const vp = og.ui.viewport(world);
  const need = Math.max(Math.tan((55 / 2) * (Math.PI / 180)), (HALF_WIDTH * KEEP) / (vp.width / vp.height));
  const fov = Math.min(80, (2 * Math.atan(need) * 180) / Math.PI);
  const back = need / Math.tan((fov / 2) * (Math.PI / 180));
  cam.position = { x: TARGET.x + (FROM.x - TARGET.x) * back, y: TARGET.y + (FROM.y - TARGET.y) * back, z: TARGET.z + (FROM.z - TARGET.z) * back };
  cam.quaternion = og.transform.lookAt(cam.position, TARGET);
  cam.fov = fov;
}

const Framing = defineSystem("Framing", { reads: [], writes: [og.Camera] }, (_q, _time, world) => frame(world));

/** Each fixture as a store of its scalars (`<member>_<state>`) and a keyed collection per list (`<member>_<state>_<prop>`). */
function stores(world: World): Record<string, UiReadable<unknown>> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, UiReadable<unknown>> = { page: scope.writable("page", "field") };
  for (const [member, states] of Object.entries(FIXTURES)) {
    for (const [state, fixture] of Object.entries(states)) {
      const name = `${member}_${state}`;
      out[name] = scope.writable(name, fixture);
      for (const [prop, value] of Object.entries(fixture)) {
        // The hits start empty: `hits` sends them, so each number rises from then.
        const rows = member === "damageNumbers" ? [] : (value as { id: string }[]);
        if (Array.isArray(value)) out[`${name}_${prop}`] = scope.collection(`${name}_${prop}`, rows, { key: "id" });
      }
    }
  }
  return out;
}

/** How many times `hits` has sent the hits again: each volley's rows are new. */
let volley = 0;

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  must(scope.mount("sheet", "WorldSheet", stores(world)), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  systems: [Framing],
  tick: 60,
  seed: 1,
  start(world) {
    scene(world);
    // The prompt's action, defined as a game defines it, so its key shows.
    og.input.defineAction(world, { name: "interact", kind: "button", bindings: [{ control: "KeyE" }, { control: "pad.x" }] });
    setStyle(world, "fantasy");
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "field").set(PAGES.includes(args[0] as never) ? args[0]! : "field");
    },
    hits: (world) => {
      const scope = og.ui.scope(world, "sheet");
      volley++;
      for (const [state, fixture] of Object.entries(FIXTURES.damageNumbers!)) {
        const rows = fixture.hits as Array<{ id: string }>;
        const hits = scope.collection(`damageNumbers_${state}_hits`, [] as Array<{ id: string }>, { key: "id" });
        for (const key of hits.keys()) hits.remove(key);
        for (const row of rows) hits.insert({ ...row, id: `${row.id}.${volley}` });
      }
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: `A review page of the world-anchored components (${Object.values(NAMES).length} members).` },
    hits: { usage: "", doc: "Every damage number's hit again, so they rise from now." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
