import { og, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";

function must<T>(result: Result<T>): T {
  if (!result.ok) throw new Error(result.reason);
  return result.value;
}

// Keep the resource and command names consumed by rooms.test.ts and room-then.
const Duel = defineResource("Duel", {
  sum0: f32(), sum1: f32(),
  droppedAt: u32(), drops: u32(), rejoinedAt: u32(), rejoins: u32(),
  // The scripted bot's switch (`bot:<n>`), world state so a restore or a rollback keeps it.
  bot: u32(),
});

const Submit = defineSystem("Submit", { reads: [og.Lockstep, Duel] }, (_q, time, world) => {
  const lock = must(world.resource(og.Lockstep));
  const bot = must(world.resource(Duel)).bot !== 0;
  const seat = lock.localPlayer;
  const phase = (time.tick + seat * 17) % 50;
  const input = !bot || time.tick < 60
    ? { x: 0, y: 0, buttons: 0 }
    : { x: phase < 21 ? 0.75 : -0.25, y: seat === 0 ? 0.5 : -0.75, buttons: phase === 0 ? 1 : 0 };
  must(og.net.submit(world, input));
});

const Apply = defineSystem("Apply", { reads: [og.PlayerInput], writes: [Duel], after: ["Submit"] }, (q, _time, world) => {
  const tally = must(world.resource(Duel));
  for (const a of q.archetypes()) {
    const input = a.column(og.PlayerInput);
    for (let i = 0; i < a.count; i++) {
      const value = input.x[i]! + 2 * input.y[i]! + input.buttons[i]!;
      if (input.player[i] === 0) tally.sum0 += value;
      else tally.sum1 += value;
    }
  }
  og.assert.always(Number.isFinite(tally.sum0) && Number.isFinite(tally.sum1), "both seats accumulate finite inputs");
  og.assert.sometimes(tally.sum0 !== 0 && tally.sum1 !== 0, "both seats supply inputs");
});

const Seats = defineSystem("Seats", { reads: [og.SeatDropped, og.SeatRejoined], writes: [Duel] }, (_q, _time, world) => {
  const tally = must(world.resource(Duel));
  for (const event of world.events(og.SeatDropped)) {
    tally.droppedAt = event.tick;
    tally.drops++;
  }
  for (const event of world.events(og.SeatRejoined)) {
    tally.rejoinedAt = event.tick;
    tally.rejoins++;
  }
});

function start(world: World): void {
  const lock = must(world.resource(og.Lockstep));
  lock.enabled = true;
  lock.players = 2;
  lock.inputDelayTicks = 2;
  lock.hashEveryTicks = 10;
  for (const player of [0, 1]) {
    must(world.insert(must(world.spawn({ name: `seat-${player}` })), og.PlayerInput, { player }));
  }
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Duel],
  systems: [Submit, Apply, Seats],
  start,
  commands: {
    bot: (world, args) => { must(world.resource(Duel)).bot = Number(args[0]) > 0 ? 1 : 0; },
    rollback: (world, args) => {
      const lock = must(world.resource(og.Lockstep));
      lock.mode = "rollback";
      lock.maxPredictionTicks = Number(args[0] ?? 8);
    },
    stall: (world, args) => { must(world.resource(og.Lockstep)).stallTimeoutMs = Number(args[0]); },
    quick: (world, args) => {
      must(og.platform.quickPlay(world, { mode: args[0] || "duel", players: 2, timeoutMs: Number(args[1] ?? 30000) }));
    },
  },
  facts: (world) => {
    const tally = must(world.resource(Duel));
    return { sum0: tally.sum0, sum1: tally.sum1, droppedAt: tally.droppedAt, drops: tally.drops, rejoinedAt: tally.rejoinedAt, rejoins: tally.rejoins };
  },
});
