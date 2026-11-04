// The puzzle kit's fixtures: a world's levels and one level's goals in the
// shapes the kit's components take, for the review sheet (sheet/), the
// skill's examples and a game's first draft. Plain data: copy and edit it.

import type { GoalChipsDesc, SagaMapDesc } from "../types.ts";

export function saga(): SagaMapDesc {
  const stars = [3, 3, 2, 3, 1, 2, 0];
  return {
    label: "Sugar Cove", levelLabel: "Level", bossLabel: "Boss", starsLabel: "{n} of {max} stars", lockedLabel: "Locked", currentLabel: "You are here",
    current: "l7",
    nodes: Array.from({ length: 12 }, (_, i) => ({
      key: `l${i + 1}`, label: String(i + 1), stars: stars[i] ?? 0, maxStars: 3, locked: i > 6, boss: i === 9, lane: [0, 1, 2, 1, 0, -1, -2, -1, 0, 1, 2, 1][i]!,
    })),
  };
}

export function goals(done = false): GoalChipsDesc {
  return {
    label: "Goals", progressLabel: "{have} of {need}", doneLabel: "Done",
    goals: [
      { key: "berry", label: "Berries", icon: "cherry", have: done ? 30 : 18, need: 30 },
      { key: "ice", label: "Ice tiles", icon: "snowflake", have: 12, need: 12 },
      { key: "bomb", label: "Bombs", icon: "bomb", have: 1, need: 4 },
    ],
  };
}
