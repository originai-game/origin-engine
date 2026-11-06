// The seat: every device reaches the game through seat 0's og.PlayerInput
// (src/seats.ts). Space held is the `tap` bit and taps the orb; the pad's
// X is `buy` and buys the chosen generator (refused here, too dear); a
// finger on the orb is `point` with the cursor on the orb and taps it; a
// finger beside the orb points elsewhere and taps nothing.
import { playtest } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";
import { game, ORB } from "./lib.ts";

const bit = (name: string) => 1 << creation.controls.buttons.findIndex((b) => b.name === name);

interface Input { player: number; buttons: number; aimX: number; aimY: number }

export default playtest({
  name: "keys, the pad and a finger each reach seat 0's og.PlayerInput and play the orb",
  size: "1280x720",
  commands: [
    "0.3:play",
    "1.0:key_down:Space",
    "1.1:get:seat-0.og.PlayerInput:key.json",
    "1.12:key_up:Space",
    "1.3:get:Game:tapped.json",
    "1.4:gamepad:x",
    "1.5:get:seat-0.og.PlayerInput:pad.json",
    "1.55:gamepad:off",
    "1.7:get:Game:refused.json",
    `1.8:touch_move:${ORB.x + 15},${ORB.y + 10}`,
    "1.82:touch_start",
    "1.9:get:seat-0.og.PlayerInput:finger.json",
    "1.95:touch_end",
    "2.1:get:Game:poked.json",
    `2.2:touch_move:${ORB.x + 300},${ORB.y}`,
    "2.22:touch_start",
    "2.3:get:seat-0.og.PlayerInput:beside.json",
    "2.35:touch_end",
    "2.5:get:Game:beside-game.json",
    "2.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const input = (file: string) => run.json<{ value: Input }>(file).value;
    const key = input("key.json");
    t.equal(key.player, 0, "the seat's entity carries seat 0");
    t.ok((key.buttons & bit("tap")) !== 0, `Space held is the tap bit (buttons ${key.buttons})`);
    t.equal(game(run, "tapped.json").taps, 1, "and tapped the orb once");
    t.ok((input("pad.json").buttons & bit("buy")) !== 0, "the pad's X is the buy bit");
    t.equal(game(run, "refused.json").refused, 1, "and asked for the chosen generator: refused, too dear");
    const finger = input("finger.json");
    t.ok((finger.buttons & bit("point")) !== 0, "a finger down is the point bit");
    // The orb's centre is (0, 0.35) m; 72 CSS px a metre on the 720 px tall surface.
    t.ok(Math.abs(finger.aimX - 15 / 72) < 0.05 && Math.abs(finger.aimY - (0.35 - 10 / 72)) < 0.05, `with the cursor where it landed on the orb (${finger.aimX.toFixed(2)}, ${finger.aimY.toFixed(2)})`);
    t.equal(game(run, "poked.json").taps, 2, "and tapped it");
    t.ok(input("beside.json").aimX > 3, "a finger beside the orb points beside it");
    t.equal(game(run, "beside-game.json").taps, 2, "and tapped nothing");
  },
});
