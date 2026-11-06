// Three frames compared with references/<tier>/, named after this
// scaffold's view (and -local for two on one screen, -phone for a phone in
// portrait): the title over the yard, a blow landing on a risen (its flash,
// the numbers, the sparks, its frame at the top, the quest tracker), and
// Maren's conversation with its replies. `OG_FX_CAPTURE=1 og test .`
// records this tier's references; the template carries every view's.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { FLAVOUR, press, script, SEATS, TOUCH } from "./lib.ts";

const FILES = [`title-${FLAVOUR}.png`, `fight-${FLAVOUR}.png`, `talk-${FLAVOUR}.png`];

export default playtest({
  name: `the title, a blow landing and Maren's conversation match their reference frames (${FLAVOUR})`,
  size: TOUCH ? "1170x2532" : "1280x720",
  commands: script(
    ...(TOUCH ? ["0.1:viewport:phone"] : []),
    `1.2:screenshot:${FILES[0]}`,
    "1.3:run",
    "1.4:quest:clear",
    "2.2:warp:foe1",
    ...(SEATS > 1 ? ["2.2:warp_seat:1:foe2"] : []),
    press("Space", 2.25),
    `2.45:screenshot:${FILES[1]}`,
    "3.2:warp:maren",
    ...(SEATS > 1 ? ["3.2:warp_seat:1:gate"] : []),
    press("KeyE", 3.4),
    `4.2:screenshot:${FILES[2]}`,
    "4.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, TOUCH ? 1170 : 1280, `${f} is ${TOUCH ? 1170 : 1280} wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
