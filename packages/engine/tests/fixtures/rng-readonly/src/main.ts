// The archetypes round's reproduction of og.Rng declared only in `reads`
// (docs/phase-1.md, "Phase 2, archetypes round"): a fork from a read-only
// og.Rng answered NaN, while the same fork from a system that declared it
// in `writes` answered 0.6868…. Both systems fork "x" on tick 1 and keep the
// fork's first draw, the seed and the main stream's state before and after,
// in the Repro resource; tests/core.test.ts reads it on both hosts.
import { og, defineResource, defineSystem, f64, u32 } from "@origingame/engine";

export const Repro = defineResource("Repro", {
  readsSeed: u32(),
  readsDraw: f64(-1),
  readsStateBefore: u32(),
  readsStateAfter: u32(),
  writesSeed: u32(),
  writesDraw: f64(-1),
});

const ReadsOnly = defineSystem("ReadsOnly", { reads: [og.Rng], writes: [Repro] }, (_q, time, world) => {
  if (time.tick !== 1) return;
  const r = world.resource(og.Rng);
  const out = world.resource(Repro);
  if (!r.ok || !out.ok) return;
  out.value.readsSeed = r.value.seed;
  out.value.readsStateBefore = r.value.state;
  out.value.readsDraw = r.value.fork("x").next();
  out.value.readsStateAfter = r.value.state;
});

const Writes = defineSystem("Writes", { writes: [og.Rng, Repro] }, (_q, time, world) => {
  if (time.tick !== 1) return;
  const r = world.resource(og.Rng);
  const out = world.resource(Repro);
  if (!r.ok || !out.ok) return;
  out.value.writesSeed = r.value.seed;
  out.value.writesDraw = r.value.fork("x").next();
});

export default og.game({ tick: 60, seed: 1, resources: [Repro], systems: [ReadsOnly, Writes], start() {} });
