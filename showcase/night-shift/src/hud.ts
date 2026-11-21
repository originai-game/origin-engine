// The courier's HUD (assets/ui/hud.tsx): the engine's minimap, timer,
// tracker, bar and action hints, fed from the world each tick. H hides it.
import { defineResource, defineSystem, f64, og } from "@origingame/engine";
import type { Texture, World } from "@origingame/engine";
import { Pedestrian } from "./cast.ts";
import { must } from "./util.ts";
import { mount, useStore } from "@origingame/engine/ui";
import { Hud as HudView } from "../assets/ui/hud.tsx";

/** Seconds to make the drop, and where it is: under the gate. */
const DELIVERY = 180;
const DROP = { x: 0, z: -24 };
/** The map's extent (tools/draw-map.sh): x -25 to 25 across, z -40 to 10 down. */
const onMap = (x: number, z: number): { x: number; y: number } => ({ x: (x + 25) / 50, y: (z + 40) / 50 });

interface Marker { readonly id: string; readonly x: number; readonly y: number; readonly tone: string; readonly player: boolean }
const HINTS = [
  { action: "weather", hint: "Weather", hold: false },
  { action: "season", hint: "Season", hold: false },
  { action: "time", hint: "Night / dawn", hold: false },
  { action: "hud", hint: "HUD", hold: false },
];
const STEPS = [
  { id: "collect", text: "Collect at Lantern Row", done: true, tag: "" },
  { id: "deliver", text: "Deliver to Gate Nine", done: false, tag: "" },
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

/** Loads the HUD's map; called from start. */
export function loadHud(world: World): void {
  og.ui.configure(world, {
    font: must(og.load.font(world, "fonts/latin/NotoSans-Latin.ttf")),
  });
  must(world.resource(HudAssets)).map = must(og.load.texture(world, "textures/map.png")) as number;
}

/** Registers the HUD's stores and mounts it (GameDesc.ui); the TSX follows the stores. */
export function mountHud(world: World): void {
  const scope = og.ui.scope(world, "hud");
  const stores = {
    shown: scope.writable("shown", true),
    seconds: scope.writable("seconds", DELIVERY),
    distance: scope.writable("distance", ""),
    markers: scope.collection("markers", markers(world), { key: "id" }),
  };
  const map = must(world.resource(HudAssets)).map as Texture;
  mount(world, () => {
    const shown = useStore(stores.shown), seconds = useStore(stores.seconds), distance = useStore(stores.distance), marks = useStore(stores.markers);
    return HudView({
      get shown() { return shown(); },
      get seconds() { return seconds(); },
      total: DELIVERY,
      parcel: "Parcel 7C · Fragile",
      steps: STEPS,
      get distance() { return distance(); },
      condition: 96,
      map,
      get markers() { return marks(); },
      district: "Lantern Row",
      hints: HINTS,
    });
  }, {}, { layer: "hud" });
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
