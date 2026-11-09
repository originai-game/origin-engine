// A game's own input shape over a room (og.net.shape, rooms.test.ts): each
// seat's input is a Cursor (in place of og.PlayerInput) and its orders are
// commands, one Order record each, that act on one tick only. Each peer's
// bot gives two orders every 20 ticks from tick 100 to 399 (30 per seat),
// by the tick alone, so a lockstep and a rollback session play the same
// inputs; Apply counts each order once, by the seat that gave it.
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The seat's input: a cursor the engine writes every tick, as og.PlayerInput. */
const Cursor = defineComponent("Cursor", { player: u32(), tick: u32(), x: f32(), y: f32() });
/** One order: move `unit` to (x, y). `player` is the seat that gave it. */
const Order = defineComponent("Order", { player: u32(), unit: u32(), x: f32(), y: f32() });
/** A unit, owned by a seat, walking to its target. */
const Unit = defineComponent("Unit", { owner: u32(), index: u32(), x: f32(), y: f32(), tx: f32(), ty: f32() });
const Tally = defineResource("Tally", { orders0: u32(), orders1: u32(), lastTick: u32(), cursor: f32() });

const Give = defineSystem("Give", { reads: [og.Lockstep] }, (_q, time, world) => {
  const lock = must(world.resource(og.Lockstep));
  if (!lock.running) return;
  const seat = lock.localPlayer;
  const t = time.tick;
  // Still for the first 60 ticks, so any session that starts before then plays the same inputs.
  if (t < 60) return;
  must(og.net.submitInput(world, Cursor, { x: (t % 13) + seat, y: seat * 2 }));
  if (t >= 100 && t < 400 && t % 20 === 0) {
    for (const k of [0, 1]) must(og.net.command(world, Order, { unit: (t / 20 + k) % 3, x: seat * 10 + (t % 7), y: k + (t % 11) }));
  }
});

const Steer = defineSystem("Steer", { reads: [Cursor], writes: [Tally] }, (q, _time, world) => {
  const tally = must(world.resource(Tally));
  for (const a of q.archetypes()) {
    const c = a.column(Cursor);
    for (let i = 0; i < a.count; i++) tally.cursor += c.x[i]! * (c.player[i]! + 1);
  }
});

const Apply = defineSystem("Apply", { writes: [Unit, Tally] }, (q, time, world) => {
  const tally = must(world.resource(Tally));
  for (const o of must(og.net.commands(world, Order))) {
    if (o.player === 0) tally.orders0++;
    else tally.orders1++;
    tally.lastTick = time.tick;
    for (const a of q.archetypes()) {
      const u = a.column(Unit);
      for (let i = 0; i < a.count; i++) if (u.owner[i] === o.player && u.index[i] === o.unit) {
        u.tx[i] = o.x;
        u.ty[i] = o.y;
      }
    }
  }
  for (const a of q.archetypes()) {
    const u = a.column(Unit);
    for (let i = 0; i < a.count; i++) {
      u.x[i] += (u.tx[i]! - u.x[i]!) * 0.125;
      u.y[i] += (u.ty[i]! - u.y[i]!) * 0.125;
    }
  }
});

function start(world: World): void {
  must(og.net.shape(world, { input: Cursor, command: Order, maxCommands: 4 }));
  const lock = must(world.resource(og.Lockstep));
  lock.enabled = true;
  lock.players = 2;
  lock.hashEvery = 10;
  for (const player of [0, 1]) {
    must(world.insert(must(world.spawn({ name: `cursor-${player}` })), Cursor, { player }));
    for (const index of [0, 1, 2]) must(world.insert(must(world.spawn({ name: `unit-${player}-${index}` })), Unit, { owner: player, index }));
  }
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Tally],
  systems: [Give, Steer, Apply],
  start,
  commands: {
    rollback: (world, args) => {
      const lock = must(world.resource(og.Lockstep));
      lock.mode = "rollback";
      lock.maxPrediction = Number(args[0] ?? 8);
    },
  },
  facts: (world) => {
    const t = must(world.resource(Tally));
    return { orders0: t.orders0, orders1: t.orders1, lastTick: t.lastTick, cursor: t.cursor };
  },
});
