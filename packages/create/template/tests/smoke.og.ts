import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "the game starts",
  commands: "0.1:quit",
  check: (run, t) => t.ok(run.quit, "the game reached quit"),
});
