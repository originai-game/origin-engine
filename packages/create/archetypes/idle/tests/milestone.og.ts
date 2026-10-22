// Milestones and big numbers: the tenth Storm doubles what storms make
// (a burst from its card, a fanfare, "Storm ×2!"); the wallet shows 1.23M
// as "1.23M" and the rate in the same words; a supernova's stardust makes
// everything 10 % stronger each.
import { playtest } from "@origingame/engine/playtest";
import { game, press, text } from "./lib.ts";

export default playtest({
  name: "the tenth of a kind doubles it; big numbers read 1.23M",
  commands: [
    "0.3:play",
    "0.9:own:1:9",
    "1.0:get:Game:nine.json",
    "1.1:sparks:1234567",
    press("Digit2", 1.2),
    "1.35:get:Game:ten.json",
    "1.4:dump_ui:ui.json",
    "1.45:screenshot:milestone.png",
    "1.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const nine = game(run, "nine.json");
    t.equal(nine.rate, 36, "nine storms make 36 a second (4 each)");
    const ten = game(run, "ten.json");
    t.equal(ten.owned1, 10, "2 bought the tenth");
    t.equal(ten.rate, 80, "ten storms make twice 40: 80 a second");
    t.equal(text(run, "ui.json", "Amount", "Sparks"), `${(ten.sparks / 1e6).toFixed(2)}M`, `the wallet reads ${text(run, "ui.json", "Amount", "Sparks")}`);
    t.equal(text(run, "ui.json", "Rate"), "+80 a second", "the rate reads +80 a second");
    t.equal(text(run, "ui.json", "Owned", "Head"), "×0", "the first card counts its embers");
  },
});
