// og.systems.combat (IDL `combat`): an og.Attack plays og.AttackMove entities
// tick by tick, takes og.AttackLink entities on buffered presses
// (og.input.consume) inside their windows, and drives its og.HitVolume
// entities; last tick's og.HitEvent takes its move's damage, or its
// og.Projectile's, from the target's og.Health unless the target's og.Hurt
// says it is invulnerable. A projectile is no attacker: it never enters
// og.CombatAttackers, and og.combat despawns it once its pierce is spent.
// A game lists the systems in GameDesc.systems and sets every number: a
// zero window, damage or invulnerability does nothing.
//
// Two systems, because a system's query requires every component it
// declares: og.combatIndex lists the attackers (its query is og.Attack), and
// og.combat reads and writes the moves, links, volumes and targets entity by
// entity (its query matches nothing). All their state is in these
// components, so saves, restores and replays carry it; counts are ticks and
// the arithmetic is one f32 subtraction, so every host computes the same.
//
// To change a rule, copy this file into the game and list the copies in
// GameDesc.systems instead of og.systems.combat.

import type { Entity, World } from "@origingame/engine";
import { calls, defineSystem, BUILTINS, type SystemToken, type Token } from "../runtime.ts";

const T = BUILTINS as unknown as Record<string, Token>;
const Attack = T.Attack!;
const AttackMove = T.AttackMove!;
const AttackLink = T.AttackLink!;
const Health = T.Health!;
const Hurt = T.Hurt!;
const HitVolume = T.HitVolume!;
const HitEvent = T.HitEvent!;
const Damaged = T.Damaged!;
const Died = T.Died!;
const CombatAttackers = T.CombatAttackers!;
const Projectile = T.Projectile!;

type V3 = { x: number; y: number; z: number };
type Phase = "idle" | "startup" | "active" | "recovery";

interface AttackValue {
  move: Entity;
  next: Entity;
  tick: number;
  phase: Phase;
  swing: number;
  swingMove: Entity;
  cancelable: boolean;
  starters: readonly Entity[];
  volumes: readonly Entity[];
}
interface MoveValue {
  startup: number;
  active: number;
  recovery: number;
  damage: number;
  volumes: number;
  cancelOpen: number;
  cancelClose: number;
  links: readonly Entity[];
}
interface LinkValue {
  action: string;
  to: Entity;
  open: number;
  close: number;
}
interface HurtValue {
  invulnerableTicks: number;
  invulnerableUntil: number;
  staggerTicks: number;
  staggeredUntil: number;
  knockback: number;
}
interface HitValue {
  volume: Entity;
  owner: Entity;
  swing: number;
  target: Entity;
  point: V3;
  normal: V3;
}
interface ProjectileValue {
  damage: number;
  shooter: Entity;
  pierce: number;
  struck: number;
}
interface VolumeValue {
  active: boolean;
  owner: Entity;
  swing: number;
}
type Got<V> = { ok: true; value: V } | { ok: false };

function get<V>(world: World, e: Entity, token: Token): V | null {
  if (!e) return null;
  const r = (world.get as (e: Entity, t: Token) => Got<V>)(e, token);
  return r.ok ? r.value : null;
}

function set(world: World, e: Entity, token: Token, value: object): void {
  const r = (world.set as (e: Entity, t: Token, v: object) => { ok: boolean; reason?: string })(e, token, value);
  if (!r.ok) throw new Error(`og.combat: ${r.reason}`);
}

function emit(world: World, token: Token, value: object): void {
  const r = (world.emit as (t: Token, v: object) => { ok: boolean; reason?: string })(token, value);
  if (!r.ok) throw new Error(`og.combat: ${r.reason}`);
}

const total = (m: MoveValue) => m.startup + m.active + m.recovery;

function phaseOf(m: MoveValue | null, t: number): Phase {
  if (!m || t >= total(m)) return "idle";
  if (t < m.startup) return "startup";
  return t < m.startup + m.active ? "active" : "recovery";
}

/** og.combatIndex: every og.Attack entity, in entity order, for og.combat. */
export const combatIndex = defineSystem(
  "og.combatIndex",
  { reads: [Attack], writes: [CombatAttackers] },
  (q: { archetypes(): readonly { count: number; entity(i: number): number }[] }, _time: unknown, world: World) => {
    const found: Entity[] = [];
    for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) found.push(a.entity(i) as Entity);
    found.sort((x, y) => (x as number) - (y as number));
    if (found.length > 1024) throw new Error(`og.combat: ${found.length} entities carry og.Attack; og.CombatAttackers lists at most 1024`);
    const r = (world.resource as (t: Token) => Got<{ attackers: readonly Entity[] }>)(CombatAttackers);
    if (!r.ok) return;
    const was = r.value.attackers;
    if (was.length === found.length && was.every((e, i) => e === found[i])) return;
    r.value.attackers = found;
  },
);

/** A hit's damage taken from its target, as og.Damaged and og.Died from `source`: whether it took any. */
function strike(world: World, tick: number, hit: HitValue, amount: number, source: Entity, move: Entity): boolean {
  if (!(amount > 0)) return false;
  const health = get<{ value: number; dead: boolean }>(world, hit.target, Health);
  if (!health || health.dead) return false;
  const hurt = get<HurtValue>(world, hit.target, Hurt);
  if (hurt && tick < hurt.invulnerableUntil) return false;
  const left = Math.max(0, health.value - amount);
  const dead = left <= 0;
  set(world, hit.target, Health, { value: left, dead });
  let knockback: V3 = { x: 0, y: 0, z: 0 };
  if (hurt) {
    const changes: Partial<HurtValue> = {};
    if (hurt.invulnerableTicks > 0) changes.invulnerableUntil = tick + hurt.invulnerableTicks;
    if (hurt.staggerTicks > 0) changes.staggeredUntil = tick + hurt.staggerTicks;
    if (changes.invulnerableUntil !== undefined || changes.staggeredUntil !== undefined) set(world, hit.target, Hurt, changes);
    // Away from the volume: the hit's normal points from the target towards it.
    if (hurt.knockback > 0) knockback = { x: -hit.normal.x * hurt.knockback, y: -hit.normal.y * hurt.knockback, z: -hit.normal.z * hurt.knockback };
  }
  emit(world, Damaged, { target: hit.target, source, move, swing: hit.swing, amount, health: left, point: hit.point, knockback });
  if (dead) emit(world, Died, { entity: hit.target, source });
  return true;
}

/**
 * Last tick's hits, in event order: a projectile's takes its damage, and
 * counts towards its pierce when it took any; any other takes the damage of
 * its owner's swing.
 */
function damage(world: World, tick: number): void {
  // The projectiles struck in this run, with their count so far.
  const shots = new Map<Entity, { p: ProjectileValue; struck: number }>();
  for (const hit of world.events(HitEvent as never) as unknown as readonly HitValue[]) {
    let shot = shots.get(hit.volume);
    if (!shot) {
      const p = get<ProjectileValue>(world, hit.volume, Projectile);
      if (p) shots.set(hit.volume, (shot = { p, struck: p.struck }));
    }
    if (shot) {
      const { p } = shot;
      // Its shooter, or a hit after it was spent in this run (the rest of its sweep): nothing.
      if (hit.target === p.shooter || (p.pierce > 0 && shot.struck >= p.pierce)) continue;
      // Only a strike that takes damage counts: a target without og.Health
      // (a graze ring), a dead one or one in its i-frames lets it pass.
      if (strike(world, tick, hit, p.damage, p.shooter || hit.owner, 0 as Entity)) shot.struck += 1;
      continue;
    }
    // A hit is read the tick after it struck, before any attack moves, so its
    // swing is its attacker's current one: it takes the move that swing
    // started, even if the game has cut it since.
    const attack = get<AttackValue>(world, hit.owner, Attack);
    if (!attack || attack.swing !== hit.swing) continue;
    const move = get<MoveValue>(world, attack.swingMove, AttackMove);
    if (move) strike(world, tick, hit, move.damage, hit.owner, attack.swingMove);
  }
  for (const [e, { p, struck }] of shots) {
    if (p.pierce > 0 && struck >= p.pierce) {
      const d = world.despawn(e);
      if (!d.ok) throw new Error(`og.combat: ${d.reason}`);
    } else if (struck !== p.struck) set(world, e, Projectile, { struck });
  }
}

/** One attacker's tick: its move advanced, a link taken, its volumes set. */
function play(world: World, e: Entity, tick: number, moving: boolean): void {
  const a = get<AttackValue>(world, e, Attack);
  if (!a) return;
  let move = a.move;
  let t = a.tick;
  let swing = a.swing;
  let next = a.next;
  let swingMove = a.swingMove;
  let m = get<MoveValue>(world, move, AttackMove);
  if (m && moving) {
    t += 1;
    if (t >= total(m)) {
      move = 0 as Entity;
      t = 0;
      m = null;
    }
  }
  if (!m) {
    // Idle, or cut by the game (`move` 0): nothing plays.
    move = 0 as Entity;
    t = 0;
  }
  const startable = (to: Entity) => {
    const n = get<MoveValue>(world, to, AttackMove);
    return n && total(n) > 0 ? n : null;
  };
  const start = (to: Entity, n: MoveValue) => {
    move = to;
    m = n;
    t = 0;
    swing += 1;
    swingMove = to;
  };
  const hurt = get<HurtValue>(world, e, Hurt);
  if (hurt && tick < hurt.staggeredUntil) {
    // A stagger cuts the move and starts nothing.
    move = 0 as Entity;
    m = null;
    t = 0;
    next = 0 as Entity;
  } else if (next) {
    const n = startable(next);
    if (n) start(next, n);
    next = 0 as Entity;
  } else if (moving) {
    const links = m ? m.links : a.starters;
    for (const l of links) {
      const link = get<LinkValue>(world, l, AttackLink);
      if (!link || !link.action) continue;
      if (m && !(t >= link.open && t < link.close)) continue;
      const n = startable(link.to);
      if (!n) continue;
      // Only a link that can be taken now consumes its press: an early one waits in its buffer.
      if (calls.input.consume(world as unknown as { readonly id: number }, link.action)) {
        start(link.to, n);
        break;
      }
    }
  }
  const phase = phaseOf(m, t);
  const cancelable = !m || (t >= m.cancelOpen && t < m.cancelClose);
  if (move !== a.move || next !== a.next || t !== a.tick || swing !== a.swing || phase !== a.phase || cancelable !== a.cancelable || swingMove !== a.swingMove) {
    set(world, e, Attack, { move, next, tick: t, swing, swingMove, phase, cancelable });
  }
  // Its volumes: those of the move's mask strike in its active window.
  a.volumes.forEach((v, i) => {
    const hv = get<VolumeValue>(world, v, HitVolume);
    if (!hv) return;
    const active = phase === "active" && i < 32 && ((m!.volumes >>> i) & 1) === 1;
    if (hv.active !== active || hv.owner !== e || hv.swing !== swing) set(world, v, HitVolume, { active, owner: e, swing });
  });
}

/** og.combat: last tick's hits, then each attacker in entity order. */
export const combatStep = defineSystem(
  "og.combat",
  {
    reads: [AttackMove, AttackLink, HitEvent, CombatAttackers],
    writes: [Attack, Health, Hurt, HitVolume, Projectile, Damaged, Died],
    after: ["og.combatIndex"],
  },
  (_q: unknown, time: { tick: number; scaledDt: number }, world: World) => {
    damage(world, time.tick);
    const r = (world.resource as (t: Token) => Got<{ attackers: readonly Entity[] }>)(CombatAttackers);
    if (!r.ok) return;
    // A hit stop holds every move where it is.
    const moving = time.scaledDt > 0;
    for (const e of r.value.attackers) play(world, e, time.tick, moving);
  },
);

export const combat: readonly SystemToken[] = Object.freeze([combatIndex, combatStep]);
