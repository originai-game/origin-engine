// The lod station (depth S6; tests/evidence/debt/render-lod): 2,000 trees
// of 3,160 triangles cooked into levels of 3,160, 1,580 and 790. Every
// check is an `expect:` step evaluated in the run (docs/design.md §3.4):
// the nearest tree of the front row (tree-20, about 10 m away) draws
// level 1 and a tree of the back row (tree-1980, about 200 m) level 2,
// and the frame's main pass draws at least 3× fewer triangles than with
// RenderSettings.lodBias 0, which draws every tree whole (lavapipe:
// 1,300,812 against 5,150,788). Without a renderer the levels and the
// triangle count are unchecked.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "The lod station picks coarser levels with distance and draws at least 3x fewer triangles than full detail",
  size: "1280x720",
  commands: [
    "0.5:station:lod",
    "1.5:expect:state:station==\"lod\"",
    "1.5:expect:entity:tree-20:render.lod==1",
    "1.5:expect:entity:tree-1980:render.lod==2",
    // A third of the field at full detail (5,150,788 / 3).
    "1.5:expect:timings:triangles<=1716929",
    "1.5:expect:timings:triangles>=1000000",
    "1.5:screenshot:lod.png",
    // lodBias 0: every tree its full mesh.
    "1.6:set:og.RenderSettings.lodBias=0",
    "2.0:expect:entity:tree-20:render.lod==0",
    "2.0:expect:entity:tree-1980:render.lod==0",
    "2.0:expect:timings:triangles>=5000000",
    "2.0:screenshot:lod-off.png",
    "2.1:dump_render:lod-off.json",
    "2.3:quit",
  ].join(";"),
});
