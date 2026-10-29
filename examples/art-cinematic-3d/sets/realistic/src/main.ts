// The cinematic 3D family's realistic set (sets/realistic/): the shared
// beats (src/scene.ts) with scanned and generated models, under a low sun
// behind the scene (golden hour, the light the realistic styles' previews
// grade), the street lamp's light at its glass and the hand lantern's on
// the crate. The look still decides the grade, fog and lenses.
import { og } from "@origingame/engine";
import { build } from "./scene.ts";

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    build(world, {
      eye: { x: 2.2, y: 1.55, z: 6.6 },
      at: { x: -0.3, y: 1.15, z: -1.2 },
      fov: 42,
      sunDirection: { x: -0.55, y: -0.17, z: 0.5 },
      sunIntensity: 4,
      lanternAt: { x: 2.9, y: 2.75, z: -1.2 },
      lanternIntensity: 5,
      lights: [{ x: 1.15, y: 0.75, z: 0.25, intensity: 2.5, distance: 3 }],
      // Dusk: a warm horizon under a cooler, darker zenith, the sun low on the right behind the scene.
      sky: { zenith: { r: 0.16, g: 0.22, b: 0.36 }, horizon: { r: 0.95, g: 0.62, b: 0.36 }, sunColor: { r: 1, g: 0.72, b: 0.45 }, brightness: 1 },
    });
  },
});
