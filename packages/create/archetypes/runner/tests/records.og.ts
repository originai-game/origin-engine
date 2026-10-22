// The runner on og.platform (here the native host's local store, in
// memory for each run): the first tick asks the store for the best score,
// a crash submits the score to the `main` board, files a `round.over`
// event and, being a new best, saves it; `dump_platform` shows each call,
// its tick and its answer.
import { playtest } from "@origingame/engine/playtest";
import { OVER, runOf, tap } from "./lib.ts";

interface PlatformDump {
  host: string;
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; state: string; result?: { ok: boolean; data: Record<string, unknown> } }>;
}

export default playtest({
  name: "the best score is read from the store, and a crash submits the score and saves the new best",
  commands: [
    tap(0.5),
    "0.6:set:runner.Tuning.spawnInterval=60",
    "2.2:place:crate:3",
    "3.6:dump_entity:runner:over.json",
    "3.7:dump_platform:platform.json",
    "3.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const over = runOf(run, "over.json");
    t.equal(over.phase, OVER, "the crate ended the round");
    const p = run.json<PlatformDump>("platform.json");
    t.equal(p.host, "local", "og run answers from its local store");
    const of = (m: string) => p.calls.filter((c) => c.method === m);
    const get = of("store.get")[0];
    t.ok(get?.tick === 0 && get.args["key"] === "best", "tick 0 asks the store for `best`");
    t.equal(get?.result?.data["value"], null, "nothing was stored before this run");
    const submit = of("leaderboard.submit")[0];
    t.equal(submit?.args["score"], over.score, `the crash submitted the score (${over.score})`);
    t.equal(submit?.result?.data["rank"], 1, "the only run on the local board ranks first");
    const set = of("store.set")[0];
    t.equal(set?.args["value"], over.score, "the new best was saved");
    t.equal(set?.result?.data["revision"], 1, "as the key's first revision");
    t.equal(of("telemetry")[0]?.args["name"], "round.over", "a round.over event was filed");
    t.equal(of("telemetry")[0]?.state, "sent", "one-way: nothing answers it");
  },
});
