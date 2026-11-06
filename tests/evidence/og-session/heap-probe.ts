// Baseline limitation probe: this deliberately keeps state outside the world.
// bun tests/evidence/og-session/heap-probe.ts (no renderer required).
import { og, defineResource, defineSystem, i32 } from "../../../packages/engine/src/index.ts";
import { createEngine } from "../../../packages/engine/src/runtime.ts";
import { loadNativeHost } from "../../../packages/engine/src/node/index.ts";

loadNativeHost();
let heapCounter = 0;
const Counter = defineResource("SessionHeapProbe", { value: i32() });
const Count = defineSystem("Count", { writes: [Counter] }, (_q, _time, world) => {
  const value = world.resource(Counter);
  if (!value.ok) throw new Error(value.reason);
  value.value.value = ++heapCounter;
});
const made = createEngine(og.game({ resources: [Counter], systems: [Count] }));
if (!made.ok) throw new Error(made.reason);
const engine = made.value;
const before = engine.probe.snapshot();
if (!before) throw new Error("snapshot refused");
const first = engine.advance(1);
if (!first.ok) throw new Error(first.reason);
const firstWorld = engine.probe.snapshot();
if (!engine.probe.restore(before.id)) throw new Error("restore refused");
const restored = engine.probe.snapshot();
const second = engine.advance(1);
if (!second.ok) throw new Error(second.reason);
const secondWorld = engine.probe.snapshot();
if (!firstWorld || !restored || !secondWorld) throw new Error("snapshot refused");
const result = {
  initialHash: before.hash,
  restoredHash: restored.hash,
  firstTickHash: firstWorld.hash,
  repeatedTickHash: secondWorld.hash,
  heapCounter,
  restoresWorld: before.hash === restored.hash,
  restoresHeap: firstWorld.hash === secondWorld.hash,
};
engine.destroy();
process.stdout.write(JSON.stringify(result) + "\n");
if (!result.restoresWorld || result.restoresHeap || heapCounter !== 2) {
  throw new Error("baseline behavior changed: reassess snapshot coverage");
}
