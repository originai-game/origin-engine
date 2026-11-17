// The light: the lamp in its room and the beam it sweeps through the sea
// mist, once round every twelve seconds.
import { defineSystem, og, quat } from "@origingame/engine";
import type { World } from "@origingame/engine";
import type { Coast } from "./coast.ts";
import { must } from "./must.ts";

/** Seconds for the beam to go once round. */
const PERIOD = 12;
/** Where the beam points at tick 0; five seconds on it lies out over the sea, to the right of the cover. */
const START = 0.64;
/** How far below level the beam dips. */
const DIP = 0.03;

/** How far along the beam its haze reaches, and how wide it is. */
const REACH = 170, WIDTH = 16;

const beamAt = (seconds: number) => quat.multiply(
  og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, START - (seconds / PERIOD) * 2 * Math.PI),
  og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -DIP),
);

/** Turns the beam and the haze it shows in, which lies along it from the lamp. */
function aim(world: World, seconds: number): void {
  const beam = world.find("beam"), haze = world.find("beam-haze");
  if (beam === null || haze === null) return;
  const quaternion = beamAt(seconds);
  must(world.set(beam, og.Transform, { quaternion }));
  const lamp = must(world.get(beam, og.Transform)).position, along = og.transform.cameraForward(quaternion), half = REACH / 2 + 4;
  must(world.set(haze, og.Transform, { quaternion, position: { x: lamp.x + along.x * half, y: lamp.y + along.y * half, z: lamp.z + along.z * half } }));
}

export function buildLight(world: World, coast: Coast): void {
  const lamp = must(world.spawn({ name: "lamp", position: coast.lamp }));
  must(world.insert(lamp, og.Mesh, {
    geometry: must(og.render.sphereGeometry(world, { radius: 0.55 })),
    material: must(og.render.standardMaterial(world, { color: { r: 1, g: 0.85, b: 0.55 }, emissive: { r: 1, g: 0.78, b: 0.42 }, emissiveIntensity: 30 })),
  }));
  must(world.insert(lamp, og.PointLight, { color: { r: 1, g: 0.8, b: 0.5 }, intensity: 40, distance: 30 }));
  // A spot light looks along its -Z.
  const beam = must(world.spawn({ name: "beam", position: coast.lamp }));
  must(world.insert(beam, og.SpotLight, { color: { r: 1, g: 0.86, b: 0.6 }, intensity: 120000, distance: 420, angle: 0.06, penumbra: 0.5 }));
  // The haze the beam shows in: a box along the beam alone, so the air
  // beside it stays as the weather has it.
  const haze = must(world.spawn({ name: "beam-haze", position: coast.lamp }));
  must(world.insert(haze, og.Volume, { size: { x: WIDTH, y: WIDTH, z: REACH }, fade: 5 }));
  must(world.insert(haze, og.FogVolume, { color: { r: 1, g: 0.95, b: 0.88 }, density: 0.008, scattering: 1 }));
  aim(world, 0);
}

export const Beam = defineSystem("Beam", { writes: [og.Transform] }, (_q, time, world) => aim(world, time.tick * time.dt));
