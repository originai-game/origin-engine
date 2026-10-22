// The lobby by the keys and a pad: Space on the title is Play (the lobby);
// Space sits the WASD half down (seat 1, Pip), Enter the arrows (seat 2,
// Juno), a pad's A sits it in the next seat (Moss) and its B stands it up
// again; Q stands WASD up and Space sits it down again, and D picks its
// colour, skipping the one Juno wears; Start stays disabled while one sits
// alone; Escape in the lobby goes back to the title (everyone up) and Space
// returns; when both are ready a "Starting in" count runs and the match
// begins on its own: round 1's count-in, two standing.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, INTRO, KEYS_LEFT, KEYS_RIGHT, LOBBY, pad, PAD1, press, script, texts, TITLE } from "./lib.ts";

export default playtest({
  name: "keys and a pad sit down, pick colours, stand up and ready up in the lobby; Escape returns to the title; all ready starts the match",
  commands: script(
    press("Space", 0.5),
    "1.0:get:Game:lobby.json",
    press("Space", 1.1),
    "1.5:get:Game:one.json",
    "1.5:dump_ui:one-ui.json",
    press("Escape", 1.6),
    "2.1:get:Game:title.json",
    // Play takes the focus back about 0.7 s after the iris begins; a Space before that is lost (FINDINGS.md #10).
    press("Space", 2.4),
    press("Space", 2.7),
    press("Enter", 2.9),
    pad("a", PAD1, 3.1),
    "3.5:get:Game:three.json",
    pad("b", PAD1, 3.6),
    press("KeyQ", 3.8),
    "4.2:get:Game:left.json",
    press("Space", 4.3),
    press("KeyD", 4.6),
    "5.0:get:Game:colour.json",
    press("Space", 5.1),
    press("Enter", 5.3),
    "5.8:get:Game:ready.json",
    "5.8:dump_ui:ready-ui.json",
    "7.5:get:Game:match.json",
    "7.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const lobby = game(run, "lobby.json");
    t.ok(lobby.phase === LOBBY && lobby.leftTitle && !lobby.joined0, "Space on the title is Play: the lobby, nobody seated yet");
    const one = game(run, "one.json");
    t.ok(one.joined0 && one.source0 === KEYS_LEFT && one.colour0 === 0 && !one.joined1, "Space sits the WASD half in the first seat, in Coral");
    const start = elements(run, "one-ui.json").find((e) => e.id === "Start");
    t.ok(start !== undefined && start.states.includes("disabled"), "Start is disabled while one player sits alone");
    const title = game(run, "title.json");
    t.ok(title.phase === TITLE && !title.joined0 && title.source0 === -1, "Escape in the lobby goes back to the title and the seat is given up");
    const three = game(run, "three.json");
    t.ok(three.phase === LOBBY && three.joined0 && three.joined1 && three.joined2, "Space again (Play), then Space, Enter and pad 2's A sit three players down");
    t.ok(three.source1 === KEYS_RIGHT && three.source2 === PAD1 && three.colour1 === 1 && three.colour2 === 2, `the arrows and pad 2 took the next seats and the next free colours (sources ${three.source1}, ${three.source2}; colours ${three.colour1}, ${three.colour2})`);
    const left = game(run, "left.json");
    t.ok(!left.joined2 && !left.joined0 && left.joined1, "pad 2's B and Q stand their players up");
    t.equal(left.leaves, 2, "two leaves counted (B's and Q's; Escape's return to the title stands everyone up without one)");
    const colour = game(run, "colour.json");
    t.ok(colour.joined0 && colour.colour0 === 2, `Space sits WASD down again and D skips Juno's Sun for Mint (colour ${colour.colour0})`);
    const ready = game(run, "ready.json");
    t.ok(ready.ready0 && ready.ready1 && ready.launch > 0, `Space and Enter ready both players and the launch count runs (${ready.launch} ticks in)`);
    const words = texts(run, "ready-ui.json");
    t.ok(words.includes("P1 · Moss") && words.includes("P2 · Juno") && words.filter((w) => w === "Ready!").length === 2, "the seat cards name Moss and Juno, both Ready!");
    t.ok(words.some((w) => w.startsWith("Starting in ")), `the lobby says "Starting in…" (${words.find((w) => w.startsWith("Starting"))})`);
    const match = game(run, "match.json");
    t.ok(match.phase === INTRO && match.round === 0 && match.matches === 1 && match.standing === 2, `the match began on its own: round 1's count-in with two standing (phase ${match.phase}, matches ${match.matches})`);
    t.equal(match.joins, 5, "five joins in all");
  },
});
