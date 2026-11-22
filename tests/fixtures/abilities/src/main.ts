// A hero with two abilities and three stats (README.md): a dash (KeyK) of
// two charges, each back 60 ticks after it went, 12 ticks apart; a fireball
// (KeyJ) of one charge, back after 90, that costs 20 of 50 mana. The probe's
// commands apply statuses: poison (stack, at most 3, -2 armour a stack), a
// slow (refresh, -50% speed) and a shield (strongest, + its strength in
// armour). Every tick the Check system recomputes armour and speed from the
// statuses it sees and holds the stats to them.
//
//   og run tests/fixtures/abilities --headless --commands "2:dump_world;3:quit"
import { defineResource, defineSystem, f32, og, u32 } from "@origingame/engine";
import type { Entity, Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export const Seen = defineResource("AbilitySeen", {
  dashUses: u32(),
  fireUses: u32(),
  dashCharges: u32(),
  mana: f32(),
  armour: f32(),
  speed: f32(),
  poisonStacks: u32(),
  shield: f32(),
  slowed: u32(),
  statuses: u32(),
});

const find = (world: World, name: string) => world.find(name) as Entity;

/** The hero's applied statuses, by kind. */
function statuses(world: World): Map<string, { stacks: number; strength: number }> {
  const out = new Map<string, { stacks: number; strength: number }>();
  for (const c of world.children(find(world, "hero"))) {
    const s = world.get(c, og.Status);
    if (s.ok && s.value.applied) out.set(s.value.kind, { stacks: s.value.stacks, strength: s.value.strength });
  }
  return out;
}

export function facts(world: World): Record<string, number> {
  const f = world.resource(Seen);
  if (!f.ok) return {};
  const v = f.value;
  return {
    dashUses: v.dashUses, fireUses: v.fireUses, dashCharges: v.dashCharges, mana: v.mana, armour: v.armour,
    speed: v.speed, poisonStacks: v.poisonStacks, shield: v.shield, slowed: v.slowed, statuses: v.statuses,
  };
}

const Check = defineSystem(
  "Check",
  { reads: [og.AbilityUsed, og.Ability, og.Stat, og.Status, og.Parent], writes: [Seen] },
  (_q, _time, world) => {
    const f = must(world.resource(Seen));
    for (const e of world.events(og.AbilityUsed)) {
      if (e.ability === find(world, "dash")) f.dashUses += 1;
      if (e.ability === find(world, "fireball")) f.fireUses += 1;
    }
    const dash = must(world.get(find(world, "dash"), og.Ability));
    og.assert.always(dash.charges <= dash.maxCharges, "charges never pass the most", { charges: dash.charges });
    f.dashCharges = dash.charges;
    const stat = (name: string) => must(world.get(find(world, name), og.Stat)).value;
    f.mana = stat("mana");
    og.assert.always(f.mana >= 0, "mana is never spent past 0", { mana: f.mana });
    const on = statuses(world);
    f.poisonStacks = on.get("poison")?.stacks ?? 0;
    f.shield = on.get("shield")?.strength ?? 0;
    f.slowed = on.has("slow") ? 1 : 0;
    f.statuses = on.size;
    og.assert.always(f.poisonStacks <= 3, "poison stacks to 3 at most", { stacks: f.poisonStacks });
    f.armour = stat("armour");
    f.speed = stat("speed");
    // The stats as the statuses on the hero make them: adds, then percents.
    og.assert.always(f.armour === 10 - 2 * f.poisonStacks + f.shield, "armour is its base and the statuses' adds", { f: facts(world) });
    og.assert.always(f.speed === (5 + 1) * (1 - 0.5 * f.slowed), "speed is its base, the boots, then the slow", { f: facts(world) });
    og.assert.sometimes(f.dashUses === 2 && f.dashCharges === 0, "both dashes spent");
  },
);

function status(world: World, kind: string, desc: object, stat: string, op: "add" | "percent", amount: number): void {
  const s = must(world.spawn({ parent: find(world, "hero"), bare: true }));
  must(world.insert(s, og.Status, { kind, stacks: 1, ...desc }));
  const m = must(world.spawn({ parent: s, bare: true }));
  must(world.insert(m, og.Modifier, { stat: find(world, stat), op, amount }));
}

export default og.game({
  tick: 60,
  seed: 3,
  resources: [Seen],
  systems: [...og.systems.abilities, Check],
  start(world) {
    must(og.input.defineAction(world, { name: "dash", bindings: [{ control: "KeyK" }] }));
    must(og.input.defineAction(world, { name: "fire", bindings: [{ control: "KeyJ" }] }));
    const hero = must(world.spawn({ name: "hero" }));
    const stat = (name: string, base: number) => {
      const e = must(world.spawn({ name, parent: hero, bare: true }));
      must(world.insert(e, og.Stat, { base }));
      return e;
    };
    const mana = stat("mana", 50);
    stat("armour", 10);
    const speed = stat("speed", 5);
    // Boots: +1 speed, with no status, so they last.
    const boots = must(world.spawn({ name: "boots", parent: hero, bare: true }));
    must(world.insert(boots, og.Modifier, { stat: speed, op: "add", amount: 1 }));
    const dash = must(world.spawn({ name: "dash", parent: hero, bare: true }));
    must(world.insert(dash, og.Ability, { owner: hero, action: "dash", maxCharges: 2, charges: 2, rechargeTicks: 60, lockTicks: 12 }));
    const fire = must(world.spawn({ name: "fireball", parent: hero, bare: true }));
    must(world.insert(fire, og.Ability, { owner: hero, action: "fire", maxCharges: 1, charges: 1, rechargeTicks: 90, pool: mana, cost: 20 }));
  },
  commands: {
    poison: (world) => status(world, "poison", { stacking: "stack", ticks: 120, maxStacks: 3 }, "armour", "add", -2),
    slow: (world) => status(world, "slow", { stacking: "refresh", ticks: 60 }, "speed", "percent", -0.5),
    shield: (world, args) => {
      const strength = Number(args[0]);
      status(world, "shield", { stacking: "strongest", ticks: 0, strength }, "armour", "add", strength);
    },
  },
  facts,
});
