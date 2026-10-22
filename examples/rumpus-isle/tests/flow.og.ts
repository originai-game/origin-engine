// A whole match for three (Pip on WASD, Juno on the arrows, Moss on pad 1),
// by the commands: the round's card and its count-in, Escape pauses the
// round (the clock held, the pause menu) and Resume lets it run; two knocks
// end the bump ring 3, 2, 1; the scoreboard's Next begins the berry rush,
// whose whistle with no berries taken is a three-way tie (+3 each); the
// tiles' whistle the same; the podium stands Pip on its top step and names
// him; Play again starts a new match with the points cleared; a podium of
// equal points names both; New players is the lobby with the seats kept;
// Escape there is the title.
import { playtest } from "@origingame/engine/playtest";
import { game, INTRO, LOBBY, OVER, pad, PAD0, PLAY, PODIUM, press, script, tap, texts, TITLE, value } from "./lib.ts";

export default playtest({
  name: "a match for three by the commands: card and count-in, pause and resume, three rounds' places and points, the podium, play again, the lobby and the title",
  commands: script(
    "0.2:seat:wasd",
    "0.3:seat:arrows",
    pad("a", PAD0, 0.4),
    "0.7:get:Game:lobby.json",
    "0.8:match",
    // The iris keeps the lobby on screen for half a second after the match begins.
    "1.4:dump_ui:card.json",
    "1.4:get:Game:intro.json",
    "2.0:dump_ui:count.json",
    "2.1:go",
    press("Escape", 2.3),
    "2.5:get:Game:paused.json",
    "2.5:get:og.TimeScale:held.json",
    "3.5:get:Game:still.json",
    "3.5:dump_ui:pause.json",
    tap("resume", 3.6),
    "4.1:get:Game:resumed.json",
    "4.2:knock:2",
    "4.5:knock:1",
    "4.8:get:Game:round1.json",
    "6.8:dump_ui:scores1.json",
    tap("Next", 6.9),
    "7.3:get:Game:round2.json",
    "7.4:go",
    "7.6:finish",
    "7.9:get:Game:tie.json",
    "7.9:dump_ui:tie-ui.json",
    "9.7:next",
    "9.8:go",
    "10.0:finish",
    "11.9:next",
    "12.4:get:Game:podium.json",
    "12.9:dump_ui:podium-ui.json",
    tap("Again", 13.0),
    "13.4:get:Game:again.json",
    "13.5:podium:5,5,2",
    "14.1:dump_ui:shared.json",
    tap("Lobby", 14.2),
    "14.7:get:Game:back.json",
    press("Escape", 14.8),
    "15.3:get:Game:title.json",
    "15.4:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const lobby = game(run, "lobby.json");
    t.ok(lobby.phase === LOBBY && lobby.joined0 && lobby.joined1 && lobby.joined2 && lobby.source2 === PAD0, "three sit in the lobby: WASD, the arrows and pad 1");
    const intro = game(run, "intro.json");
    t.ok(intro.phase === INTRO && intro.round === 0 && intro.standing === 3, "the match begins with the first round's count-in, three standing");
    const card = texts(run, "card.json");
    for (const w of ["Round 1 of 3", "Bump Ring", "1/3 · Bump Ring", "0:40"]) t.ok(card.includes(w), `the card and the HUD say "${w}"`);
    t.ok(card.some((w) => w.startsWith("Shove the others into the sea")), "the card says how the round is played");
    t.ok(!card.includes("3"), "no number shows in the card's first second");
    t.ok(texts(run, "count.json").includes("3"), "a second later the count-in shows 3");
    const paused = game(run, "paused.json");
    t.ok(paused.phase === PLAY && paused.paused, "Escape in the round pauses it");
    t.equal(value<{ scale: number }>(run, "held.json").scale, 0, "the time scale is held at 0");
    const still = game(run, "still.json");
    t.equal(still.left, paused.left, `the clock stood still for a second (${still.left.toFixed(2)} s left)`);
    const menu = texts(run, "pause.json");
    for (const w of ["Paused", "Resume", "Settings", "Back to the lobby", "Quit to title"]) t.ok(menu.includes(w), `the pause menu says "${w}"`);
    const resumed = game(run, "resumed.json");
    t.ok(!resumed.paused && resumed.left < still.left - 0.3, `Resume lets the clock run (${resumed.left.toFixed(2)} s left)`);
    const r1 = game(run, "round1.json");
    t.ok(r1.phase === OVER && r1.place0 === 1 && r1.place1 === 2 && r1.place2 === 3, "Moss knocked out first is 3rd, Juno 2nd, Pip 1st");
    t.ok(r1.points0 === 3 && r1.points1 === 2 && r1.points2 === 1, `3, 2 and 1 points (${r1.points0}, ${r1.points1}, ${r1.points2})`);
    const scores1 = texts(run, "scores1.json");
    t.ok(scores1.indexOf("P1 · Pip") < scores1.indexOf("P2 · Juno") && scores1.indexOf("P2 · Juno") < scores1.indexOf("P3 · Moss"), "the scoreboard lists them by place");
    const r2 = game(run, "round2.json");
    t.ok(r2.phase === INTRO && r2.round === 1, "Next begins the berry rush's count-in");
    const tie = game(run, "tie.json");
    t.ok(tie.phase === OVER && tie.place0 === 1 && tie.place1 === 1 && tie.place2 === 1 && tie.gained0 === 3 && tie.gained2 === 3, "the whistle with no berries taken is a three-way tie for 1st, +3 each");
    t.ok(texts(run, "tie-ui.json").includes("Pip & Juno & Moss share the round!"), "the banner names all three");
    const podium = game(run, "podium.json");
    t.ok(podium.phase === PODIUM && podium.points0 === 9 && podium.points1 === 8 && podium.points2 === 7, `the podium after the third round, 9, 8 and 7 points (${podium.points0}, ${podium.points1}, ${podium.points2})`);
    t.ok(podium.place0 === 1 && podium.place1 === 2 && podium.place2 === 3, "placed 1st, 2nd and 3rd");
    t.ok(podium.y0 > podium.y1 && podium.y1 > podium.y2 && podium.y0 > 1 && Math.abs(podium.x0) < 0.1, `Pip stands on the top step, Juno and Moss lower (y ${podium.y0.toFixed(2)}, ${podium.y1.toFixed(2)}, ${podium.y2.toFixed(2)})`);
    const words = texts(run, "podium-ui.json");
    for (const w of ["Pip rules Rumpus Isle!", "1st", "2nd", "3rd", "9 pts", "Play again", "New players", "Title"]) t.ok(words.includes(w), `the podium says "${w}"`);
    const again = game(run, "again.json");
    t.ok(again.phase === INTRO && again.round === 0 && again.matches === 2 && again.points0 === 0 && again.points2 === 0, "Play again starts a second match with the points cleared");
    t.ok(texts(run, "shared.json").includes("Pip & Juno share the isle!"), "a podium of equal points names both");
    const back = game(run, "back.json");
    t.ok(back.phase === LOBBY && back.joined0 && back.joined1 && back.joined2, "New players is the lobby with the three seats kept");
    const title = game(run, "title.json");
    t.ok(title.phase === TITLE && !title.joined0 && !title.joined2, "Escape in the lobby is the title, everyone up");
    t.equal(title.a11yUnnamed, 0, "no focusable control without a name");
  },
});
