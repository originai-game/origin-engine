// og.systems.interact (IDL `interact`): an og.Interactor focuses one
// og.Interactable among og.play.candidates by its own rule, uses it on a
// press of its action (og.input.consume) or once the action has been held on
// it for its holdTicks, and reports og.Interacted. Using an og.Door flips it,
// an og.Toggle flips, an og.Grab is picked up and the next press throws it;
// a plate toggle follows its sensor (og.physics.sensorContents) and a door
// follows its toggle, opening and closing its navmesh link with it. A game
// lists the systems in GameDesc.systems and sets every number: a zero range
// focuses nothing.
//
// Four systems, because a system's query requires every component it
// declares: og.interactors, og.toggles and og.doors list their entities, and
// og.interact reads and writes the rest entity by entity (its query matches
// nothing). All their state is in these components and the navmesh, so
// saves, restores and replays carry it; ticks count holds, the choice is a
// total order, and vectors use + - * only.
//
// To change a rule, copy this file into the game and list the copies in
// GameDesc.systems instead of og.systems.interact.

import type { Entity, World } from "@origingame/engine";
import { calls, defineSystem, BUILTINS, type SystemToken, type Token } from "../runtime.ts";

const T = BUILTINS as unknown as Record<string, Token>;
const Interactable = T.Interactable!;
const Interactor = T.Interactor!;
const Interacted = T.Interacted!;
const Door = T.Door!;
const Toggle = T.Toggle!;
const Grab = T.Grab!;
const InteractEntities = T.InteractEntities!;
const Transform = T.Transform!;
const RigidBody = T.RigidBody!;
const Velocity = T.Velocity!;
const Contact = T.Contact!;

type V3 = { x: number; y: number; z: number };
type Q = { x: number; y: number; z: number; w: number };
type Rule = "nearest" | "centred" | "priority";
type W = { readonly id: number };

interface InteractorValue {
  action: string;
  range: number;
  cone: number;
  sight: boolean;
  sightMask: number;
  rule: Rule;
  eye: V3;
  focus: Entity;
  progress: number;
  holding: Entity;
}
interface InteractableValue {
  priority: number;
  holdTicks: number;
  disabled: boolean;
}
interface DoorValue {
  open: boolean;
  toggle: Entity;
  mesh: number;
  link: number;
  linkOpen: boolean;
  linked: boolean;
}
interface ToggleValue {
  on: boolean;
  plate: boolean;
}
interface GrabValue {
  carry: V3;
  throwSpeed: number;
  heldBy: Entity;
}
interface TransformValue {
  position: V3;
  quaternion: Q;
}
interface Candidate {
  entity: Entity;
  distance: number;
  angle: number;
  visible: boolean;
}
interface Lists {
  interactors: readonly Entity[];
  toggles: readonly Entity[];
  doors: readonly Entity[];
}
type Got<V> = { ok: true; value: V } | { ok: false; reason?: string };

function get<V>(world: World, e: Entity, token: Token): V | null {
  if (!e) return null;
  const r = (world.get as (e: Entity, t: Token) => Got<V>)(e, token);
  return r.ok ? r.value : null;
}

function set(world: World, e: Entity, token: Token, value: object): void {
  const r = (world.set as (e: Entity, t: Token, v: object) => Got<unknown>)(e, token, value);
  if (!r.ok) throw new Error(`og.interact: ${r.reason}`);
}

function must<V>(r: unknown): V {
  const g = r as Got<V>;
  if (!g.ok) throw new Error(`og.interact: ${g.reason}`);
  return g.value;
}

/** q * v * q^-1. */
function rotate(q: Q, v: V3): V3 {
  const tx = 2 * (q.y * v.z - q.z * v.y);
  const ty = 2 * (q.z * v.x - q.x * v.z);
  const tz = 2 * (q.x * v.y - q.y * v.x);
  return {
    x: v.x + q.w * tx + (q.y * tz - q.z * ty),
    y: v.y + q.w * ty + (q.z * tx - q.x * tz),
    z: v.z + q.w * tz + (q.x * ty - q.y * tx),
  };
}

const add = (a: V3, b: V3): V3 => ({ x: a.x + b.x, y: a.y + b.y, z: a.z + b.z });
const FORWARD: V3 = { x: 0, y: 0, z: 1 };
const IDENTITY: Q = { x: 0, y: 0, z: 0, w: 1 };

interface ViewLike {
  count: number;
  entity(i: number): number;
}

/** A system that lists every entity of one component, in entity order, into one list of og.InteractEntities. */
function lister(name: string, token: Token, key: keyof Lists, cap: number, after: string[]): SystemToken {
  return defineSystem(name, { reads: [token], writes: [InteractEntities], after }, (q: { archetypes(): readonly ViewLike[] }, _t: unknown, world: World) => {
    const found: Entity[] = [];
    for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) found.push(a.entity(i) as Entity);
    found.sort((x, y) => (x as number) - (y as number));
    if (found.length > cap) throw new Error(`og.interact: ${found.length} entities carry og.${token.name.replace(/^og\./, "")}; og.InteractEntities lists at most ${cap}`);
    const r = (world.resource as (t: Token) => Got<Lists>)(InteractEntities);
    if (!r.ok) return;
    const was = r.value[key];
    if (was.length === found.length && was.every((e, i) => e === found[i])) return;
    (r.value as unknown as Record<string, Entity[]>)[key] = found;
  });
}

export const interactors = lister("og.interactors", Interactor, "interactors", 256, []);
export const toggles = lister("og.toggles", Toggle, "toggles", 1024, ["og.interactors"]);
export const doors = lister("og.doors", Door, "doors", 1024, ["og.toggles"]);

/** The focus by the interactor's rule: candidates come nearest first, equal distances in entity order. */
function choose(world: World, rule: Rule, found: readonly Candidate[], self: Entity): Entity {
  let best: Candidate | null = null;
  let bestPriority = 0;
  for (const c of found) {
    if (c.entity === self || !c.visible) continue;
    const it = get<InteractableValue>(world, c.entity, Interactable);
    if (!it || it.disabled) continue;
    const grab = get<GrabValue>(world, c.entity, Grab);
    if (grab && grab.heldBy) continue;
    if (!best) {
      best = c;
      bestPriority = it.priority;
      continue;
    }
    // Later candidates are never nearer: only a strictly better key replaces the best.
    if (rule === "centred" ? c.angle < best.angle : rule === "priority" ? it.priority > bestPriority : false) {
      best = c;
      bestPriority = it.priority;
    }
  }
  return best ? best.entity : (0 as Entity);
}

/** What using `target` does, after og.Interacted is sent. */
function use(world: World, who: Entity, target: Entity, holder: InteractorValue): void {
  (world.emit as (t: Token, v: object) => Got<unknown>)(Interacted, { interactor: who, target });
  const grab = get<GrabValue>(world, target, Grab);
  if (grab) {
    set(world, target, Grab, { heldBy: who });
    set(world, target, RigidBody, { kind: "kinematic" });
    holder.holding = target;
  }
  const toggle = get<ToggleValue>(world, target, Toggle);
  if (toggle && !toggle.plate) set(world, target, Toggle, { on: !toggle.on });
  const door = get<DoorValue>(world, target, Door);
  if (door && !door.toggle) set(world, target, Door, { open: !door.open });
}

/** One interactor's tick: its carried body, or its focus, hold and use. */
function stepInteractor(world: World, who: Entity): void {
  const me = get<InteractorValue>(world, who, Interactor);
  const pose = get<TransformValue>(world, who, Transform);
  if (!me || !pose) return;
  const q = pose.quaternion.w === 0 && pose.quaternion.x === 0 && pose.quaternion.y === 0 && pose.quaternion.z === 0 ? IDENTITY : pose.quaternion;
  const next: InteractorValue = { ...me };
  const grab = get<GrabValue>(world, me.holding, Grab);
  if (me.holding && (!grab || grab.heldBy !== who)) next.holding = 0 as Entity;
  if (next.holding) {
    next.focus = 0 as Entity;
    next.progress = 0;
    if (me.action && calls.input.consume(world as unknown as W, me.action)) {
      // Thrown: dynamic again, along the carrier's forward.
      const dir = rotate(q, FORWARD);
      const s = grab!.throwSpeed;
      set(world, me.holding, Grab, { heldBy: 0 as Entity });
      set(world, me.holding, RigidBody, { kind: "dynamic" });
      set(world, me.holding, Velocity, { linear: { x: dir.x * s, y: dir.y * s, z: dir.z * s }, angular: { x: 0, y: 0, z: 0 } });
      next.holding = 0 as Entity;
    } else {
      set(world, me.holding, Transform, { position: add(pose.position, rotate(q, grab!.carry)), quaternion: q });
      set(world, me.holding, Velocity, { linear: { x: 0, y: 0, z: 0 }, angular: { x: 0, y: 0, z: 0 } });
    }
  } else if (me.range > 0) {
    const found = must<Candidate[]>(
      calls.play.candidates(world as unknown as W, {
        from: add(pose.position, rotate(q, me.eye)),
        forward: rotate(q, FORWARD),
        range: me.range,
        cone: me.cone,
        with: "og.Interactable",
        sight: me.sight,
        filter: { mask: me.sightMask, exclude: who },
      }),
    );
    next.focus = choose(world, me.rule, found, who);
    if (next.focus !== me.focus) next.progress = 0;
    const it = get<InteractableValue>(world, next.focus, Interactable);
    if (it && me.action) {
      if (next.progress > 0) {
        // A hold under way: it goes on while the action is down, and ends when it is let go.
        next.progress = calls.input.isActionDown(world as unknown as W, me.action) ? next.progress + 1 : 0;
      } else if (calls.input.consume(world as unknown as W, me.action)) {
        next.progress = 1;
      }
      if (next.progress > 0 && next.progress >= it.holdTicks) {
        next.progress = 0;
        use(world, who, next.focus, next);
      }
    } else {
      next.progress = 0;
    }
  } else {
    next.focus = 0 as Entity;
    next.progress = 0;
  }
  if (next.focus !== me.focus || next.progress !== me.progress || next.holding !== me.holding) {
    set(world, who, Interactor, { focus: next.focus, progress: next.progress, holding: next.holding });
  }
}

/** og.interact: each interactor in entity order, then plates, then doors and their links. */
export const interactStep = defineSystem(
  "og.interact",
  {
    reads: [Interactable, InteractEntities, Contact],
    writes: [Interactor, Toggle, Door, Grab, Transform, RigidBody, Velocity, Interacted],
    after: ["og.doors"],
  },
  (_q: unknown, _time: unknown, world: World) => {
    const r = (world.resource as (t: Token) => Got<Lists>)(InteractEntities);
    if (!r.ok) return;
    for (const e of r.value.interactors) stepInteractor(world, e);
    for (const e of r.value.toggles) {
      const t = get<ToggleValue>(world, e, Toggle);
      if (!t || !t.plate) continue;
      const on = must<Entity[]>(calls.physics.sensorContents(world as unknown as W, e)).length > 0;
      if (on !== t.on) set(world, e, Toggle, { on });
    }
    for (const e of r.value.doors) {
      const d = get<DoorValue>(world, e, Door);
      if (!d) continue;
      let open = d.open;
      const t = get<ToggleValue>(world, d.toggle, Toggle);
      if (t) open = t.on;
      const push = d.mesh !== 0 && (!d.linked || open !== d.linkOpen);
      if (push) must(calls.nav.setLinkOpen(world as unknown as W, d.mesh, d.link, open));
      if (open !== d.open || push) set(world, e, Door, push ? { open, linkOpen: open, linked: true } : { open });
    }
  },
);

export const interact: readonly SystemToken[] = Object.freeze([interactors, toggles, doors, interactStep]);
