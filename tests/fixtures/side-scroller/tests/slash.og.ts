// The knight runs, stops and slashes as `og test` plays it: the sprite
// graph reaches its attack state and returns to idle, the slash's hit frame
// blows the mound into pieces, and the frame of the slash matches its Metal
// reference.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "the knight runs and slashes, the hit blows the mound apart, and draws as recorded",
  size: "960x540",
  require: ["the knight slashes", "the mound breaks into pieces"],
  commands: [
    "0.3:run",
    "1.5:attack",
    "1.9:screenshot:slash.png",
    "1.9:expect:frame:references/metal/slash.png:rmse:<=3",
    "2.6:expect:state:hits==1",
    "2.6:expect:state:detached>=1",
    "2.6:expect:state:steps>=2",
    "2.6:expect:state:state==0",
    "2.65:quit",
  ].join(";"),
});
