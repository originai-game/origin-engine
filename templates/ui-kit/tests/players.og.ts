// Local multiplayer on one screen (og.ui.setPlayers): pads 1 and 2 each
// move a focus of their own, shown as a ring in their player's colour, and
// pick a hero with A for their own player; a touch picks for player 1
// (pad 0's shared focus); the frame with both rings matches
// crates/og-ui/references/<tier>/kit-desktop-players.png.
// Ways it could fail: an extra pad moves the shared focus or another pad's,
// its A picks for player 1 or for no one, or its ring is not drawn.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { SCREEN, demo, tap } from "./lib.ts";

const pad = (t: number, buttons: string, n: number): string =>
  [`${t.toFixed(2)}:gamepad:${buttons}@${n}`, `${(t + 0.05).toFixed(2)}:gamepad:-@${n}`].join(";");

export default playtest({
  name: "pads 1 and 2 move their own rings and pick for their own players",
  size: "1280x720",
  commands: [
    "0.3:show:players",
    // Each extra pad's first move lands on the hero nearest the centre (Mage).
    pad(0.8, "right", 1),
    pad(1.0, "down", 2),
    pad(1.2, "right", 2),
    "1.4:dump_entity:demo:moved.json",
    "1.5:screenshot:players.png",
    pad(1.6, "a", 1),
    pad(1.8, "a", 2),
    tap(2.0, "Heroes/Hero#7"),
    "2.4:dump_entity:demo:picked.json",
    "2.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const moved = demo(run, "moved.json");
    t.equal(moved.screen, SCREEN.players, "the lobby is open");
    t.equal([moved.focus0, moved.focus1, moved.focus2, moved.focus3], [0, 3, 4, 0], "pad 1 is on Mage, pad 2 moved on to Rogue, the shared focus has not moved");
    const picked = demo(run, "picked.json");
    t.equal([picked.pick0, picked.pick1, picked.pick2, picked.pick3], [8, 3, 4, 0], "A picked Mage for P2 and Rogue for P3; the touch picked Thief for P1");
    checkFrames(run, t, ["players.png"], "kit-desktop-");
  },
});
