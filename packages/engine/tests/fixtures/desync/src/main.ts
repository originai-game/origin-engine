// Two peers in lockstep whose one system reads Math.random() into the world
// at tick 100: the shared hashes part there, og.Desync names tick 100, and
// `dump_desync` names Wobble.offset on the entity `wobbler` (rooms.test.ts).
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";

const Wobble = defineComponent("Wobble", { offset: f32() });
/** og.Desync as this peer saw it: the first divergent tick, 0 before. */
const Seen = defineResource("Seen", { tick: u32(), peer: u32() });

const Nudge = defineSystem("Nudge", { writes: [Wobble] }, (q, time) => {
  if (time.tick !== 100) return;
  for (const a of q.archetypes()) {
    const w = a.column(Wobble);
    // The bug under test: a value no other peer draws.
    for (let i = 0; i < a.count; i++) w.offset[i] = Math.random();
  }
});

const Watch = defineSystem("Watch", { reads: [og.Desync], writes: [Seen] }, (_q, _time, world) => {
  const s = world.resource(Seen);
  for (const d of world.events(og.Desync)) if (s.ok && s.value.tick === 0) {
    s.value.tick = d.tick;
    s.value.peer = d.peer;
  }
});

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Seen],
  systems: [Nudge, Watch],
  start(world) {
    const lock = world.resource(og.Lockstep);
    if (!lock.ok) throw new Error(lock.reason);
    lock.value.enabled = true;
    lock.value.players = 2;
    lock.value.hashEvery = 10;
    const e = world.spawn({ name: "wobbler" });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Wobble);
  },
});
