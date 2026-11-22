// og.systems.abilities (IDL `abilities`): an og.Ability is used on a press
// of its action (og.input.consume, only when it can be used) or by
// og.abilities.use, spending a charge, its lock and its cost from its pool
// (an og.Stat's base), and reports og.AbilityUsed; below maxCharges a charge
// returns every rechargeTicks. An og.Status spawned on a target merges with
// the target's status of the same kind by that status's stacking rule and
// counts down its ticks; an og.Stat's value is its base with its og.Modifier
// entities applied: adds, then the percents' sum, then each multiply, each
// kind in ascending order of what it adds. Ticks count only while time
// moves (none in a hit stop). A game lists the systems in GameDesc.systems
// and sets every number: an ability of 0 charges is never used.
//
// Five systems, because a system's query requires every component it
// declares: four list their entities, and og.abilities (whose query matches
// nothing) reads and writes the rest entity by entity. All their state is in
// these components, so saves, restores and replays carry it.
//
// To change a rule, copy this file into the game and list the copies in
// GameDesc.systems instead of og.systems.abilities.

import type { Entity, World } from "@origingame/engine";
import { calls, defineSystem, BUILTINS, type SystemToken, type Token } from "../runtime.ts";

const T = BUILTINS as unknown as Record<string, Token>;
const Ability = T.Ability!;
const AbilityUsed = T.AbilityUsed!;
const Stat = T.Stat!;
const Modifier = T.Modifier!;
const Status = T.Status!;
const AbilityEntities = T.AbilityEntities!;
const Parent = T.Parent!;

type W = { readonly id: number };
type Got<V> = { ok: true; value: V } | { ok: false; reason?: string };
type Done = { ok: true; value: undefined } | { ok: false; reason: string };

interface AbilityValue {
  owner: Entity;
  action: string;
  maxCharges: number;
  charges: number;
  rechargeTicks: number;
  rechargedTicks: number;
  lockTicks: number;
  lockLeftTicks: number;
  pool: Entity;
  cost: number;
}
interface StatValue {
  base: number;
  value: number;
}
interface ModifierValue {
  stat: Entity;
  op: "add" | "percent" | "multiply";
  amount: number;
}
interface StatusValue {
  kind: string;
  stacking: "refresh" | "stack" | "strongest";
  ticks: number;
  stacks: number;
  maxStacks: number;
  strength: number;
  applied: boolean;
}
interface Lists {
  abilities: readonly Entity[];
  stats: readonly Entity[];
  statuses: readonly Entity[];
  modifiers: readonly Entity[];
}

function get<V>(world: World, e: Entity, token: Token): V | null {
  if (!e) return null;
  const r = (world.get as (e: Entity, t: Token) => Got<V>)(e, token);
  return r.ok ? r.value : null;
}

function set(world: World, e: Entity, token: Token, value: object): void {
  const r = (world.set as (e: Entity, t: Token, v: object) => Got<unknown>)(e, token, value);
  if (!r.ok) throw new Error(`og.abilities: ${r.reason}`);
}

/** Why an ability cannot be used now, or null when it can. */
function blocked(world: World, a: AbilityValue): string | null {
  if (a.charges === 0) return "it has no charge";
  if (a.lockLeftTicks > 0) return `it is locked for ${a.lockLeftTicks} more ticks`;
  if (a.pool && a.cost > 0) {
    const pool = get<StatValue>(world, a.pool, Stat);
    if (!pool) return `its pool ${a.pool} has no og.Stat`;
    if (pool.value < a.cost) return `its pool holds ${pool.value}, under its cost of ${a.cost}`;
  }
  return null;
}

export function ready(world: World, ability: Entity): boolean {
  const a = get<AbilityValue>(world, ability, Ability);
  return a !== null && blocked(world, a) === null;
}

export function use(world: World, ability: Entity): Done {
  const a = get<AbilityValue>(world, ability, Ability);
  if (!a) return { ok: false, reason: `og.abilities: entity ${ability} has no og.Ability` };
  const why = blocked(world, a);
  if (why) return { ok: false, reason: `og.abilities: the ability ${ability} cannot be used: ${why}` };
  if (a.pool && a.cost > 0) {
    const pool = get<StatValue>(world, a.pool, Stat)!;
    // The value is the base with modifiers: take the cost from both, so a second use this tick sees it.
    set(world, a.pool, Stat, { base: pool.base - a.cost, value: pool.value - a.cost });
  }
  set(world, ability, Ability, { charges: a.charges - 1, lockLeftTicks: a.lockTicks });
  (world.emit as (t: Token, v: object) => Got<unknown>)(AbilityUsed, { ability, owner: a.owner });
  return { ok: true, value: undefined };
}

/** One ability's tick: its lock and recharge if time moves, then a press. */
function stepAbility(world: World, e: Entity, moving: boolean): void {
  const a = get<AbilityValue>(world, e, Ability);
  if (!a) return;
  let { charges, rechargedTicks: recharge, lockLeftTicks: lock } = a;
  if (moving) {
    if (lock > 0) lock -= 1;
    if (charges < a.maxCharges && a.rechargeTicks > 0) {
      recharge += 1;
      if (recharge >= a.rechargeTicks) {
        charges += 1;
        recharge = 0;
      }
    }
  }
  if (charges >= a.maxCharges) recharge = 0;
  if (charges !== a.charges || recharge !== a.rechargedTicks || lock !== a.lockLeftTicks) set(world, e, Ability, { charges, rechargedTicks: recharge, lockLeftTicks: lock });
  if (a.action && a.maxCharges > 0 && blocked(world, { ...a, charges, lockLeftTicks: lock }) === null && calls.input.consume(world as unknown as W, a.action)) {
    const r = use(world, e);
    if (!r.ok) throw new Error(r.reason);
  }
}

/** Merges each target's statuses of one kind, then counts them down: the statuses it despawned. */
function stepStatuses(world: World, list: readonly Entity[], moving: boolean): Set<Entity> {
  const gone = new Set<Entity>();
  const drop = (e: Entity) => {
    gone.add(e);
    const d = world.despawn(e);
    if (!d.ok) throw new Error(`og.abilities: ${d.reason}`);
  };
  const values = new Map<Entity, StatusValue>();
  for (const e of list) {
    const s = get<StatusValue>(world, e, Status);
    if (s) values.set(e, s);
  }
  const key = (e: Entity, s: StatusValue) => (s.kind ? `${get<{ entity: Entity }>(world, e, Parent)?.entity ?? 0}:${s.kind}` : "");
  // The statuses already on their targets keep their place; the new ones merge into them in entity order.
  const kept = new Map<string, Entity>();
  for (const [e, s] of values) if (s.applied && key(e, s) && !kept.has(key(e, s))) kept.set(key(e, s), e);
  for (const [e, s] of values) {
    if (s.applied) continue;
    const k = key(e, s);
    const firstOf = k ? kept.get(k) : undefined;
    if (firstOf === undefined) {
      s.applied = true;
      if (k) kept.set(k, e);
      set(world, e, Status, { applied: true });
      continue;
    }
    const old = values.get(firstOf)!;
    if (old.stacking === "strongest" && s.strength > old.strength) {
      drop(firstOf);
      s.applied = true;
      kept.set(k, e);
      set(world, e, Status, { applied: true });
      continue;
    }
    if (old.stacking === "stack") old.stacks = old.maxStacks > 0 ? Math.max(old.stacks, Math.min(old.stacks + s.stacks, old.maxStacks)) : old.stacks;
    if (old.stacking !== "strongest" || s.strength === old.strength) old.ticks = s.ticks;
    set(world, firstOf, Status, { stacks: old.stacks, ticks: old.ticks });
    drop(e);
  }
  if (!moving) return gone;
  for (const [e, s] of values) {
    if (gone.has(e) || s.ticks === 0) continue;
    if (s.ticks === 1) drop(e);
    else set(world, e, Status, { ticks: s.ticks - 1 });
  }
  return gone;
}

/** Each stat's value: its base, plus its adds, times 1 + its percents' sum, times 1 + each multiply. */
function stepStats(world: World, lists: Lists, gone: ReadonlySet<Entity>): void {
  const by = new Map<Entity, { add: number[]; percent: number[]; multiply: number[] }>();
  for (const e of lists.modifiers) {
    const m = get<ModifierValue>(world, e, Modifier);
    if (!m || !m.stat || m.amount === 0) continue;
    let times = 1;
    const parent = get<{ entity: Entity }>(world, e, Parent)?.entity ?? (0 as Entity);
    const status = get<StatusValue>(world, parent, Status);
    if (status) {
      if (gone.has(parent)) continue;
      times = status.stacks;
    }
    if (times === 0) continue;
    let list = by.get(m.stat);
    if (!list) by.set(m.stat, (list = { add: [], percent: [], multiply: [] }));
    list[m.op].push(m.amount * times);
  }
  const up = (a: number, b: number) => a - b;
  for (const e of lists.stats) {
    const s = get<StatValue>(world, e, Stat);
    if (!s) continue;
    const m = by.get(e);
    let v = s.base;
    if (m) {
      for (const x of m.add.sort(up)) v += x;
      let percent = 0;
      for (const x of m.percent.sort(up)) percent += x;
      v *= 1 + percent;
      for (const x of m.multiply.sort(up)) v *= 1 + x;
    }
    if (Math.fround(v) !== s.value) set(world, e, Stat, { value: v });
  }
}

interface ViewLike {
  count: number;
  entity(i: number): number;
}

/** A system that lists every entity of one component, in entity order, into one list of og.AbilityEntities. */
function lister(name: string, token: Token, key: keyof Lists, after: string[]): SystemToken {
  return defineSystem(name, { reads: [token], writes: [AbilityEntities], after }, (q: { archetypes(): readonly ViewLike[] }, _t: unknown, world: World) => {
    const found: Entity[] = [];
    for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) found.push(a.entity(i) as Entity);
    found.sort((x, y) => (x as number) - (y as number));
    if (found.length > 1024) throw new Error(`og.abilities: ${found.length} entities carry og.${token.name.replace(/^og\./, "")}; og.AbilityEntities lists at most 1024`);
    const r = (world.resource as (t: Token) => Got<Lists>)(AbilityEntities);
    if (!r.ok) return;
    const was = r.value[key];
    if (was.length === found.length && was.every((e, i) => e === found[i])) return;
    (r.value as unknown as Record<string, Entity[]>)[key] = found;
  });
}

export const abilityList = lister("og.abilityList", Ability, "abilities", []);
export const statList = lister("og.statList", Stat, "stats", ["og.abilityList"]);
export const statusList = lister("og.statusList", Status, "statuses", ["og.statList"]);
export const modifierList = lister("og.modifierList", Modifier, "modifiers", ["og.statusList"]);

/** og.abilities: each ability in entity order, then statuses, then stats. */
export const abilitiesStep = defineSystem(
  "og.abilities",
  { reads: [Modifier, Parent], writes: [AbilityEntities, Ability, Stat, Status, AbilityUsed], after: ["og.modifierList"] },
  (_q: unknown, time: { scaledDt: number }, world: World) => {
    const r = (world.resource as (t: Token) => Got<Lists>)(AbilityEntities);
    if (!r.ok) return;
    const moving = time.scaledDt > 0;
    for (const e of r.value.abilities) stepAbility(world, e, moving);
    const gone = stepStatuses(world, r.value.statuses, moving);
    stepStats(world, r.value, gone);
    if (gone.size === 0) return;
    // The lists name live entities only: drop the statuses despawned and the modifiers that went with them.
    const lists = r.value as unknown as Record<string, Entity[]>;
    lists.statuses = r.value.statuses.filter((e) => !gone.has(e));
    lists.modifiers = r.value.modifiers.filter((e) => !gone.has(get<{ entity: Entity }>(world, e, Parent)?.entity ?? (0 as Entity)));
  },
);

export const abilities: readonly SystemToken[] = Object.freeze([abilityList, statList, statusList, modifierList, abilitiesStep]);

/** og.abilities' functions (IDL `host false`). */
export const abilitiesModule = Object.freeze({ ready, use });
