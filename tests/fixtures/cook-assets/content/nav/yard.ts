import { file } from "@origingame/engine/author";

// Ground and wall in local metres. Scene/model/terrain refs can supply
// geometry instead; this declaration loads as base/nav/yard.
export default file.navmesh({
  agents: [{ radius: 0.4, height: 1.8, climb: 0.4, slope: 45 }],
  cellSize: 0.25,
  tileSize: 32,
  bounds: { min: [-12,-1,-12], max: [12,3,12] },
  boxes: [
    { center: [0,-0.1,0], size: [30,0.2,30] },
    { center: [-3,1,-2], size: [0.6,2,6] },
  ],
});
