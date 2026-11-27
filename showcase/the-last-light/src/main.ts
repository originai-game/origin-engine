// The Last Light: a wanderer on a cliff path, a lighthouse on its headland,
// the sun going down into a swelling sea. The player walks the wanderer
// (player.ts) and turns the weather, the season and the time (climate.ts).
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Climate, Seasons, buildClimate, climateCommands, climateHelp } from "./climate.ts";
import { buildCoast } from "./coast.ts";
import { Beam, buildLight } from "./lighthouse.ts";
import { buildSky } from "./sky.ts";
import { buildWanderer, Clips } from "./wanderer.ts";
import { buildPlayer, Player, Walk } from "./player.ts";
import { buildQuest, mountHud, Play, Quest, questCommands } from "./quest.ts";
import { CAMERA } from "./layout.ts";
import { must } from "./must.ts";
import { Burn, loadReel, mountReel, ReelAssets, reelCommands } from "./reel.ts";

function build(world: World): void {
  const coast = buildCoast(world);
  buildWanderer(world, coast);
  buildLight(world, coast);
  buildSky(world);
  buildClimate(world);
  buildPlayer(world);
  buildQuest(world);
  loadReel(world);

  const cam = must(world.resource(og.Camera));
  cam.fov = CAMERA.fov;
  cam.far = 6000;
}

export default og.game({ start: build, resources: [Climate, Player, Quest, Clips, ReelAssets], systems: [Walk, Play, Seasons, Beam, Burn], ui: (world) => { mountHud(world); mountReel(world); }, commands: { ...climateCommands, ...questCommands, ...reelCommands }, commandHelp: climateHelp });
