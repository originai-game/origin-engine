// The Last Light: a wanderer on a cliff path, a lighthouse on its headland,
// the sun going down into a swelling sea.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Climate, Seasons, buildClimate, climateCommands, climateHelp } from "./climate.ts";
import { buildCoast } from "./coast.ts";
import { Beam, buildLight } from "./lighthouse.ts";
import { buildSky } from "./sky.ts";
import { buildWanderer } from "./wanderer.ts";
import { CAMERA } from "./layout.ts";
import { must } from "./must.ts";

function build(world: World): void {
  const coast = buildCoast(world);
  buildWanderer(world, coast);
  buildLight(world, coast);
  buildSky(world);
  buildClimate(world);

  const cam = must(world.resource(og.Camera));
  cam.position = { x: CAMERA.x, y: coast.heightAt(CAMERA.x, CAMERA.z) + CAMERA.eye, z: CAMERA.z };
  cam.quaternion = og.transform.lookAt(cam.position, { x: CAMERA.lookX, y: cam.position.y - CAMERA.drop, z: CAMERA.lookZ });
  cam.fov = CAMERA.fov;
  cam.far = 6000;
}

export default og.game({ start: build, resources: [Climate], systems: [Seasons, Beam], commands: climateCommands, commandHelp: climateHelp });
