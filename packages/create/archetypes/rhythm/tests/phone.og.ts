// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu, then the score, the groove and Pause below the notch, the combo
// above the highway, three 2 m lanes across the width with the hit line
// above the home bar; a finger on a lane at its note's time hits it.
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, songStart, sorted, tap, type UiElement, ui } from "./lib.ts";

const notes = chart();
const t0 = songStart(1.0);
/** CSS px per metre: the camera is 7.2 m across 390 px. */
const scale = 390 / 7.2;
const laneX = (lane: number) => 195 + (lane - 1) * 2 * scale;
const touch = (lane: number, t: number) => `${t.toFixed(4)}:touch_move:${laneX(lane).toFixed(1)},690;${(t + 0.0001).toFixed(4)}:touch_start;${(t + 0.05).toFixed(4)}:touch_end`;

export default playtest({
  name: "the phone layout: a HUD inside the safe area, three lanes across, fingers hit the notes",
  size: "1170x2532",
  commands: sorted([
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    "1.0:play",
    ...notes.filter((n) => n.beat < 8).map((n) => touch(n.lane, t0 + n.beat * BEAT)),
    `${(t0 + 7.9 * BEAT).toFixed(3)}:get:Game:hit.json`,
    `${(t0 + 7.9 * BEAT).toFixed(3)}:dump_ui:hud.json`,
    `${(t0 + 7.95 * BEAT).toFixed(3)}:screenshot:play.png`,
    `${(t0 + 8 * BEAT).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = ui(run, "title.json").elements;
    t.ok(title.some((e) => e.id === "play") && title.every((e) => e.y >= 47 && e.y + e.height <= 844 - 34), "the title's menu sits inside the safe area");
    const h = game(run, "hit.json");
    t.equal(h.hits, 8, "eight notes hit by fingers on their lanes");
    t.equal(h.perfect, 8, "all on time");
    const hud = ui(run, "hud.json");
    const el = (id: string): UiElement | undefined => hud.elements.find((e) => e.id === id);
    t.equal(hud.surface.width, 390, "laid out for a 390 px wide phone");
    for (const id of ["Status", "Corner", "Combo"]) t.ok((el(id)?.y ?? 0) >= 47, `${id} sits below the notch`);
    t.ok(hud.elements.every((e) => e.x >= 0 && e.x + e.width <= 390 && e.y + e.height <= 844 - 34), "nothing runs off the sides or under the home bar");
    const pause = el("Pause");
    t.ok(pause !== undefined && pause.width >= 44 && pause.height >= 44, "Pause is touch-sized");
    t.frame("play.png", () => t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels"));
  },
});
