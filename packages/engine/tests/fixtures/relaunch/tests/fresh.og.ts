// The same two processes without `store: "keep"`: each has its own store,
// so the second finds nothing.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "without a kept store a relaunch finds no save",
  commands: "0.5:save;0.8:quit",
  then: [{ commands: "0.3:get:Away:second.json;0.4:quit" }],
  check(run, t) {
    const second = run.then[0]!.json<{ value: { loaded: number; found: number } }>("second.json").value;
    t.equal(second.loaded, 1, "the second launch was answered");
    t.equal(second.found, 0, "and found no save");
  },
});
