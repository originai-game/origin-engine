// First Light: a shepherd and a dog under an old oak look over a lake
// village at first light. The player switches weather, season and time of
// day; nothing else is at stake.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Air, Choose, Scene, SEASONS, TIMES, WEATHERS, buildEnv, setScene } from "./env.ts";
import { Hud, mountHud } from "./hud.ts";
import { WOODS, buildLand } from "./land.ts";
import { VIEW, buildSet, inView } from "./set.ts";
import { groundAt, must } from "./util.ts";

function build(world: World): void {
  const land = buildLand(world);
  buildSet(world, land);
  buildEnv(world);

  must(og.input.defineAction(world, { name: "weather", kind: "button", bindings: [{ control: "Digit1" }, { control: "pad.x" }] }));
  must(og.input.defineAction(world, { name: "season", kind: "button", bindings: [{ control: "Digit2" }, { control: "pad.y" }] }));
  must(og.input.defineAction(world, { name: "time", kind: "button", bindings: [{ control: "Digit3" }, { control: "pad.b" }] }));
  must(og.input.defineAction(world, { name: "hud", kind: "button", bindings: [{ control: "KeyH" }, { control: "pad.back" }] }));

  const cam = must(world.resource(og.Camera));
  const eye = { x: VIEW.x, y: groundAt(world, land, VIEW.x, VIEW.z) + VIEW.eye, z: VIEW.z };
  const ahead = inView(0, 100);
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: ahead.x, y: eye.y - 2.5, z: ahead.z });
  cam.fov = 46;
  cam.far = 16000;
}

export default og.game({
  resources: [Scene],
  systems: [Choose, Air, Hud],
  start: build,
  ui: mountHud,
  commands: {
    set_scene: setScene,
    /** `draft[:bare]`: the woods cast no shadows, or with `bare` are not drawn at all, for a quick look on a machine without a GPU. */
    draft: (world, args) => {
      for (const name of WOODS) {
        const wood = world.find(name);
        if (wood === null) continue;
        if (args[0] === "bare") must(world.despawn(wood));
        else must(world.set(wood, og.TerrainScatter, { castShadows: false }));
      }
    },
    /** `set_hud:<on|off>`: a script's H. */
    set_hud: (world, args) => { must(world.resource(Scene)).hud = args[0] === "off" ? 0 : 1; },
  },
  facts: (world) => {
    const s = world.resource(Scene);
    if (!s.ok) return {};
    return { weather: WEATHERS[s.value.weather]!, season: SEASONS[s.value.season]!, time: TIMES[s.value.time]!, hour: s.value.hour, hud: s.value.hud };
  },
});
