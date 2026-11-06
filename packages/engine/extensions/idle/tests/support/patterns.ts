import { idle, type IdleGeneratorResult } from "@origingame/engine/extensions/idle/index.ts";

import { run } from "../../../../tests/support/patterns.ts";

// ── idle ────────────────────────────────────────────────────────────────

export function generator(affordable: () => boolean) {
  const results: IdleGeneratorResult[] = [];
  const engine = run(() => {
    results.push(idle.idleGenerator("Solar", { name: "Solar Array", level: 125, ownedText: "348", rateText: "+1.24 M / sec", cycle: 0.4, cycleSeconds: 2, costText: "12.4 M", affordable: affordable() }));
  });
  return { engine, results, bought: () => results.filter((r) => r.bought).length };
}
