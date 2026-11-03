// What Rumpus Isle keeps in og.platform across a relaunch (store: "keep").
// The first process chooses Français in the settings (saved as og.settings
// when they close) and plays a crumble round to a fall (the longest stood
// written under `records` at the revision read). The relaunch reads both on
// its first tick: the title is in French, its footer names the record, and
// the Game resource carries the best before anyone plays.
import { playtest } from "@origingame/engine/playtest";
import { calls, game, script, tap, texts, TITLE, twoPlayers, value } from "./lib.ts";

export default playtest({
  name: "the settings and the isle's records survive a relaunch: a French title with the record in its footer",
  store: "keep",
  commands: script(
    tap("settings", 0.6),
    tap("Tab#3", 1.2),
    tap("Option#1", 1.6),
    tap("Done", 2.0),
    "2.4:get:Prefs:prefs.json",
    ...twoPlayers(2.5, 3),
    // Standing still, both fall with their tiles 2.73 s into the round (the grace, then crackTicks).
    "6.0:get:Game:fell.json",
    "6.1:dump_platform:calls.json",
    "6.2:quit",
  ),
  then: [
    {
      commands: [
        "0.8:dump_ui:title.json",
        "0.8:get:Game:at-title.json",
        "0.8:get:Prefs:prefs-back.json",
        "0.9:dump_platform:calls.json",
        "1.0:quit",
      ].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    t.equal(value<{ language: number }>(run, "prefs.json").language, 1, "Français chosen on the settings' Language tab");
    const fell = game(run, "fell.json");
    t.ok(fell.round === 2 && fell.splashes >= 1 && fell.bestCrumble > 1 && fell.newRecord, `the crumble round's fall set the record (${fell.bestCrumble} s)`);
    t.ok(fell.saves === 1 && fell.recordsRevision >= 1, `and it was written (${fell.saves} save, revision ${fell.recordsRevision})`);
    const first = calls(run, "calls.json");
    const settings = first.find((c) => c.method === "store.set" && c.args.key === "og.settings");
    t.ok(settings !== undefined && settings.args.value?.language === "fr" && settings.result?.ok === true, "the settings were saved with the language when they closed");
    const records = first.find((c) => c.method === "store.set" && c.args.key === "records");
    // The record is an f32 field: the payload carries it widened (2.700000047683716 for 2.7).
    t.ok(records !== undefined && Math.abs((records.args.value?.crumble as number) - fell.bestCrumble) < 1e-6 && records.args.value?.v === 1, "the records record holds the seconds on the tiles");
    t.ok(records !== undefined && records.args.revision === 0, `written at the revision read, 0 (${records?.args.revision})`);

    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    const title = game(next, "at-title.json");
    t.ok(title.phase === TITLE && title.recordsRead && title.bestCrumble === fell.bestCrumble && title.bestBerries === 0, `the relaunch read the records on its first tick (${title.bestCrumble} s)`);
    t.ok(title.recordsRevision >= 1, `at the revision the first process wrote (${title.recordsRevision})`);
    t.equal(value<{ language: number }>(next, "prefs-back.json").language, 1, "and the settings: French again");
    const words = texts(next, "title.json");
    t.ok(words.includes("Jouer") && words.includes("Réglages"), "the title is in French");
    t.ok(words.includes(`${fell.bestCrumble.toFixed(1)} s sur les Dalles friables`), `its footer names the record (${words.find((w) => w.includes(" s sur "))})`);
    const reads = calls(next, "calls.json").filter((c) => c.method === "store.get").map((c) => c.args.key);
    t.ok(reads.includes("og.settings") && reads.includes("records"), "both were read through og.platform");
  },
});
