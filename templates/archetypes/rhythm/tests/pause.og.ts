// Pause, settings and the resume: bar 1 hit, then Escape in bar 2: the
// clock freezes (no note is missed while paused), the band stops, the
// time scale holds; the settings switch to 简体中文 and save; Resume counts
// in again and continues from the beat it paused on: the drums start again
// part-way into bar 2's stem (AudioSource.startOffset), and bar 2's last
// notes are hit at their new times.
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, type PlatformDump, play, press, songStart, sorted, tap, text } from "./lib.ts";

const notes = chart();
const t0 = songStart(1.0);
const pauseAt = t0 + 5.3 * BEAT;
/** The resume (the `unhold` command, as Resume does): the count-in ends where a start's would. */
const resumeAt = 10.0;
const countTo = songStart(resumeAt);
/** The beat Escape paused on: its tick's time into the song. */
const pausedBeat = (Math.round(pauseAt * 60) / 60 - t0) / BEAT;
/** Bar 1's time after the resume: the paused beat comes at the count-in's end. */
const t1 = countTo - pausedBeat * BEAT;

export default playtest({
  name: "Escape freezes the clock and stops the band; 简体中文 is saved; Resume counts in and continues from the beat it paused on",
  commands: sorted([
    "1.0:play",
    ...play(notes, t0, 0, 5.3),
    press("Escape", pauseAt),
    `${(pauseAt + 0.3).toFixed(3)}:get:Game:paused.json`,
    `${(pauseAt + 0.3).toFixed(3)}:get:og.TimeScale.scale:scale.json`,
    `${(pauseAt + 0.3).toFixed(3)}:dump_entity:stem-drums:drums.json`,
    `${(pauseAt + 0.4).toFixed(3)}:screenshot:pause.png`,
    tap("settings", 6.6),
    "7.2:move_to_text:简体中文",
    "7.22:touch_start",
    "7.25:touch_end",
    "7.6:screenshot:settings-zh.png",
    tap("Done", 7.8),
    "8.9:get:Game:held.json",
    "8.9:get:Prefs:prefs.json",
    "8.9:dump_platform:platform.json",
    `${resumeAt}:unhold`,
    `${(resumeAt + 0.1).toFixed(3)}:get:Game:resumed.json`,
    `${(resumeAt + 0.15).toFixed(3)}:dump_entity:stem-drums:drums2.json`,
    ...play(notes, t1, pausedBeat, 8),
    `${(t1 + 7.9 * BEAT).toFixed(3)}:get:Game:after.json`,
    `${(t1 + 7.9 * BEAT).toFixed(3)}:dump_ui:play-zh.json`,
    `${(t1 + 7.95 * BEAT).toFixed(3)}:screenshot:play-zh.png`,
    `${(t1 + 8.0 * BEAT).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const p = game(run, "paused.json");
    t.equal(p.paused, true, "Escape paused");
    t.equal(p.judged, 6, "after bar 1's four notes and bar 2's first two");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "the time scale holds at 0");
    t.equal(run.json<{ components: { "og.AudioSource": { playing: boolean } } }>("drums.json").components["og.AudioSource"].playing, false, "the band stopped");
    const h = game(run, "held.json");
    t.ok(h.judged === 6 && h.miss === 0, "three seconds later nothing was missed: the clock is frozen");
    const prefs = run.json<{ value: { language: number; saves: number } }>("prefs.json").value;
    t.ok(prefs.language === 1 && prefs.saves === 1, "简体中文 was chosen and saved");
    const pd = run.json<PlatformDump>("platform.json");
    t.ok(pd.calls.some((c) => c.method === "store.set" && c.args["key"] === "og.settings" && (c.args["value"] as { language?: string }).language === "zh-CN"), "through og.platform");
    const r = game(run, "resumed.json");
    t.equal(r.paused, false, "Resume");
    t.ok(Math.abs(r.songStart - t1) < 1e-6, `continued from beat ${pausedBeat.toFixed(3)}: bar 1 now at ${t1.toFixed(3)} s, the paused beat at the count-in's end`);
    t.ok(Math.abs(r.countTo - countTo) < 1e-6, `after a count-in to ${countTo.toFixed(3)} s`);
    const drums = run.json<{ components: { "og.AudioSource": { startAt: number; startOffset: number } } }>("drums2.json").components["og.AudioSource"];
    t.ok(Math.abs(drums.startAt - countTo) < 1e-6, "the drums start again at the count-in's end");
    t.ok(Math.abs(drums.startOffset - (pausedBeat - 4) * BEAT) < 1e-6, `part-way into bar 2's stem: ${((pausedBeat - 4) * BEAT).toFixed(3)} s in, not from its start`);
    const a = game(run, "after.json");
    t.equal(a.perfect, 8, "bar 2's last two notes were hit at their new times: eight Perfects");
    t.equal(a.miss, 0, "and nothing missed, bar 1 not judged twice");
    t.equal(text(run, "play-zh.json", "Label", "Score"), "得分", "the HUD speaks 简体中文");
    for (const f of ["pause.png", "settings-zh.png", "play-zh.png"]) t.equal(run.png(f).width, 1280, `${f} captured`);
  },
});
