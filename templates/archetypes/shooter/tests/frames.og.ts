// Two frames at 1280×720 compared with references/<tier>/, one pair per
// view (first person, over the shoulder, isometric) and for the
// first-person split screen: the title over the landing field, and the
// fight, the rifle mid-burst (its flash, a tracer, the impacts) at two
// walkers and a drone charging its beam, with the shooter kit's HUD.
// `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { AIM_RIGHT, ISOMETRIC, LOCAL, SEATS, VARIANT } from "./lib.ts";

const FILES = ["title.png", "fight.png"];

export default playtest({
  name: `the title and the fight match their reference frames (${VARIANT})`,
  size: "1280x720",
  commands: [
    "0.9:screenshot:title.png",
    "1.0:run",
    `1.1:warp:${-3 - AIM_RIGHT}:11`,
    ...(SEATS > 1 ? ["1.1:warp:2:11:1", "1.1:aim:20:-3:1"] : []),
    `1.2:aim:${ISOMETRIC ? 30 : 8}:-3`,
    "1.3:spawn:walker:-5:1",
    "1.3:spawn:walker:-2.4:3",
    "1.3:spawn:drone:-7:-2",
    "2.2:key_down:KeyF",
    ...(LOCAL ? ["2.2:key_down:Slash"] : []),
    "2.5:screenshot:fight.png",
    "2.6:key_up:KeyF",
    ...(LOCAL ? ["2.6:key_up:Slash"] : []),
    "2.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, `${VARIANT}-`, VARIANT === "fp");
  },
});
