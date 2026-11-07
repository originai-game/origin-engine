// Corpus 03-exact-optional: an optional property given undefined under exactOptionalPropertyTypes
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("n", async () => { const run = await runPlaytest({ commands: "0.1:quit", seed: undefined }); expect(run.quit).toBe(true); });
