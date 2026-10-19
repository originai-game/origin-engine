// Two processes, one kept store: the first saves at its host time, the
// second launches two hours later by its clock and reads the time away.
import { playtest } from "@origingame/engine/playtest";

const T = 1767225600000;

export default playtest({
  name: "a relaunch two hours later reads the time away from the kept store",
  store: "keep",
  clock: T,
  commands: "0.5:save;0.7:get:Away:first.json;0.8:quit",
  then: [{ commands: "0.3:get:Away:second.json;0.4:quit", clock: T + 2 * 3600 * 1000 }],
  check(run, t) {
    type A = { value: { found: number; savedAt: number; answeredAt: number; away: number; saves: number } };
    const first = run.json<A>("first.json").value;
    t.equal(first.found, 0, "the first launch found no save");
    t.equal(first.saves, 1, "and saved");
    t.equal(run.then.length, 1, "the second process ran");
    const second = run.then[0]!.json<A>("second.json").value;
    t.equal(second.found, 1, "the second launch found the first's save in the kept store");
    t.near(second.savedAt, T + 500, 1, "saved at the first host's time (0.5 s in)");
    t.near(second.answeredAt, T + 7200000, 100, "answered at the second host's time (two hours on)");
    t.near(second.away, 7200 - 0.5, 0.1, "away two hours, less the half second before the save");
  },
});
