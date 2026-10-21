// Three frames at 1280x720 compared with references/<tier>/: the dealt
// board (the nine-sliced panel, the checker tilemap, the gems' idle clips),
// the swap's line mid-clear (the burst clip, the squash to nothing, the
// floating +30) and the cascade mid-clear (the gold crit +60 and the
// camera shake). Every motion is a tween or a flipbook clip on scaled time
// and the shake is seeded by the tick, so the same script draws the same
// frames. Record a tier's references with `OG_FX_CAPTURE=1 og test .`.
import { playtest } from "@origingame/engine/playtest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { checkFrames, decodePng, outDir } from "./frames.ts";
import { CASCADE_BOARD, PHASE, matchOf, tap } from "./lib.ts";

const FILES = ["dealt.png", "clearing.png", "cascade.png"];

export default playtest({
  name: "the dealt board, a clear and a cascade match their reference frames",
  size: "1280x720",
  commands: [
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    "1.2:screenshot:dealt.png",
    "1.3:swap:2,0:2,1",
    "1.53:screenshot:clearing.png",
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
    // The calm ground: away from the board the frame is the brand ink #14161b.
    t.frame("dealt.png", () => {
      const dealt = decodePng(readFileSync(join(outDir(), "dealt.png")));
      const at = (700 * dealt.width + 20) * 4;
      const ink = [dealt.rgba[at]!, dealt.rgba[at + 1]!, dealt.rgba[at + 2]!];
      t.ok(Math.abs(ink[0]! - 0x14) <= 3 && Math.abs(ink[1]! - 0x16) <= 3 && Math.abs(ink[2]! - 0x1b) <= 3, `the corner is the brand ink (${ink.join(", ")})`);
    });
    checkFrames(run, t, FILES, "");
  },
});
