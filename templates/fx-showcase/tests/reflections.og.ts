// The reflections station (depth S6; tests/evidence/debt/render-lod): a
// closed room whose og.ReflectionProbe puts the room in the chrome sphere.
// Every check is an `expect:` step (docs/design.md §3.4). The sphere's
// upper half (CSS 360,250 120×50, at 1280×720) shows the room's pale
// ceiling and back wall with the probe (lavapipe 179,171,168) and the
// sky's blue without it (187,207,229); each colour's tolerance leaves out
// the other. Without a renderer the regions are unchecked.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "The reflections station's chrome sphere shows the room with its probe and the sky without",
  size: "1280x720",
  commands: [
    "0.5:station:reflections",
    "1.5:expect:state:station==\"reflections\"",
    "1.5:expect:region:360,250,120,50:color:#b3aba8:20",
    "1.5:screenshot:reflections-probe-on.png",
    "1.55:dump_render:reflections.json",
    // The probe hidden: the sphere reflects the environment's sky.
    "1.6:set:refl-probe.og.ReflectionProbe.visible=false",
    "2.0:expect:region:360,250,120,50:color:#bbcfe5:20",
    "2.0:screenshot:reflections-probe-off.png",
    "2.3:quit",
  ].join(";"),
});
