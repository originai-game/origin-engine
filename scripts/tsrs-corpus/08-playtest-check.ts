// Corpus 08-playtest-check: t.ok without its message
import { playtest } from "@origingame/engine/playtest";
export default playtest({ name: "n", commands: "0.1:quit", check(run, t) { t.ok(run.quit); } });
