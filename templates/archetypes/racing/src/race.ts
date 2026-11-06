// The race's clock and the field's standing: the countdown's beeps and GO
// (the cars stand until then: src/car.ts Drive), each car's place (the
// finished in the order they crossed the line, then the rest by distance
// run), the first finish, and the race's close: over for every seat once
// each has finished, or Tuning.closeAfter after the winner crossed the line,
// when a car still running is out (DNF). src/ui.ts shows the results
// RESULTS_DELAY after the close.

import { defineSystem } from "@origingame/engine";
import { Car, COUNTDOWN, Game, Phase, Sound, Tuning } from "./components.ts";
import { tuning } from "./car.ts";
import { must } from "./world.ts";

/** A countdown second's start, in race ticks (3, 2, 1, then GO at COUNTDOWN). */
export const BEEPS = [0, 60, 120] as const;

export const Race = defineSystem("Race", { reads: [Tuning], writes: [Game, Car], after: ["Drive"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.phase === Phase.Title || g.held) return;
  if ((BEEPS as readonly number[]).includes(g.clock)) g.sfx |= Sound.Beep;
  if (g.clock === COUNTDOWN) g.sfx |= Sound.Go;
  // The field's order: finished first (by when), then by distance run, then by grid slot.
  const field: { c: Record<string, { [i: number]: number }>; i: number }[] = [];
  for (const a of q.archetypes()) {
    const c = a.column(Car) as unknown as Record<string, { [i: number]: number }>;
    for (let i = 0; i < a.count; i++) field.push({ c, i });
  }
  const key = (r: (typeof field)[number]) => ({ finish: r.c.finish![r.i]!, progress: r.c.progress![r.i]!, index: r.c.index![r.i]! });
  field.sort((a, b) => {
    const x = key(a);
    const y = key(b);
    if (x.finish > 0 || y.finish > 0) {
      if (x.finish === 0) return 1;
      if (y.finish === 0) return -1;
      if (x.finish !== y.finish) return x.finish - y.finish;
    }
    if (x.progress !== y.progress) return y.progress - x.progress;
    return x.index - y.index;
  });
  let seats = 0;
  let seatsDone = 0;
  field.forEach((r, n) => {
    r.c.place![r.i] = n + 1;
    const finish = r.c.finish![r.i]!;
    if (finish > 0 && (g.firstFinish === 0 || finish < g.firstFinish)) g.firstFinish = finish;
    if (r.c.seat![r.i]! >= 0) {
      seats++;
      if (finish > 0) seatsDone++;
    }
  });
  if (!g.closed && (seatsDone === seats || (g.firstFinish > 0 && g.clock - g.firstFinish >= Math.round(tuning(world).closeAfter * 60)))) {
    g.closed = true;
    g.lastSeatFinish = g.clock;
    for (const r of field) if (r.c.finish![r.i]! === 0 && r.c.seat![r.i]! >= 0) r.c.out![r.i] = 1;
  }
  g.clock += 1;
});
