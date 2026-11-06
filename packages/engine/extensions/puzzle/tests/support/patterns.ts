import { og } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { puzzle, type SagaMapResult, type SagaNode } from "@origingame/engine/extensions/puzzle/index.ts";

import { run } from "../../../../tests/support/patterns.ts";

// ── saga map ────────────────────────────────────────────────────────────

export const LEVELS: SagaNode[] = Array.from({ length: 30 }, (_, i) => ({
  key: `l${i + 1}`, label: `${i + 1}`, stars: i < 20 ? 3 - (i % 3) : 0, locked: i > 20, boss: i === 24,
}));

export function saga(viewport?: Partial<Api.Viewport>) {
  const results: SagaMapResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: viewport ? 360 : 1100 }, height: { kind: "fixed", px: viewport ? 640 : 520 } } as never, () => {
      results.push(puzzle.sagaMap("Saga", { nodes: LEVELS, current: "l21" }));
    });
  }, viewport);
  engine.advance(6);
  return { engine, results };
}
