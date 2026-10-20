// Two fingers on the materials screen (og.Pointers: the primary touch by
// touch_move/touch_start/touch_end, the second by `touch:1:...`): moved
// up together over the quest list they scroll it by their movement and
// press no quest (the first finger landed on one); spread apart over the
// map they zoom it about their centre (80 px apart to 160: 200 %), and the
// map's value reads the zoom; brought back together it zooms back out.
// Ways it could fail: the first finger's press clicks the row, the list
// stays put, the pinch zooms about the map's corner, or the zoom is not
// in the UI's state.
import { playtest } from "@origingame/engine/playtest";
import { demo, ui } from "./lib.ts";

// At 1280×720 the quest list is at (866, 345) 200×200 and the map at
// (540, 345) 314×177 (tests/evidence/P4/ui-shell/gestures.md); the script
// checks the rows and the map are there before it touches them.
const LIST = { x: 960, y: 423 };
const MAP = { x: 697, y: 433 };
const steps: string[] = ["0.5:show:materials", "1.2:dump_ui:before.json"];
let t = 1.3;
const at = (dt = 0.05) => (t += dt).toFixed(2);
// The list: two fingers 60 px apart on Quest 2, up by 90 in three moves.
steps.push(`${at()}:touch_move:${LIST.x - 30},${LIST.y}`);
steps.push(`${at(0.0)}:touch_start`);
steps.push(`${at()}:touch:1:down:${LIST.x + 30},${LIST.y}`);
for (const dy of [30, 60, 90]) {
  steps.push(`${at()}:touch_move:${LIST.x - 30},${LIST.y - dy}`);
  steps.push(`${at(0.0)}:touch:1:move:${LIST.x + 30},${LIST.y - dy}`);
}
steps.push(`${at()}:dump_ui:scrolled.json`);
steps.push(`${at()}:touch:1:up:${LIST.x + 30},${LIST.y - 90}`);
steps.push(`${at()}:touch_end`);
steps.push(`${at(0.2)}:dump_entity:demo:list.json`);
// The map: 80 px apart about its centre, to 160, then back to 80.
steps.push(`${at()}:touch_move:${MAP.x - 40},${MAP.y}`);
steps.push(`${at(0.0)}:touch_start`);
steps.push(`${at()}:touch:1:down:${MAP.x + 40},${MAP.y}`);
for (const d of [60, 70, 80]) {
  steps.push(`${at()}:touch_move:${MAP.x - d},${MAP.y}`);
  steps.push(`${at(0.0)}:touch:1:move:${MAP.x + d},${MAP.y}`);
}
steps.push(`${at()}:dump_entity:demo:zoomed.json`);
steps.push(`${at(0.0)}:dump_ui:zoomed-ui.json`);
for (const d of [60, 40]) {
  steps.push(`${at()}:touch_move:${MAP.x - d},${MAP.y}`);
  steps.push(`${at(0.0)}:touch:1:move:${MAP.x + d},${MAP.y}`);
}
steps.push(`${at()}:touch_end`);
steps.push(`${at()}:touch:1:up:${MAP.x + 40},${MAP.y}`);
steps.push(`${at(0.1)}:dump_entity:demo:back.json`);
steps.push(`${at()}:quit`);

export default playtest({
  name: "two fingers scroll the quest list and pinch the map",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = ui(run, "before.json");
    const el = (els: typeof before, id: string) => els.find((e) => e.id === id);
    const list = el(before, "Quests");
    const map = el(before, "Map");
    t.ok(!!list && list.x <= LIST.x - 30 && list.x + list.width >= LIST.x + 30 && list.y <= LIST.y && list.y + list.height >= LIST.y, `the list is under the fingers (${JSON.stringify(list)})`);
    t.ok(!!map && Math.abs(map.x + map.width / 2 - MAP.x) <= 1 && Math.abs(map.y + map.height / 2 - MAP.y) <= 1, `the fingers pinch about the map's centre (${JSON.stringify(map)})`);
    const q5 = el(before, "Quest#5");
    const moved = el(ui(run, "scrolled.json"), "Quest#5");
    t.equal(moved && q5 ? q5.y - moved.y : null, 90, "the list followed the two fingers up by 90 px");
    t.equal(demo(run, "list.json").quests, 0, "no quest was pressed");
    const zoomed = demo(run, "zoomed.json").zoom;
    t.ok(Math.abs(zoomed - 2) < 1e-4, `the pinch doubled the zoom (${zoomed})`);
    t.equal(el(ui(run, "zoomed-ui.json"), "Map")?.value, "200%", "the map's value reads its zoom");
    const back = demo(run, "back.json").zoom;
    t.ok(Math.abs(back - 1) < 1e-4, `bringing the fingers back zoomed it out to the whole picture (${back})`);
  },
});
