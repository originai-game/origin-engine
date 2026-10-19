// Latency calibration: on the calibration screen the clock clicks every
// beat; eight taps, each four ticks (67 ms) after its click, measure an
// offset of 67 ms; Done saves it. Then a play whose every press is 67 ms
// late is judged Perfect throughout, where without the offset each would
// be a Great.
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, type PlatformDump, play, press, songStart, sorted, tap, TITLE } from "./lib.ts";

const notes = chart();
/** The `calibrate` command at 1.0 s: clicks from the next whole tick half a second on, every beat. */
const c0 = Math.ceil((1.0 + 0.5) * 60) / 60;
const late = 4 / 60;
const t0 = songStart(7.0);

interface Src { components: { "og.AudioSource": { startAt: number } } }

export default playtest({
  name: "eight taps 67 ms late calibrate a 67 ms offset, and a late play is judged Perfect",
  commands: sorted([
    "1.0:calibrate",
    ...Array.from({ length: 8 }, (_, k) => press("KeyS", c0 + (k + 1) * BEAT + late)),
    `${(c0 + 2.1 * BEAT).toFixed(3)}:dump_entity:stem-click3:click.json`,
    `${(c0 + 8.6 * BEAT).toFixed(3)}:get:Game:measured.json`,
    `${(c0 + 8.6 * BEAT).toFixed(3)}:screenshot:calibrate.png`,
    tap("Done", 5.9),
    "6.5:get:Game:done.json",
    "6.5:get:Prefs:prefs.json",
    "6.5:dump_platform:platform.json",
    "7.0:play",
    ...play(notes, t0, 0, 8, late),
    `${(t0 + 7.9 * BEAT).toFixed(3)}:get:Game:played.json`,
    `${(t0 + 8 * BEAT).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const click = run.json<Src>("click.json").components["og.AudioSource"].startAt;
    t.ok(Math.abs((click - c0) / BEAT - Math.round((click - c0) / BEAT)) < 1e-6, `the clicks are started on the beats (${click.toFixed(3)} s)`);
    const m = game(run, "measured.json");
    t.equal(m.calTaps, 8, "eight taps measured");
    t.ok(Math.abs(m.calSum / 8 - late * 1000) < 0.5, `each ${(late * 1000).toFixed(1)} ms after its click (${(m.calSum / 8).toFixed(1)})`);
    t.equal(game(run, "done.json").phase, TITLE, "Done went back to the title");
    const prefs = run.json<{ value: { offsetMs: number } }>("prefs.json").value;
    t.equal(prefs.offsetMs, 67, "the offset is 67 ms");
    const pd = run.json<PlatformDump>("platform.json");
    t.ok(pd.calls.some((c) => c.method === "store.set" && (c.args["value"] as { offsetMs?: number } | undefined)?.offsetMs === 67), "and was saved with the settings");
    const p = game(run, "played.json");
    t.equal(p.perfect, 8, "eight presses 67 ms late were all Perfect");
  },
});
