// Corpus 08-playtest-check: a matcher without its expected value
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("n", async () => { const run = await runPlaytest({ commands: "0.1:quit" }); expect(run.quit).toBe(); });
