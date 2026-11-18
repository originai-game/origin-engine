// The world labels' review sheet (docs/ui.md §6.2): the nameplates, overhead
// bars, loot, prompts, speech and waypoints a game anchors over its scene
// and the telegraphs, rings, placement ghosts and paths it lays on its
// ground, over the approved mockup's sample art: `scene` (the third-person
// scene), `topdown` (the strategy scene), `states` (every label in every
// state) and `marks` (every ground mark, bubble and bar in every state).
//
// The camera looks down over an empty ground; the art is an unlit plane far
// behind it, facing the camera and covering the screen. Each label stands
// at the point a fixed depth along the ray through its pixel in the
// mockup's frame, each ground mark where that ray meets the ground, so the
// labels sit on the art where the mockup drew them (src/fixtures.ts).
//
// Commands: page:<scene|topdown|states|marks>, style:default, reduced:<on|off>.

import { og, defineSystem, type World } from "@origingame/engine";
import { LAYOUTS, STATES, type Placed } from "./fixtures.ts";

const PAGES = ["scene", "topdown", "states", "marks"] as const;
const KINDS = ["Nameplate", "LootLabel", "InteractionPrompt", "SpeechBubble", "Telegraph", "SelectionRing", "PlacementGhost", "Waypoint", "PathLine"];
const GROUNDS = new Set(["Telegraph", "SelectionRing", "PlacementGhost", "PathLine"]);
const DOCUMENTS = ["ui/sheet.svelte"];
/** The camera: its height over the ground, its vertical field of view; the art's and the labels' depths. */
const HEIGHT = 10, FOV = 50, BACK = 60, LABEL = 12;
/** How far the camera looks down on each page, degrees: the mockup's ground marks lie at its tilt (the scene's telegraph squashed to about 0.3, the top-down and the boards' to about 0.5). */
const TILTS: Record<string, number> = { scene: 12, topdown: 30, states: 30, marks: 30 };
/** The art each page stands over. */
const ART: Record<string, string> = { scene: "scene", topdown: "topdown", states: "scene", marks: "topdown" };

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

function setStyle(world: World): void {
  const ink = must(og.load.texture(world, "icons/ink-icons.png"), "ink icons");
  og.ui.configure(world, { style: null, font: null, fallbackIcons: { texture: ink, columns: 8, rows: 4, names: INK_NAMES } });
}

type V3 = { x: number; y: number; z: number };
const EYE: V3 = { x: 0, y: HEIGHT, z: 0 };
let FORWARD: V3 = { x: 0, y: 0, z: -1 };
let UP: V3 = { x: 0, y: 1, z: 0 };
/** Turns the camera's axes to the page's tilt. */
function aim(page: string): void {
  const t = ((TILTS[page] ?? 30) * Math.PI) / 180;
  FORWARD = { x: 0, y: -Math.sin(t), z: -Math.cos(t) };
  UP = { x: 0, y: Math.cos(t), z: -Math.sin(t) };
}
const TAN = Math.tan(((FOV / 2) * Math.PI) / 180);

/** The ray through a viewport pixel, its forward part 1 (a step along it is a step of view depth). */
function ray(px: number, py: number, vw: number, vh: number): V3 {
  const nx = (px / vw) * 2 - 1, ny = 1 - (py / vh) * 2;
  const a = nx * TAN * (vw / vh), b = ny * TAN;
  return { x: FORWARD.x + a, y: FORWARD.y + UP.y * b, z: FORWARD.z + UP.z * b };
}
const along = (d: V3, s: number): V3 => ({ x: EYE.x + d.x * s, y: EYE.y + d.y * s, z: EYE.z + d.z * s });
const text = (p: V3): string => `${p.x.toFixed(3)} ${p.y.toFixed(3)} ${p.z.toFixed(3)}`;

let backdrops: Record<string, number> = {};

/** Points the camera and turns the art planes to face it, at the art's depth. */
function look(world: World): void {
  const cam = must(world.resource(og.Camera), "camera");
  cam.position = EYE;
  cam.quaternion = og.transform.lookAt(EYE, { x: EYE.x + FORWARD.x, y: EYE.y + FORWARD.y, z: EYE.z + FORWARD.z });
  for (const e of Object.values(backdrops)) must(world.set(e as never, og.Transform, { position: along(FORWARD, BACK), quaternion: cam.quaternion }), "backdrop");
}

function scene(world: World): void {
  aim("scene");
  const cam = must(world.resource(og.Camera), "camera");
  cam.position = EYE;
  cam.quaternion = og.transform.lookAt(EYE, { x: EYE.x + FORWARD.x, y: EYE.y + FORWARD.y, z: EYE.z + FORWARD.z });
  cam.fov = FOV;
  cam.background = { r: 0.04, g: 0.05, b: 0.08 };
  const plane = must(og.render.planeGeometry(world, {}), "plane");
  for (const art of ["scene", "topdown"]) {
    const map = must(og.load.texture(world, `art/world-${art}.jpg`), art);
    const material = must(og.render.standardMaterial(world, { color: { r: 1, g: 1, b: 1 }, map, unlit: true, transparent: true }), "material");
    const e = must(world.spawn({ position: along(FORWARD, BACK), quaternion: cam.quaternion }), "spawn");
    must(world.insert(e, og.Mesh, { geometry: plane, material, castShadow: false, receiveShadow: false }), "mesh");
    backdrops[art] = e as unknown as number;
  }
}

/** The layout for the viewport: the page's phone layout on a phone, else its desktop one, and the map from its pixels to the viewport's (the art covers the screen, centred). */
function layout(page: string, vw: number, vh: number): { items: Placed[]; map: (x: number, y: number) => [number, number]; scale: number } {
  const pages = LAYOUTS[page] ?? LAYOUTS.scene!;
  const key = vw < 600 && vh > vw ? "portrait" : vw < 1000 && vw > vh ? "landscape" : "desktop";
  const [lw, lh, items] = pages[key] ?? pages.desktop!;
  const s = Math.max(vw / lw, vh / lh);
  const ox = (lw * s - vw) / 2, oy = (lh * s - vh) / 2;
  return { items, map: (x, y) => [x * s - ox, y * s - oy], scale: s };
}

/** Every kind's rows for the page at this viewport. */
function rows(page: string, vw: number, vh: number): Record<string, Record<string, unknown>[]> {
  const out: Record<string, Record<string, unknown>[]> = Object.fromEntries([...KINDS, "bars"].map((k) => [k, []]));
  const { items, map, scale } = layout(page, vw, vh);
  items.forEach((p, i) => {
    const [px, py] = map(p.x, p.y);
    const d = ray(px, py, vw, vh);
    const id = p.id ?? `${p.kind}-${p.state}-${i}`;
    if (p.kind === "OverheadBars") {
      out.bars!.push({ id, entity: 0, at: text(along(d, LABEL)), value: 100, max: 100, side: p.state, resource: 0, resourceMax: 0, ...p.over });
      return;
    }
    const row: Record<string, unknown> = { ...STATES[p.kind]![p.state], ...p.over, id, entity: 0 };
    if (GROUNDS.has(p.kind)) {
      const s = -EYE.y / d.y;
      row.at = text(along(d, s));
      // The mark's width in world units at the depth it lies.
      const w = ((p.w ?? 100) * scale * 2 * s * TAN) / vh;
      if (p.kind === "Telegraph" || p.kind === "SelectionRing") row.radius = w / 2;
      if (p.kind === "PlacementGhost") { row.width = w; row.depth = w; }
      if (p.kind === "PathLine") row.size = w;
    } else {
      row.at = text(along(d, LABEL));
      row.height = 0;
      if ("maxDistance" in row) row.maxDistance = 80;
    }
    out[p.kind]!.push(row);
  });
  return out;
}

/** The page and viewport laid out last: the rows are laid again when either changes. */
let laid = "";

function relay(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const page = scope.get(scope.writable("page", "scene")) as string;
  const vp = og.ui.viewport(world);
  const key = `${page}:${vp.width}x${vp.height}`;
  if (key === laid) return;
  laid = key;
  aim(page);
  look(world);
  for (const [art, e] of Object.entries(backdrops)) {
    const hh = BACK * TAN, aspect = vp.width / vp.height;
    const [w, h] = aspect >= 16 / 9 ? [2 * hh * aspect, (2 * hh * aspect * 9) / 16] : [((2 * hh) * 16) / 9, 2 * hh];
    must(world.set(e as never, og.Transform, { scale: { x: w, y: h, z: 1 } }), "scale");
    must(world.set(e as never, og.Mesh, { visible: ART[page] === art }), "visible");
  }
  for (const [kind, list] of Object.entries(rows(page, vp.width, vp.height))) {
    const c = scope.collection(kind, [] as Array<Record<string, unknown>>, { key: "id" });
    for (const k of c.keys()) c.remove(k);
    for (const row of list) c.insert(row as never);
  }
}

const Layout = defineSystem("Layout", { reads: [og.Transform, og.Mesh, og.Camera], writes: [og.Transform, og.Mesh, og.Camera] }, (_q, _t, world) => relay(world));

function show(world: World): void {
  const scope = og.ui.scope(world, "sheet");
  const inputs = Object.fromEntries([...KINDS, "bars"].map((k) => [k, scope.collection(k, [] as Array<{ id: string }>, { key: "id" })]));
  must(scope.mount("sheet", "WorldSheet", { page: scope.writable("page", "scene"), ...inputs }), "mount");
  const errors = scope.takeErrors();
  if (errors.length) throw new Error(errors.join("; "));
}

export default og.game({
  systems: [Layout],
  tick: 60,
  seed: 1,
  start(world) {
    scene(world);
    // The prompt's and the loot's actions, defined as a game defines them, so their keys show.
    og.input.defineAction(world, { name: "interact", kind: "button", bindings: [{ control: "KeyE" }, { control: "pad.x" }] });
    og.input.defineAction(world, { name: "pick", kind: "button", bindings: [{ control: "KeyZ" }, { control: "pad.a" }] });
    setStyle(world);
    for (const path of DOCUMENTS) must(og.load.document(world, path), path);
  },
  ui: show,
  commands: {
    style: (world) => setStyle(world),
    page: (world, args) => {
      og.ui.scope(world, "sheet").writable("page", "scene").set(PAGES.includes(args[0] as never) ? args[0]! : "scene");
    },
    reduced: (world, args) => og.ui.configure(world, { reducedMotion: args[0] !== "off" }),
  },
  commandHelp: {
    style: { usage: "<default>", doc: "The default style the pages are dressed in." },
    page: { usage: `<${PAGES.join("|")}>`, doc: "A review page of the world labels." },
    reduced: { usage: "<on|off>", doc: "Reduced motion on or off." },
  },
});
