// The same two processes without `store: "keep"`: each has its own store,
// so the second finds nothing.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("without a kept store a relaunch finds no save", async () => {
  const run = await runPlaytest({ commands: "0.5:save;0.8:quit", then: [{ commands: "0.3:get:Away:second.json;0.4:quit" }] });
  const second = run.then[0]!.json<{ value: { loaded: number; found: number } }>("second.json").value;
  expect(second.loaded, "the second launch was answered").toBe(1);
  expect(second.found, "and found no save").toBe(0);
});
