// What the season shows: the cherry over the shophouse and what it sheds.
// Blossom and falling petals in spring, green in summer, amber leaves
// coming down in autumn, bare wood in winter. The Flora system follows the
// amounts in the air (air.ts), so the tree turns as the season ramps.
//
// The tree is a `plant` model, which the engine's seasons will drive
// through its leaves (the seasons and weather proposal, step 3); until
// then the crown is tinted here and gives way to the bare wood in winter.
import { defineSystem, og } from "@origingame/engine";
import type { Color, World } from "@origingame/engine";
import { Air } from "./air.ts";
import { must, rgb, smooth, v3, yaw } from "./util.ts";

const BLOSSOM = rgb(1, 0.5, 0.72);
const GREEN = rgb(0.2, 0.5, 0.16);
const AMBER = rgb(0.95, 0.4, 0.08);
const WITHERED = rgb(0.1, 0.06, 0.04);
/** Where the tree stands: beside the shophouse, its crown over the bike. */
const TREE = v3(4.6, 0, 0.6);

export function buildFlora(world: World): void {
  const at = { position: TREE, quaternion: yaw(40) };
  const crown = must(world.instantiate(must(og.load.gltf(world, "models/cherry.ogt")), { name: "cherry", ...at }));
  must(world.set(crown, og.Model, { sway: 0.6, tint: BLOSSOM }));
  const bare = must(world.instantiate(must(og.load.gltf(world, "models/cherry-bare.ogt")), { name: "cherry-bare", ...at }));
  must(world.set(bare, og.Model, { sway: 0.6, visible: false }));

  // What the crown sheds, born through it: petals now, leaves when autumn comes.
  const shed = v3(TREE.x - 0.6, 4.6, TREE.z + 0.6);
  const petals = must(world.spawn({ name: "petals", position: shed }));
  must(world.insert(petals, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/petals.ogt")), seed: 3 }));
  const leaves = must(world.spawn({ name: "leaves", position: shed }));
  must(world.insert(leaves, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/leaves.ogt")), seed: 5, stopTick: 1 }));
}

/** Starts or stops a looping emitter; the particles in the air live out. */
function shed(world: World, name: string, on: boolean, tick: number): void {
  const e = world.find(name);
  if (e === null) return;
  const stopped = must(world.get(e, og.ParticleSystem)).stopTick !== 0;
  if (on && stopped) must(world.set(e, og.ParticleSystem, { startTick: tick, stopTick: 0 }));
  if (!on && !stopped) must(world.set(e, og.ParticleSystem, { stopTick: tick }));
}

export const Flora = defineSystem("Flora", { reads: [Air], writes: [og.Model, og.ParticleSystem], after: ["Air"] }, (_q, time, world) => {
  const air = must(world.resource(Air));
  const parts: readonly (readonly [number, Color])[] = [[air.spring, BLOSSOM], [air.summer, GREEN], [air.autumn, AMBER], [air.winter, WITHERED]];
  const total = Math.max(parts.reduce((sum, [k]) => sum + k, 0), 1e-6);
  let tint = rgb(0, 0, 0);
  for (const [k, c] of parts) tint = rgb(tint.r + c.r * k / total, tint.g + c.g * k / total, tint.b + c.b * k / total);
  // The last leaves are down once winter is nearly in.
  const leafless = smooth(air.winter) > 0.9;
  const crown = world.find("cherry"), bare = world.find("cherry-bare");
  if (crown !== null) must(world.set(crown, og.Model, { tint, visible: !leafless }));
  if (bare !== null) must(world.set(bare, og.Model, { visible: leafless }));
  shed(world, "petals", air.season === 0, time.tick);
  shed(world, "leaves", air.season === 2, time.tick);
});
