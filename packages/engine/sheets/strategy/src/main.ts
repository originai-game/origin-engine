// The strategy components' review sheet (docs/ui.md §6.2): the field page
// (the RTS HUD laid out as its approved mockup over sample battlefield art),
// the defence page (an endless wave, an overflowing queue and a high-tier
// tower over the same scene) and the states page (every member in every
// fixture state it declares), dressed by a style pack or the default style. The pages use the built-in components by name, as a game
// does, with fixture stores.
//
// Commands: style:<fantasy|default>, page:<field|defence|states>.

import { og, type UiReadable, type World } from "@origingame/engine";
import { FIXTURES, NAMES } from "./fixtures.ts";

const PAGES = ["field", "defence", "states"] as const;
const STYLES = ["fantasy", "scifi", "cartoon", "default"] as const;

const DOCUMENTS = ["ui/field.svelte", "ui/defence.svelte", "ui/states.svelte", "ui/sheet.svelte"];

/** The command card's actions, so its key caps show the bound key's glyph as a game's would. */
const ACTIONS: [string, string, string][] = [
  ["cmdMove", "KeyM", "pad.a"], ["cmdStop", "KeyS", "pad.b"], ["cmdHold", "KeyH", "pad.x"], ["cmdAttack", "KeyA", "pad.y"],
];

/** The field page's sample art under assets/art (hud-<name>.jpg; sheet only, never installed), loaded in start. */
const HUD_ART = ["field", "map", ...[0, 1, 2, 3, 4, 5, 6, 7].map((i) => `rts-${i}`), ...[0, 1, 2, 3, 4, 5].map((i) => `res-${i}`)];
let hudArt: Record<string, number> = {};

/** The field page's members: each default fixture with the sample art in place of the atlas icons, as a game's stores would carry its textures. */
function fieldFixtures(): Record<string, Record<string, unknown>> {
  const art = (name: string) => hudArt[name] ?? 0;
  const res: Record<string, string> = { gold: "res-0", wood: "res-1", food: "res-2", crystal: "res-3", pop: "res-4" };
  const cmd: Record<string, string> = { attack: "rts-4", hold: "rts-5", rally: "rts-7", upgrade: "rts-6", barracks: "rts-0", tower: "rts-3", archer: "rts-2" };
  const rows = (member: string, prop: string) => FIXTURES[member]!.default![prop] as Record<string, unknown>[];
  const d = (member: string) => FIXTURES[member]!.default!;
  const unit = (id: string) => art(id.startsWith("a") ? "rts-2" : "rts-1");
  return {
    resources: { ...d("resources"), resources: rows("resources", "resources").map((r) => ({ ...r, icon: art(res[r.id as string] ?? "") })) },
    clock: d("clock"),
    wave: { ...d("wave"), icon: art("res-5") },
    alerts: d("alerts"),
    groups: d("groups"),
    minimap: { ...d("minimap"), map: art("map") },
    selection: { ...d("selection"), portrait: art("rts-1"), units: rows("selection", "units").map((u) => ({ ...u, portrait: unit(u.id as string) })) },
    queue: { ...d("queue"), items: rows("queue", "items").map((q) => ({ ...q, picture: unit(q.id === "o1" ? "m" : "a") })) },
    commands: { ...d("commands"), commands: rows("commands", "commands").map((c) => ({ ...c, icon: art(cmd[c.id as string] ?? ""), costIcon: c.cost !== "" ? art(c.id === "tower" ? "res-3" : "res-0") : 0 })) },
  };
}

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

/** The style and the fallback icons in one configure, so neither resets the other. */
function setStyle(world: World, name: string): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  const fallbackIcons = { texture: ink, columns: 8, rows: 4, names: INK_NAMES };
  if (name === "default") {
    og.ui.configure(world, { style: null, font: null, fallbackIcons });
  } else og.ui.configure(world, { style: must(og.load.style(world, `@styles/${name}`), name), font: null, fallbackIcons });
}

/** Each fixture as a store of its scalars (`<member>_<state>`) and a keyed collection per list (`<member>_<state>_<prop>`). */
function stores(world: World): Record<string, UiReadable<unknown>> {
  const scope = og.ui.scope(world, "sheet");
  const out: Record<string, UiReadable<unknown>> = { page: scope.writable("page", "field"), hudField: scope.writable("hudField", hudArt.field ?? 0) };
  for (const [member, fixture] of Object.entries(fieldFixtures())) {
    const name = `field_${member}`;
    out[name] = scope.writable(name, fixture);
    for (const [prop, value] of Object.entries(fixture)) {
      if (Array.isArray(value)) out[`${name}_${prop}`] = scope.collection(`${name}_${prop}`, value as { id: string }[], { key: "id" });
    }
  }
  const captions: Record<string, string> = {};
  for (const [member, states] of Object.entries(FIXTURES)) {
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
  must(scope.mount("sheet", "StrategySheet", stores(world)), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.05, g: 0.05, b: 0.06 };
    for (const [name, key, pad] of ACTIONS) og.input.defineAction(world, { name, kind: "button", bindings: [{ control: key }, { control: pad }] });
    setStyle(world, "fantasy");
    hudArt = Object.fromEntries(HUD_ART.map((p) => [p, must(og.load.texture(world, `art/hud-${p}.jpg`), p) as number]));
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world, args) => setStyle(world, STYLES.includes(args[0] as never) ? args[0]! : "fantasy"),
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "light" ? "light" : "dark" }),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "field").set(PAGES.includes(args[0] as never) ? args[0]! : "field");
    },
  },
  commandHelp: {
    style: { usage: `<${STYLES.join("|")}>`, doc: "The style the pages are dressed in." },
    scheme: { usage: "<light|dark>", doc: "The colour scheme the style is drawn in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the strategy kit." },
  },
});
