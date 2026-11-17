// Night Shift: a courier and a hover-bike at the mouth of a market alley.
// The set and the cast are built once (set.ts, cast.ts); air.ts runs the
// weather, the season and the time of day over them.
import { defineSystem, og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Air, AirSystem, buildAir, SEASONS, setSeason, setTime, setWeather, TIMES, WEATHERS } from "./air.ts";
import { buildCast, Stand, Walk } from "./cast.ts";
import { buildFlora, Flora } from "./flora.ts";
import { Hud, HudAssets, loadHud, mountHud, toggleHud } from "./hud.ts";
import { buildSet } from "./set.ts";
import { must, v3 } from "./util.ts";

function start(world: World): void {
  buildSet(world);
  buildCast(world);
  buildFlora(world);
  buildAir(world);

  loadHud(world);
  // The switches: each key steps its setting to the next, which ramps in.
  for (const [name, key, pad] of [["weather", "KeyR", "pad.x"], ["season", "KeyF", "pad.y"], ["time", "KeyT", "pad.b"], ["hud", "KeyH", "pad.back"]] as const) {
    must(og.input.defineAction(world, { name, kind: "button", bindings: [{ control: key }, { control: pad }] }));
  }

  const cam = must(world.resource(og.Camera));
  // Low, near the street, pitched up: the wet setts fill the lower third
  // and the towers stand over the gate.
  cam.position = v3(0, 0.7, 7);
  cam.quaternion = og.transform.lookAt(cam.position, v3(0, 5.2, -13));
  cam.fov = 62;
  cam.far = 600;
}

/** Steps the weather, the season and the time of day on their keys, and hides the HUD on its. */
const Switches = defineSystem("Switches", { writes: [Air] }, (_q, _time, world) => {
  const air = must(world.resource(Air));
  if (og.input.isActionJustPressed(world, "weather")) air.weather = (air.weather + 1) % WEATHERS.length;
  if (og.input.isActionJustPressed(world, "season")) air.season = (air.season + 1) % SEASONS.length;
  if (og.input.isActionJustPressed(world, "time")) air.time = (air.time + 1) % TIMES.length;
  if (og.input.isActionJustPressed(world, "hud")) toggleHud(world);
});

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/night-shift.ogt",
  resources: [Air, HudAssets],
  systems: [Switches, AirSystem, Flora, Walk, Stand, Hud],
  ui: mountHud,
  commands: {
    weather: (world, args) => setWeather(world, args[0]),
    season: (world, args) => setSeason(world, args[0]),
    time: (world, args) => setTime(world, args[0]),
    hud: (world) => toggleHud(world),
  },
  commandHelp: {
    weather: { usage: "<rain|storm|fog|clear|snow>", doc: "Switch the weather; it ramps over a few seconds." },
    season: { usage: "<spring|summer|autumn|winter>", doc: "Turn the season." },
    time: { usage: "<night|dawn>", doc: "Break the dawn, or go back to night." },
    hud: { usage: "", doc: "Show or hide the HUD." },
  },
  start,
});
