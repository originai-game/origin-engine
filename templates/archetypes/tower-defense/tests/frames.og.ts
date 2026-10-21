// Three frames at 1280×720 compared with references/<tier>/: the title over
// the garden, the first wave in a small maze (a thorn shooting, a frosted
// visitor, the cursor and its range), and the won round's end card.
// `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

const FILES = ["title.png", "wave.png", "end.png"];

export default playtest({
  name: "the title, a wave in the maze and the end card match their reference frames",
  size: "1280x720",
  commands: [
    "0.5:screenshot:title.png",
    "0.6:begin",
    "0.7:plant:hedge:4:4",
    "0.7:plant:thorn:3:4",
    "0.7:plant:frost:5:4",
    "6.0:screenshot:wave.png",
    "6.1:win",
    "8.0:screenshot:end.png",
    "8.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
