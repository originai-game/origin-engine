// The courier's HUD (assets/ui/hud.svelte): the engine's minimap, timer,
// tracker, bar and action hints, fed from the world each tick. H hides it.
import { defineResource, defineSystem, f64, og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Pedestrian } from "./cast.ts";
import { must } from "./util.ts";

/** Seconds to make the drop, and where it is: under the gate. */
const DELIVERY = 180;
const DROP = { x: 0, z: -24 };
/** The map's extent (tools/draw-map.sh): x -25 to 25 across, z -40 to 10 down. */
const onMap = (x: number, z: number): { x: number; y: number } => ({ x: (x + 25) / 50, y: (z + 40) / 50 });

/** The engine's shared ink icons, eight a row, in the atlas's order. */
const INK_ICONS = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

interface Marker { readonly id: string; readonly x: number; readonly y: number; readonly tone: string; readonly player: boolean }
const HINTS = [
  { action: "weather", hint: "Weather", hold: false },
  { action: "season", hint: "Season", hold: false },
  { action: "time", hint: "Night / dawn", hold: false },
  { action: "hud", hint: "HUD", hold: false },
];
const STEPS = [
  { id: "collect", text: "Collect at Lantern Row", done: true },
  { id: "deliver", text: "Deliver to Gate Nine", done: false },
];

function markers(world: World): Marker[] {
  const out: Marker[] = [];
  const at = (name: string): { x: number; z: number } | null => {
    const e = world.find(name);
    return e === null ? null : must(world.get(e, og.Transform)).position;
  };
  for (let i = 0; i < 8; i++) {
    const p = at(`pedestrian-${i}`);
    if (p !== null) out.push({ id: `ped-${i}`, ...onMap(p.x, p.z), tone: "neutral", player: false });
  }
  const bike = at("bike");
  if (bike !== null) out.push({ id: "bike", ...onMap(bike.x, bike.z), tone: "info", player: false });
  out.push({ id: "drop", ...onMap(DROP.x, DROP.z), tone: "accent", player: false });
  const me = at("courier");
  if (me !== null) out.push({ id: "me", ...onMap(me.x, me.z), tone: "neutral", player: true });
  return out;
}

/** What the HUD draws with, loaded in start (GameDesc.ui only reads the world): the map's texture. */
export const HudAssets = defineResource("HudAssets", { map: f64() });

/** Loads the HUD's document and map; called from start. */
export function loadHud(world: World): void {
  og.ui.configure(world, {
    font: must(og.load.font(world, "fonts/latin/NotoSans-Latin.ttf")),
    fallbackIcons: { texture: must(og.load.texture(world, "ui/icons/ink-icons.png")), columns: 8, rows: 4, names: INK_ICONS },
  });
  must(og.load.document(world, "ui/hud.svelte"));
  must(world.resource(HudAssets)).map = must(og.load.texture(world, "textures/map.png")) as number;
}

/** Registers the HUD's stores and mounts it (GameDesc.ui). */
export function mountHud(world: World): void {
  const scope = og.ui.scope(world, "hud");
  must(scope.mount("hud", "DeliveryHud", {
    shown: scope.writable("shown", true),
    seconds: scope.writable("seconds", DELIVERY),
    total: scope.writable("total", DELIVERY),
    parcel: scope.writable("parcel", "Parcel 7C · Fragile"),
    steps: scope.collection("steps", STEPS, { key: "id" }),
    distance: scope.writable("distance", ""),
    condition: scope.writable("condition", 96),
    map: scope.writable("map", must(world.resource(HudAssets)).map),
    markers: scope.collection("markers", markers(world), { key: "id" }),
    district: scope.writable("district", "Lantern Row"),
    hints: scope.collection("hints", HINTS, { key: "action" }),
  }));
}

/** Shows or hides the HUD. */
export function toggleHud(world: World): void {
  const shown = og.ui.scope(world, "hud").writable("shown", true);
  must(shown.set(!og.ui.scope(world, "hud").get(shown)));
}

/** The clock, the distance to the drop and the minimap's markers, each tick. */
export const Hud = defineSystem("Hud", { reads: [og.Transform, Pedestrian], after: ["Walk"] }, (_q, time, world) => {
  const scope = og.ui.scope(world, "hud");
  // The clock runs down and starts over: the alley is a standing shot.
  must(scope.writable("seconds", DELIVERY).set(DELIVERY - (time.tick / 60) % DELIVERY));
  const me = world.find("courier");
  if (me !== null) {
    const p = must(world.get(me, og.Transform)).position;
    must(scope.writable("distance", "").set(`${Math.round(Math.sqrt((p.x - DROP.x) ** 2 + (p.z - DROP.z) ** 2))} m`));
  }
  const rows = scope.collection<Marker>("markers", [], { key: "id" });
  must(rows.patchAll(markers(world).map((m) => ({ key: m.id, fields: { x: m.x, y: m.y } }))));
});
