// The trace test's game (tests/trace.test.ts): the lanes pile of 1,000
// boxes, drawn, so a traced run has systems, render passes, GPU passes
// where the device times them, and the physics step on the lanes' pool
// threads (og-physics goes to the pool from 500 bodies on two to four
// lanes; the test runs with OG_LANES=4), plus one system of the game's own.
import { defineSystem, og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { pile } from "../../support/lanes-pile-scene.ts";

export default og.game({
  seed: 1,
  start(world: World) {
    pile(world, 2500, true);
  },
  systems: [
    defineSystem("Busy", { reads: [og.Transform] }, () => {
      // Some work of its own, so the span is not empty.
      let x = 0;
      for (let i = 0; i < 20_000; i++) x = (x * 31 + i) % 1_000_003;
      if (x < 0) throw new Error("never");
    }),
  ],
});
