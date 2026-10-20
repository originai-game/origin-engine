// The win and the second look: the fourth lantern lit wins the run; after a
// beat the results open on og.ui.resultsScreen (the score counting up, the
// stars, the lamplighter's name, a new best) while og.render.applyLook
// swaps the dusk look for the dawn one (looks/dawn.ogt, its depth bands)
// and the clock turns to the next dawn. Again starts a run from nothing
// under the dusk look again.
import { playtest } from "@origingame/engine/playtest";
import { game, LIT, OVER, PLAY, RESULTS_READY, tap, texts, value } from "./lib.ts";

const END = 1.0;

export default playtest({
  name: "four lanterns lit: the results under the dawn look, then Again under the dusk look",
  commands: [
    "0.3:run",
    "0.4:get:og.Stylise:dusk-stylise.json",
    "0.6:light:0",
    "0.7:light:1",
    "0.8:light:2",
    "0.9:get:Game:three.json",
    `${END}:light:3`,
    `${END + 0.05}:get:Game:won.json`,
    `${END + 1.5}:get:Game:over.json`,
    `${END + 1.5}:get:og.Stylise:dawn-stylise.json`,
    `${END + 1.5}:get:og.MaterialStyle:dawn-material.json`,
    `${END + 2.6}:dump_ui:results.json`,
    tap("again", END + RESULTS_READY),
    `${END + RESULTS_READY + 0.5}:get:Game:again.json`,
    `${END + RESULTS_READY + 0.5}:get:og.Stylise:again-stylise.json`,
    `${END + RESULTS_READY + 0.5}:get:lantern-2.Lantern:lantern.json`,
    `${END + RESULTS_READY + 0.6}:quit`,
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const dusk = value<{ depthBands: number; ink: boolean }>(run, "dusk-stylise.json");
    t.ok(dusk.depthBands === 0 && dusk.ink, "a run plays under the dusk look (ink, no depth bands)");
    const three = game(run, "three.json");
    t.ok(three.phase === PLAY && three.lit === 3, "three lanterns lit, the run goes on");
    const won = game(run, "won.json");
    t.ok(won.lit === 4 && won.litMask === 15 && won.ending === LIT, "the fourth wins");
    t.equal(won.phase, PLAY, "the results wait a beat");
    const over = game(run, "over.json");
    t.ok(over.phase === OVER && over.look === 1, "then the results, under the second look");
    t.ok(over.hour > 6 && over.hour < 7, `at the next dawn (${over.hour.toFixed(2)})`);
    t.ok(over.newBest && over.best > 1000, `a new best (${over.best})`);
    t.equal(value<{ depthBands: number }>(run, "dawn-stylise.json").depthBands, 5, "og.render.applyLook replaced Stylise: five depth bands");
    t.equal(value<{ toon: boolean }>(run, "dawn-material.json").toon, false, "and MaterialStyle, whose toon the dawn look takes off");
    const words = texts(run, "results.json");
    t.ok(words.includes("The vale is lit"), "the results say the vale is lit");
    t.ok(words.includes("Nameless") && words.includes("4/4"), "with the lamplighter and the lanterns");
    const again = game(run, "again.json");
    t.ok(again.phase === PLAY && again.lit === 0 && again.hearts === 3 && again.look === 0, "Again started a run from nothing, under the dusk look");
    t.equal(value<{ depthBands: number }>(run, "again-stylise.json").depthBands, 0, "the dusk look's Stylise back");
    t.equal(value<{ lit: boolean }>(run, "lantern.json").lit, false, "the lanterns dark again");
  },
});
