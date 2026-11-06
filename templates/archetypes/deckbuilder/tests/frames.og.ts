// Three frames compared with references/<tier>/: the title over the field,
// the first fight with the hand dealt and the keys' focus on its second
// card and a foe, and the card reward after the fight. Each scaffold's
// frames are its own, by its view, its players and its platform (side or
// top, -local with two on one device, -phone at 1170 × 2532 for a phone in
// portrait). `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { game, PHONE, press, REWARD, script, TWO, VIEW } from "./lib.ts";

const FILES = ["title.png", "battle.png", "reward.png"];
const PREFIX = `${VIEW === "top-down-2d" ? "top" : "side"}${TWO ? "-local" : ""}${PHONE ? "-phone" : ""}-`;

export default playtest({
  name: "the title, a fight and its reward match their reference frames",
  size: PHONE ? "1170x2532" : "1280x720",
  commands: script([
    ...(PHONE ? ["0.1:viewport:phone"] : []),
    "0.9:screenshot:title.png",
    "1.0:goto:0",
    press("ArrowRight", 1.6),
    press("ArrowUp", 1.8),
    "2.6:screenshot:battle.png",
    "2.7:foehp:0:1",
    "2.7:foehp:1:1",
    "2.7:give:cleave",
    // Two lefts wrap from the second card to the last, the Cleave just given; it fells both.
    press("ArrowLeft", 2.8),
    press("ArrowLeft", 2.95),
    press("Enter", 3.1),
    "5.3:get:Game:won.json",
    "5.4:screenshot:reward.png",
    "5.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, PHONE ? 1170 : 1280, `${f} is ${PHONE ? 1170 : 1280} wide`);
    });
    t.equal(game(run, "won.json").phase, REWARD, "the Cleave felled both: the reward is on");
    checkFrames(run, t, FILES, PREFIX);
  },
});
