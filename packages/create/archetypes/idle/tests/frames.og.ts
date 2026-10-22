// Three frames at 1280×720 compared with references/<tier>/: the title
// over the orb, a run with every generator circling the orb, and the
// supernova's card. `OG_FX_CAPTURE=1 og test .` records this tier's
// references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

const FILES = ["title.png", "orbit.png", "card.png"];

export default playtest({
  name: "the title, the orb with its motes and the supernova's card match their reference frames",
  size: "1280x720",
  commands: [
    "0.9:screenshot:title.png",
    "1.0:play",
    "1.1:own:0:12",
    "1.1:own:1:8",
    "1.1:own:2:5",
    "1.1:own:3:3",
    "1.2:sparks:2500000",
    "2.2:screenshot:orbit.png",
    "2.3:supernova",
    "4.0:screenshot:card.png",
    "4.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
