// Four frames at 1280×720 compared with references/<tier>/<variant>/, the
// variant this scaffold's view and seats (third-person, first-person or
// isometric; `-split` with two seats on one device, `-online` over a room):
//   title.png      the title over the field lapping on the autopilot;
//   countdown.png  the grid with two of the gantry's lamps lit, the banner's 2;
//   drift.png      seat 0 drifting into the fast right sweeper behind the
//                  pack, smoke off the rear tyres, the boost ring filling;
//   results.png    the classification after seat 0 (and seat 1) won.
// `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { FIRST_PERSON, ISOMETRIC, LOCAL, ONLINE, SEATS, sorted, TOUCH } from "./lib.ts";

const VARIANT = `${FIRST_PERSON ? "first-person" : ISOMETRIC ? "isometric" : "third-person"}${LOCAL ? "-split" : ONLINE ? "-online" : ""}${TOUCH ? "-touch" : ""}`;
const FILES = ["title.png", "countdown.png", "drift.png", "results.png"];
const pack = [[92, -3, 26], [100, 3, 27], [112, -1, 27], [125, 3, 28], [140, 0, 28]].slice(SEATS - 1);
const each = (cmd: (seat: number) => string) => Array.from({ length: SEATS }, (_, s) => cmd(s));

export default playtest({
  name: "the title, the countdown, a drift and the results match their reference frames",
  size: "1280x720",
  commands: sorted([
    "1.8:screenshot:title.png",
    "1.9:race",
    "3.6:screenshot:countdown.png",
    ...pack.map(([a, x, v], k) => `5.4:warp_car:${SEATS + k}:${a}:${x}:${v}`),
    "5.4:warp:70:-2:30",
    ...(SEATS > 1 ? ["5.4:warp_seat:1:60:3:30"] : []),
    "5.4:key_down:KeyW",
    "5.55:key_down:KeyD",
    "5.6:key_down:KeyS",
    "5.85:key_up:KeyD",
    ...(LOCAL ? ["5.4:key_down:ArrowUp", "5.55:key_down:ArrowRight", "5.6:key_down:ArrowDown", "5.85:key_up:ArrowRight"] : []),
    "6.3:screenshot:drift.png",
    "6.4:key_up:KeyS",
    "6.4:key_up:KeyW",
    ...(LOCAL ? ["6.4:key_up:ArrowUp", "6.4:key_up:ArrowDown"] : []),
    ...each((s) => `6.5:warp_seat:${s}:300`),
    // Ninety seconds into the race, so the last lap and the race read as a real race's (a 25 s lap).
    "6.55:set:Game.clock=5400",
    ...each((s) => `6.6:lap:${s}:2`),
    ...each((s) => `6.6:warp_seat:${s}:${-15 - 12 * s}:${s ? 3 : -3}:30`),
    "11.4:screenshot:results.png",
    "11.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, `${VARIANT}/`);
  },
});
