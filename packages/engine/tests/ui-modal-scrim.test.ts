// A dialog closes for a pointer and a finger as it does for Escape
// (Brasswick's finding #10, first found on the level select), on the native
// host: a tap on the scrim around og.ui.modal answers `dismissed`, and a
// locked one ignores it.
//
// Ways it could fail: a tap on the scrim that does nothing; a locked dialog
// closed by a stray tap.
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { dump, engineFor } from "./support/ui.ts";

beforeAll(() => {
  loadNativeHost();
});

for (const locked of [false, true]) {
  test(`og.ui.modal: a tap on the scrim around it ${locked ? "leaves a locked one open" : "answers dismissed"}`, () => {
    let dismissed = 0;
    const engine = engineFor(`UmScrim${locked ? "Locked" : ""}`, () => {
      const r = og.ui.modal("Levels", true, { title: "Niveles", locked }, () => og.ui.label("Body", "1. La escalera de la caldera"));
      if (r.dismissed) dismissed += 1;
    });
    engine.advance(30);
    const panel = dump(engine).find((e) => e.id === "Panel" && e.parent === "Levels")!;
    expect(panel.x).toBeGreaterThan(40);
    // Left of the panel, halfway down: the scrim.
    engine.probe.commands(`0:touch_move:${Math.floor(panel.x / 2)},360;0.05:touch_start;0.1:touch_end`);
    engine.advance(20);
    expect(dismissed > 0).toBe(!locked);
  });
}
