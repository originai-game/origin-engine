// Three frames compared with references/<tier>/, one set per view and
// player count (and one for a phone in portrait): the title over the isle
// in the morning, the chapel just claimed ahead of the explorer (its flag
// rising, the brazier catching: fire, smoke, the burst, its light, the
// toast) and the first night there (the sky dark, the fog closed in, the
// brazier and the lantern lit). Two on one device draw each frame in
// two halves. `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";
import { checkFrames } from "./frames.ts";
import { LOCAL, script, VIEW } from "./lib.ts";

const PHONE = (creation.platform as string) === "phone-portrait";
const SHORT: Record<string, string> = { "third-person-3d": "third", "first-person-3d": "first", "isometric-2.5d": "iso" };
const PREFIX = `${PHONE ? "phone-" : ""}${SHORT[VIEW] ?? VIEW}-${LOCAL ? "local" : "solo"}-`;
const FILES = ["title.png", "claim.png", "night.png"];

export default playtest({
  name: "the title, a landmark claimed and the first night by it match their reference frames",
  size: PHONE ? "1170x2532" : "1280x720",
  commands: script([
    ...(PHONE ? ["0.1:viewport:phone"] : []),
    "0.9:screenshot:title.png",
    "1.0:run",
    "1.1:warp:2",
    ...(LOCAL ? ["1.1:warp_seat:1:2"] : []),
    "1.4:claim:2",
    "2.3:screenshot:claim.png",
    "2.5:clock:116",
    "4.4:screenshot:night.png",
    "4.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, PHONE ? 1170 : 1280, `${f} is ${PHONE ? 1170 : 1280} wide`);
    });
    checkFrames(run, t, FILES, PREFIX);
  },
});
