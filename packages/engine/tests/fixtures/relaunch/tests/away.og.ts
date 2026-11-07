// Two processes, one kept store: the first saves at its host time, the
// second launches two hours later by its clock and reads the time away.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

const T = 1767225600000;
type A = { value: { found: number; savedAt: number; answeredAt: number; away: number; saves: number } };

/** `actual` within `tolerance` of `expected`, both sides inclusive (an absolute tolerance). */
function near(actual: number, expected: number, tolerance: number, what: string): void {
  expect(Math.abs(actual - expected), `${what}: ${actual} is not within ${tolerance} of ${expected}`).toBeLessThanOrEqual(tolerance);
}

test("a relaunch two hours later reads the time away from the kept store", async () => {
  const run = await runPlaytest({
    store: "keep",
    clock: T,
    commands: "0.5:save;0.7:get:Away:first.json;0.8:quit",
    then: [{ commands: "0.3:get:Away:second.json;0.4:quit", clock: T + 2 * 3600 * 1000 }],
  });
  const first = run.json<A>("first.json").value;
  expect(first.found, "the first launch found no save").toBe(0);
  expect(first.saves, "and saved").toBe(1);
  expect(run.then, "the second process ran").toHaveLength(1);
  const second = run.then[0]!.json<A>("second.json").value;
  expect(second.found, "the second launch found the first's save in the kept store").toBe(1);
  near(second.savedAt, T + 500, 1, "saved at the first host's time (0.5 s in)");
  near(second.answeredAt, T + 7200000, 100, "answered at the second host's time (two hours on)");
  near(second.away, 7200 - 0.5, 0.1, "away two hours, less the half second before the save");
});
