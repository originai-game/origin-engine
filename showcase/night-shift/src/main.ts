// Night Shift: a courier and a hover-bike at the mouth of a market alley.
// The set is built once (set.ts); this file sets the night, the fog, the
// rain and the camera over it.
import { og, WEATHER_RAIN } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { buildSet } from "./set.ts";
import { must, rgb, v3 } from "./util.ts";

function start(world: World): void {
  buildSet(world);

  // Night over the city: a violet overcast lit from below, no sun.
  const sky = must(world.resource(og.Sky));
  sky.enabled = true;
  sky.zenith = rgb(0.035, 0.022, 0.075);
  sky.horizon = rgb(0.2, 0.08, 0.26);
  sky.ground = rgb(0.01, 0.008, 0.015);
  sky.sunSize = 0;
  sky.glow = 0;
  sky.brightness = 1;
  const moon = must(world.spawn({ name: "moon", quaternion: og.transform.lookAt(v3(0, 0, 0), v3(-0.3, -0.8, -0.5)) }));
  must(world.insert(moon, og.DirectionalLight, { color: rgb(0.55, 0.6, 1), intensity: 0.08, castShadow: true }));

  const fog = must(world.resource(og.Fog));
  fog.enabled = true;
  fog.color = rgb(0.16, 0.075, 0.24);
  fog.density = 0.009;
  fog.heightDensity = 0.015;
  fog.height = 2;
  fog.falloff = 0.08;
  fog.volumetric = true;
  fog.volumetricDistance = 60;
  fog.scattering = 0.6;

  const weather = must(world.resource(og.Weather));
  weather.kind = WEATHER_RAIN;
  weather.intensity = 1;
  weather.radius = 22;
  const ground = must(world.resource(og.WeatherState));
  ground.soakTime = 0.5;
  ground.puddleTime = 1;
  const wind = must(world.resource(og.Wind));
  wind.direction = v3(0.4, 0, -1);
  wind.speed = 3;

  const cam = must(world.resource(og.Camera));
  cam.position = v3(0.7, 0.32, 7.2);
  cam.quaternion = og.transform.lookAt(cam.position, v3(0.5, 4.6, -12));
  cam.fov = 62;
  cam.far = 600;
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/night-shift.ogt",
  start,
});
