<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `abilities` guide: Abilities: cooldowns, charges, costs, statuses and stats

How og.systems.abilities uses abilities with charges, recharge, a lock and a cost, keeps status effects that stack by a rule and run out, and computes stats from their modifiers in a fixed order. Read before building spells, skills, a dash with charges, buffs and debuffs, damage over time or any stat a bonus changes. Buffered presses are in `input`; hit stop in `og api show time`; a table of ability or status rows in `data`.

The API: `og api show abilities`.

<a id="guide-abilities-1"></a>
### `abilities §1` Abilities: charges, recharge, lock and cost

List the systems in the game:

```ts
og.game({ systems: [...og.systems.abilities, ...mySystems] });
```

- An **og.Ability** is on an entity of its own (a child of its owner, or
  anywhere: `owner` names who uses it). It holds `charges` up to
  `maxCharges`; give its first charges when it is inserted. A plain
  cooldown is one charge: `maxCharges: 1, charges: 1, rechargeTicks: 90`.
- Below maxCharges a charge returns every `rechargeTicks`, one at a time;
  a full ability counts nothing. `lockTicks` keeps it from being used again
  that long after a use whatever its charges (the gap between two dashes).
  Both count the ticks time moves: a hit stop or a time scale of 0 holds
  them. Used on tick T, an ability of `rechargeTicks` R has its charge back
  on tick T + R, in og.abilities' run, before the game's systems listed
  after it.
- A use takes `cost` from its `pool`, an og.Stat's base (mana, stamina,
  ammunition): a pool whose value is under the cost refuses it.
- A press of `action` uses it, read through og.input.consume only when it
  can be used, so a press a few ticks early waits in the action's buffer.
  og.abilities.use uses it from code (an AI, a scripted cast) and refuses,
  changing nothing, naming why; og.abilities.ready asks. Either sends
  og.AbilityUsed: what the ability does (a projectile, a dash, a heal) is
  the game's, read from it in a system.

Typical numbers at 60 Hz: a basic attack's lock of 20 to 40 ticks, a
dodge's of 15 to 30, a spell's recharge of 3 to 20 s, two or three charges
for a dash. These are starting points to tune by feel.

<a id="guide-abilities-2"></a>
### `abilities §2` Statuses and stats

- An **og.Stat** is a number on an entity of its own: `base` and the
  written `value`. An **og.Modifier** on an entity of its own names its
  `stat` and changes it by `op`: `add` adds its amount; `percent` amounts
  are summed and the value is times 1 + the sum; `multiply` amounts each
  multiply by 1 + theirs. The order is always adds, then percents, then
  multiplies, and each kind's amounts in ascending order, so the order
  modifiers were spawned in never changes a value. Additive percents
  (+10% and +20% make +30%) and multiplicative ones (+32%) are the game's
  choice of op.
- An **og.Status** is spawned as a child of its target to apply it; its
  modifiers are its children, and each counts its amount once per stack.
  A second status of the same `kind` on the same target merges into the
  first by the first's `stacking`:
  - `refresh`: the first takes the second's ticks (a slow re-applied);
  - `stack`: the first adds the second's stacks up to `maxStacks` and takes
    its ticks (poison that builds);
  - `strongest`: the stronger by `strength` stays, its modifiers with it; a
    tie keeps the first with the second's ticks (two shields).
- `ticks` counts down each tick time moves, the tick it was applied
  counting; at 0 the status and its modifiers are despawned. A status of 0
  ticks lasts until the game despawns it (an aura, gear).
- A stat's value is written in og.abilities' run each tick: a modifier
  spawned in a system is counted the tick after.

<a id="guide-abilities-3"></a>
### `abilities §3` Recipes

A haste of two stacks, each +50% move speed through an additive percent,
on a base speed of 4: speed 8 while it lasts, 4 once its ticks run out.

```ts recipe
import { defineSystem, og } from "@origingame/engine";
import type { Entity, Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const value = (world: World) => must(world.get(world.find("speed") as Entity, og.Stat)).value;

const Check = defineSystem("Check", { reads: [og.Stat] }, (_q, time, world) => {
  if (time.tick === 5) og.assert.always(value(world) === 8, "two stacks of +50% double 4", { value: value(world) });
  if (time.tick === 40) {
    og.assert.always(value(world) === 4, "the haste ran out", { value: value(world) });
    og.assert.reachable("the haste ran out", {});
  }
});

function haste(world: World, target: Entity, stat: Entity): void {
  const s = must(world.spawn({ parent: target }));
  must(world.insert(s, og.Status, { kind: "haste", stacking: "stack", ticks: 30, stacks: 1, maxStacks: 3 }));
  const m = must(world.spawn({ parent: s }));
  must(world.insert(m, og.Modifier, { stat, op: "percent", amount: 0.5 }));
}

export default og.game({
  systems: [...og.systems.abilities, Check],
  start(world) {
    const hero = must(world.spawn({ name: "hero" }));
    const speed = must(world.spawn({ name: "speed", parent: hero }));
    must(world.insert(speed, og.Stat, { base: 4 }));
    haste(world, hero, speed);
    haste(world, hero, speed);
  },
});
```
