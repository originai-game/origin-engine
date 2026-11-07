// The heal alone, as `og test` plays it: it resolves once, and the frame
// of its resolve flash over the post matches its Metal reference.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "a heal resolves once and its resolve draws as recorded",
  size: "960x540",
  require: ["the heal resolves"],
  commands: [
    "0.5:heal",
    "1.72:screenshot:heal.png",
    "1.72:expect:frame:references/metal/heal.png:rmse:<=3",
    "2.6:expect:state:healed==1",
    "2.6:expect:state:impacts==0",
    "2.65:quit",
  ].join(";"),
});
