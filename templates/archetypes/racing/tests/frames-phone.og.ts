// A race on a phone held sideways (844×390 at 3×, the notch on the left):
// the touch controls in the bottom corners (Left and Right; Brake, Boost
// and Throttle), the place and lap top-left, the map beside Pause
// top-right, the speedometer beside Left and Right, seat 0 on the touch Throttle
// behind the pack. Compared with references/<tier>/<variant>/race-phone.png.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { finger, FIRST_PERSON, ISOMETRIC, LOCAL, ONLINE, SEATS, sorted, TOUCH } from "./lib.ts";

const VARIANT = `${FIRST_PERSON ? "first-person" : ISOMETRIC ? "isometric" : "third-person"}${LOCAL ? "-split" : ONLINE ? "-online" : ""}${TOUCH ? "-touch" : ""}`;
const FILES = ["race-phone.png"];
const pack = [[34, -3, 22], [44, 3, 23], [54, -1, 23], [64, 3, 24], [74, 0, 24]].slice(SEATS - 1);

export default playtest({
  name: "a race on a phone held sideways matches its reference frame",
  size: "2532x1170",
  commands: sorted([
    "0.1:viewport:landscape",
    "0.3:race:go",
    ...pack.map(([a, x, v], k) => `0.5:warp_car:${SEATS + k}:${a}:${x}:${v}`),
    "0.5:warp:8:1:20",
    ...(SEATS > 1 ? ["0.5:warp_seat:1:0:-3:20"] : []),
    finger(LOCAL ? "Touch0/Pedals/Right/Throttle" : "Pedals/Pedals/Right/Throttle", 0.6, 2.0, 1),
    "2.4:screenshot:race-phone.png",
    "2.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => t.equal(run.png(FILES[0]!).width, 2532, "race-phone.png is 2532 wide"));
    checkFrames(run, t, FILES, `${VARIANT}/`);
  },
});
