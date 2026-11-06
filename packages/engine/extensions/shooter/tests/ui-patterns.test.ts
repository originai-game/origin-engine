import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { shooter, type LoadoutResult } from "@origingame/engine/extensions/shooter/index.ts";

import { run, tapOn, byPath } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

// ── loadout ─────────────────────────────────────────────────────────────

test("loadout: a locked slot is refused, not selected; a tab answers its index; Remove answers apart from its row; a callout's dot lands on its point of the art as drawn", () => {
  // Wrong: a locked slot selected, a press on Remove also answered as the row's, or a leader ending off the art (the stage's padding or the contain fit not counted).
  const results: LoadoutResult[] = [];
  const st = { slot: "primary" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = shooter.loadout("Loadout", {
        title: "Loadout",
        loadouts: ["One", "Two", "Three"],
        loadout: 0,
        slots: [
          { key: "primary", label: "Primary", name: "Raven", detail: "Lvl 12" },
          { key: "secondary", label: "Secondary", name: "Sidearm" },
          { key: "perk3", label: "Perk 3", name: "Ghost", locked: true },
        ],
        slot: st.slot,
        name: "Raven",
        aspect: 2,
        attachments: [
          { key: "optic", label: "Optic", name: "Holo", x: 0.5, y: 0.2 },
          { key: "barrel", label: "Barrel", name: "Suppressor", x: 0.1, y: 0.5, rarityLabel: "Rare", rarity: "rare" },
        ],
        stats: [{ key: "damage", label: "Damage", value: 32, delta: 4 }],
      });
      results.push(r);
      if (r.slot) st.slot = r.slot;
    });
  });
  engine.advance(10);
  tapOn(engine, byPath(engine, "Slots/perk3")!);
  expect(st.slot).toBe("primary");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["perk3"]);
  tapOn(engine, byPath(engine, "Slots/secondary")!);
  expect(st.slot).toBe("secondary");
  tapOn(engine, byPath(engine, "Tabs/Row/2")!);
  expect(results.filter((r) => r.loadout !== null).map((r) => r.loadout)).toEqual([2]);
  tapOn(engine, byPath(engine, "List/barrel/Remove")!);
  tapOn(engine, byPath(engine, "List/optic")!);
  expect(results.filter((r) => r.remove).map((r) => r.remove)).toEqual(["barrel"]);
  expect(results.filter((r) => r.attachment).map((r) => r.attachment)).toEqual(["optic"]);
  // The art is 2:1, contained in its slot: the optic's point is at its middle across, a fifth down.
  const art = byPath(engine, "Room/Art")!;
  const dw = Math.min(art.width, art.height * 2);
  const dh = dw / 2;
  const dot = byPath(engine, "Callouts/optic~Dot")!;
  expect(Math.abs(dot.x + dot.width / 2 - (art.x + art.width / 2))).toBeLessThan(2);
  expect(Math.abs(dot.y + dot.height / 2 - (art.y + (art.height - dh) / 2 + 0.2 * dh))).toBeLessThan(2);
  const callout = byPath(engine, "Callouts/optic")!;
  expect(callout.y + callout.height).toBeLessThanOrEqual(art.y + (art.height - dh) / 2 + 1);
});
