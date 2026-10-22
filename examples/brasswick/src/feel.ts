// The juice, as requests other systems make and two systems answer: the
// effects' emitters moved and restarted (Effects, one emitter per kind,
// og-game §10: a one-shot effect restarts by its startTick), and words from
// the string tables (say).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Fx, FxKind, FxSlot, Game } from "./components.ts";

type FxV = Value<typeof Fx>;

/** A word from the string tables (og-i18n §2). */
export function say(world: World, key: string, args?: Record<string, unknown>): string {
  return og.i18n.t(world, key, args);
}

/** Ask for an effect at a point this tick. */
export function effect(fx: FxV, kind: number, x: number, y: number, tick: number): void {
  if (kind === FxKind.Dust) {
    fx.dustX = x;
    fx.dustY = y;
    fx.dustTick = tick;
  } else if (kind === FxKind.Gear) {
    fx.gearX = x;
    fx.gearY = y;
    fx.gearTick = tick;
  } else {
    fx.stompX = x;
    fx.stompY = y;
    fx.stompTick = tick;
  }
}

/** Moves each kind's emitter to where it was asked for this tick and restarts it. */
export const Effects = defineSystem("Effects", { reads: [Fx, FxSlot], writes: [og.ParticleSystem, og.Transform] }, (q, time, world) => {
  const fx = world.resource(Fx);
  if (!fx.ok) return;
  const f = fx.value;
  for (const a of q.archetypes()) {
    const kinds = a.column(FxSlot).kind;
    const pos = a.column(og.Transform).position;
    const start = a.column(og.ParticleSystem).startTick;
    const visible = a.column(og.ParticleSystem).visible;
    for (let i = 0; i < a.count; i++) {
      const k = kinds[i]!;
      const [x, y, t] = k === FxKind.Dust ? [f.dustX, f.dustY, f.dustTick] : k === FxKind.Gear ? [f.gearX, f.gearY, f.gearTick] : [f.stompX, f.stompY, f.stompTick];
      if (t !== time.tick) continue;
      pos[i * 3] = x;
      pos[i * 3 + 1] = y;
      start[i] = time.tick;
      visible[i] = 1;
    }
  }
});

/** Last in the tick: a level's reset has reached every system that resets its things. */
export const Settle = defineSystem("Settle", { writes: [Game] }, (_q, _time, world) => {
  const g = world.resource(Game);
  if (g.ok && g.value.reset !== 0) g.value.reset = 0;
});
