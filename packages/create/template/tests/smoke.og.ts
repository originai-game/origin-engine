import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("the game starts", async () => {
  const run = await runPlaytest({ commands: "0.1:quit" });
  expect(run.quit, "the game reached quit").toBe(true);
});
