import { og, defineSystem } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { createEngine, type Engine, type GameToken } from "../../../../src/runtime.ts";

import { strategy, type RtsCommandsResult, type LedgerLine, type LedgerResult } from "@origingame/engine/extensions/strategy/index.ts";

import { run, systemIds, ACT } from "../../../../tests/support/patterns.ts";

// ── ledger and overlay bar ──────────────────────────────────────────────

export const CITY: LedgerLine[] = [
  { key: "res", section: "in", name: "Residential", amount: 5200, previous: 4800, funding: 9 },
  { key: "com", section: "in", name: "Commercial", amount: 3100, previous: 3400, funding: 12 },
  { key: "tour", section: "in", name: "Tourism", amount: 900, previous: 600 },
  { key: "police", section: "out", name: "Police", amount: 1800, previous: 1700, funding: 100 },
  { key: "fire", section: "out", name: "Fire", amount: 1200, previous: 1250, funding: 90, locked: true },
  { key: "parks", section: "out", name: "Parks", amount: 400, previous: 300, funding: 110 },
];

export function cityLedger(st: { sort: string; descending: boolean }, results: LedgerResult[]): Engine {
  return run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = strategy.ledger("Budget", {
        sections: [{ key: "in", name: "Income" }, { key: "out", name: "Expenses", expense: true }],
        lines: CITY.map((l) => (l.section === "in" ? { ...l, funding: -1 } : l)),
        sort: st.sort,
        descending: st.descending,
        fundingMin: 50,
        fundingMax: 150,
      });
      results.push(r);
      if (r.sort !== null) {
        st.sort = r.sort;
        st.descending = r.descending;
      }
    });
  });
}

export function rtsRun(st: { page: string; targeting: string }, results: RtsCommandsResult[]): Engine {
  const sys = defineSystem(`RtsCmd${systemIds.next++}`, {}, (_q: unknown, _t: unknown, world: Api.World) => {
    for (const a of [ACT("cmdAttack", ["KeyA"]), ACT("cmdStorm", ["KeyS"]), ACT("group1", ["Digit1"]), ACT("group2", ["Digit2"])]) og.ui.defineAction(world, a);
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column" } as never, () => {
      const r = strategy.rtsCommands("Rts", {
        console: {
          selection: { units: [{ key: "u1", name: "Marine", health: 40, healthMax: 45 }, { key: "u2", name: "Marine", health: 45, healthMax: 45 }], groups: [{ key: "g1", label: "1", count: 2 }, { key: "g2", label: "2", count: 0 }], group: "g1" },
          commands: { columns: 5 },
          anchor: "bottom",
        },
        pages: [
          { key: "root", commands: [{ key: "attack", name: "Attack", action: "cmdAttack" }, { key: "build", name: "Build" }, { key: "storm", name: "Storm", action: "cmdStorm", locked: true }] },
          { key: "build", commands: [{ key: "barracks", name: "Barracks" }] },
        ],
        page: st.page,
        groupActions: ["group1", "group2"],
        targeting: st.targeting,
      });
      results.push(r);
      if (r.page !== null) st.page = r.page;
    });
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  r.value.advance(10);
  return r.value;
}
