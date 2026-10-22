// Corpus 03-exact-optional: exactOptionalPropertyTypes, undefined for an optional field
import { playtest } from "@origingame/engine/playtest";
export default playtest({ name: "n", commands: "0.1:quit", seed: undefined, check(run, t) { t.ok(run.quit, "quit"); } });
