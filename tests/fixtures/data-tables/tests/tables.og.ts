// The tables as `og test` plays them (src/main.ts): rows read through
// handles across both tables, a reload that keeps every handle, a broken
// row refused by its line with the old table kept, and a deleted row that
// og.table.missing names. Two runs end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("rows are read by handle across tables, reload keeps handles, a bad row is refused by line", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["the tables were read"],
    commands: [
      // The cooked tables: the sword, the arrow, the bow's two salvage rows, iron's sword.
      "0.2:expect:state:swordDamage==12",
      "0.2:expect:state:arrowDamage==3",
      "0.2:expect:state:salvage==1",
      "0.2:expect:state:ironMakes==12",
      "0.2:expect:state:iconOk==1",
      "0.2:expect:state:nameOk==1",
      "0.2:expect:state:missing==0",
      // Reordered and renumbered: the held arrow handle reads the new arrow.
      "0.3:reload",
      "0.4:expect:state:sameHandle==1",
      "0.4:expect:state:swordDamage==15",
      "0.4:expect:state:arrowDamage==4",
      // A string in a number field at line 6: refused, the reloaded table kept.
      "0.5:bad",
      "0.6:expect:state:refusals==1",
      "0.6:expect:state:refusedLine==6",
      "0.6:expect:state:swordDamage==15",
      // The arrow row gone: the quiver's handle is what og.table.missing lists.
      "0.7:drop",
      "0.8:expect:state:missing==1",
      "0.85:quit",
    ].join(";"),
  });
  expect(run.hashes.length).toBe(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
