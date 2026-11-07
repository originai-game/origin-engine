// The farmer walks down the path at dusk as `og test` plays it: her
// flipbook's step events fire, she stays on the path tiles, a tilled cell
// changes the map, and the frame matches its Metal reference.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "the farmer walks the path at dusk, a cell is watered, and the farm draws as recorded",
  size: "960x540",
  require: ["the farmer steps"],
  commands: [
    "0.3:walk",
    "0.5:till:12:9",
    "2.2:screenshot:farm.png",
    "2.2:expect:frame:references/metal/farm.png:rmse:<=3",
    "2.25:expect:state:steps>=3",
    "2.25:expect:state:walked>=3.5",
    "2.25:expect:state:onPath==1",
    "2.25:expect:state:watered==7",
    "2.3:quit",
  ].join(";"),
});
