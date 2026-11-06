// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { CASCADE_BOARD, PHASE, matchOf, tap } from "./lib.ts";

const FILES = ["dealt.png", "cascade.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: [
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    "1.2:screenshot:dealt.png",
    "1.3:swap:2,0:2,1",
    "1.53:dump_entity:board:clearing.json",
    "2.14:screenshot:cascade.png",
    "2.14:dump_entity:board:cascade.json",
    "2.2:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const clearing = matchOf(run, "clearing.json");
    t.equal([clearing.phase, clearing.chain], [PHASE.clear, 1], "the first frame of play is the swap's own line clearing");
    const cascade = matchOf(run, "cascade.json");
    t.equal([cascade.phase, cascade.chain], [PHASE.clear, 2], "the last is the cascade clearing");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
