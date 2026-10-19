// The core loop through real input: Play on the title; taps on the orb
// (a click on it, Space, Enter) make sparks; a click beside the orb makes
// none; 1 buys an Ember and the HUD's button a second, each at its price
// (15, then 15 × 1.15 rounded up); the embers then make sparks on their
// own, half a spark a second each; a purchase the sparks cannot cover is
// refused and costs nothing.
import { playtest } from "@origingame/engine/playtest";
import { game, ORB, PLAY, press, tap, tapAt, TITLE } from "./lib.ts";

export default playtest({
  name: "taps make sparks, sparks buy embers, embers make sparks, a purchase too dear is refused",
  commands: [
    "0.3:get:Game:title.json",
    tap("play", 0.9),
    "1.6:get:Game:start.json",
    ...Array.from({ length: 10 }, (_, i) => tapAt(ORB.x + (i % 3) * 20 - 20, ORB.y + 10, +(1.7 + i * 0.12).toFixed(2))),
    tapAt(ORB.x + 300, ORB.y, 3.0),
    press("Space", 3.2),
    press("Enter", 3.3),
    "3.5:get:Game:tapped.json",
    "3.55:screenshot:tapped.png",
    "3.6:sparks:40",
    press("Digit1", 3.7),
    "3.8:get:Game:one.json",
    tap("Shop/Gen#0/Buy", 3.9),
    "4.1:touch_move:200,300",
    "4.2:get:Game:two.json",
    "6.2:get:Game:later.json",
    press("Digit4", 6.3),
    "6.4:get:Game:refused.json",
    "6.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const start = game(run, "start.json");
    t.equal(start.phase, PLAY, "Play showed the orb");
    t.equal(start.sparks, 0, "with no sparks");
    const tapped = game(run, "tapped.json");
    t.equal(tapped.taps, 12, "ten clicks on the orb, Space and Enter: twelve taps; the click beside it none");
    t.equal(tapped.sparks, 12, "a spark a tap");
    const one = game(run, "one.json");
    t.equal(one.owned0, 1, "1 bought an Ember");
    t.ok(Math.abs(one.sparks - (40 - 15)) < 0.1, `for 15 sparks (${one.sparks.toFixed(2)} left)`);
    t.equal(one.rate, 0.5, "half a spark a second");
    const two = game(run, "two.json");
    t.equal(two.owned0, 2, "the HUD's button bought a second");
    t.ok(Math.abs(two.sparks - (one.sparks - 18)) < 0.6, `for 18 (15 × 1.15, rounded up) (${one.sparks.toFixed(2)} → ${two.sparks.toFixed(2)}, the embers making a little meanwhile)`);
    const later = game(run, "later.json");
    t.ok(Math.abs(later.sparks - two.sparks - 2 * 1.0 * 2.0 / 2) < 0.05, `two embers made a spark a second for two seconds (${two.sparks.toFixed(2)} → ${later.sparks.toFixed(2)})`);
    const refused = game(run, "refused.json");
    t.equal(refused.refused, 1, "a Star (13,000) was refused");
    t.equal(refused.owned3, 0, "none bought");
    t.ok(refused.sparks >= later.sparks, "and nothing spent");
  },
});
