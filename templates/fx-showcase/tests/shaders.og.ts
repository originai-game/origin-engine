// The shaders station: og.load.shader materials. The sphere's dissolve
// amount changes between the two frames (og.render.setShaderParam) and the
// grass sways, while the cut cube stands still.
import { moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("shaders", 1.5, [], ({ t, frame, before }) => {
  t.ok(moved(before, frame, 300, 160, 620, 460) > 1, "the sphere dissolves further");
  t.ok(moved(before, frame, 0, 560, 1280, 720) > 1, "the grass sways");
});
