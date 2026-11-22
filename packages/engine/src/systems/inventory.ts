// og.systems.inventory (IDL `inventory`): an og.Container holds `slots`
// slot entities, its children, each an og.ItemStack of a table row and a
// count; a stack holds at most its row's stack limit, the field of the item
// table's component the container names (og.table.get). og.inventory's
// functions (add, remove, move, split, merge, craft) plan every slot they
// change on a copy and write it only when the whole move fits, so a refused
// move changes nothing. An og.Pickup lying in the world goes into the
// og.Container of the og.Collector inside its sensor
// (og.physics.sensorContents). roll draws a loot table on og.Rng.
//
// Three systems, because a system's query requires every component it
// declares: og.containers lists the containers, og.pickups lists the
// pickups, and og.inventory (whose query matches nothing) places each
// container's missing slots and moves pickups entity by entity. All their state is in these components, so saves, restores and
// replays carry it; slots go in slot order and pickups in entity order.
//
// To change a rule, copy this file into the game and list the copies in
// GameDesc.systems instead of og.systems.inventory.

import type { Entity, World } from "@origingame/engine";
import { calls, defineSystem, BUILTINS, type SystemToken, type Token } from "../runtime.ts";

const T = BUILTINS as unknown as Record<string, Token>;
const Container = T.Container!;
const ItemStack = T.ItemStack!;
const Pickup = T.Pickup!;
const Collector = T.Collector!;
const PickedUp = T.PickedUp!;
const InventoryEntities = T.InventoryEntities!;
const Parent = T.Parent!;
const Contact = T.Contact!;
const Rng = T.Rng!;

type W = { readonly id: number };
type Got<V> = { ok: true; value: V } | { ok: false; reason?: string };
type Done<V = void> = { ok: true; value: V } | { ok: false; reason: string };

interface ContainerValue {
  slots: number;
  stack: string;
}
interface StackValue {
  item: number;
  count: number;
  slot: number;
  container: Entity;
}
interface Lists {
  containers: readonly Entity[];
  pickups: readonly Entity[];
}
interface ItemCount {
  item: number;
  count: number;
}
interface LootEntry {
  item: number;
  weight: number;
  min: number;
  max: number;
}

const ok: Done = Object.freeze({ ok: true, value: undefined });
const no = (reason: string): Done<never> => ({ ok: false, reason: `og.inventory: ${reason}` });

function get<V>(world: World, e: Entity, token: Token): V | null {
  if (!e) return null;
  const r = (world.get as (e: Entity, t: Token) => Got<V>)(e, token);
  return r.ok ? r.value : null;
}

function set(world: World, e: Entity, token: Token, value: object): void {
  const r = (world.set as (e: Entity, t: Token, v: object) => Got<unknown>)(e, token, value);
  if (!r.ok) throw new Error(`og.inventory: ${r.reason}`);
}

/** Whether a system is running: world.query is refused inside one. */
function inSystem(world: World): boolean {
  return !(world.query as () => Got<unknown>)().ok;
}

/** A container's slots in slot order: its children that are its slots. */
function slotsOf(world: World, container: Entity): Entity[] {
  const found: [number, Entity][] = [];
  for (const c of world.children(container)) {
    const s = get<StackValue>(world, c, ItemStack);
    if (s && s.container === container) found.push([s.slot, c]);
  }
  found.sort((a, b) => a[0] - b[0] || (a[1] as number) - (b[1] as number));
  return found.map(([, e]) => e);
}

/** Spawns the slots a container lacks, numbered after the ones it has. */
function place(world: World, container: Entity, box: ContainerValue): number {
  const have = slotsOf(world, container).length;
  for (let slot = have; slot < box.slots; slot++) {
    const e = (world.spawn as (d: object) => Got<Entity>)({ parent: container, bare: true });
    if (!e.ok) throw new Error(`og.inventory: ${e.reason}`);
    const r = (world.insert as (e: Entity, t: Token, v: object) => Got<unknown>)(e.value, ItemStack, { item: 0, count: 0, slot, container });
    if (!r.ok) throw new Error(`og.inventory: ${r.reason}`);
  }
  return Math.max(0, box.slots - have);
}

/** One slot as a plan holds it. */
interface Cell {
  entity: Entity;
  slot: number;
  item: number;
  count: number;
  /** Takes nothing new: past a lowered slot count. */
  closed: boolean;
  /** The row's stack limit in its container. */
  limit: (item: number) => number;
}

/** Every slot a move reads, copied: written back only when the whole move fits. */
class Plan {
  readonly cells = new Map<Entity, Cell>();
  readonly before = new Map<Entity, string>();
  readonly world: World;
  constructor(world: World) {
    this.world = world;
  }

  /** A container's cells in slot order, or why it has none to plan with. */
  container(container: Entity): Done<Cell[]> {
    const box = get<ContainerValue>(this.world, container, Container);
    if (!box) return no(`entity ${container} has no og.Container`);
    let slots = slotsOf(this.world, container);
    if (slots.length < box.slots) {
      if (inSystem(this.world)) return no(`the container ${container}'s slots are placed at the end of the tick it appeared`);
      place(this.world, container, box);
      slots = slotsOf(this.world, container);
    }
    const limits = new Map<number, number>();
    const limit = (item: number): number => {
      let n = limits.get(item);
      if (n === undefined) limits.set(item, (n = stackLimit(this.world, box, item)));
      return n;
    };
    return { ok: true, value: slots.map((e) => this.cell(e, limit, box)) };
  }

  cell(e: Entity, limit: (item: number) => number, box: ContainerValue): Cell {
    let c = this.cells.get(e);
    if (!c) {
      const s = get<StackValue>(this.world, e, ItemStack)!;
      c = { entity: e, slot: s.slot, item: s.count > 0 ? s.item : 0, count: s.count, closed: s.slot >= box.slots, limit };
      this.cells.set(e, c);
      this.before.set(e, `${c.item}:${c.count}`);
    }
    return c;
  }

  /** The cells of the container a slot is in, and the slot's cell. */
  slot(e: Entity): Done<{ cells: Cell[]; cell: Cell; container: Entity }> {
    const s = get<StackValue>(this.world, e, ItemStack);
    if (!s || !s.container) return no(`entity ${e} is not a slot of an og.Container`);
    const cells = this.container(s.container);
    if (!cells.ok) return cells;
    const cell = cells.value.find((c) => c.entity === e);
    if (!cell) return no(`entity ${e} is not a slot of an og.Container`);
    return { ok: true, value: { cells: cells.value, cell, container: s.container } };
  }

  /** Writes every cell that changed. */
  commit(): void {
    for (const [e, c] of this.cells) {
      if (this.before.get(e) === `${c.item}:${c.count}`) continue;
      set(this.world, e, ItemStack, { item: c.count > 0 ? c.item : 0, count: c.count });
    }
  }
}

/** A row's stack limit in a container: its table's component's `stack` field, 0 when it has none. */
function stackLimit(world: World, box: ContainerValue, item: number): number {
  if (!box.stack || !item) return 0;
  const name = calls.table.name(world as unknown as W, item) as Got<{ schema: string }>;
  if (!name.ok) return 0;
  const row = calls.table.get(world as unknown as W, item, { name: name.value.schema }) as Got<Record<string, unknown>>;
  if (!row.ok) return 0;
  const n = row.value[box.stack];
  return typeof n === "number" && Number.isFinite(n) && n > 0 ? Math.floor(n) : 0;
}

/** Puts `count` of `item` into cells: onto stacks of it in slot order, then empty slots. Returns how many did not fit. */
function fill(cells: readonly Cell[], item: number, count: number, skip?: Cell): number {
  let left = count;
  for (const pass of [0, 1]) {
    for (const c of cells) {
      if (left === 0) return 0;
      if (c === skip || c.closed) continue;
      if (pass === 0 ? c.count === 0 || c.item !== item : c.count !== 0) continue;
      const room = c.limit(item) - c.count;
      if (room <= 0) continue;
      const n = Math.min(room, left);
      c.item = item;
      c.count += n;
      left -= n;
    }
  }
  return left;
}

/** Takes `count` of `item` from cells, last slot first. Returns how many were short. */
function drain(cells: readonly Cell[], item: number, count: number): number {
  let left = count;
  for (let i = cells.length - 1; i >= 0 && left > 0; i--) {
    const c = cells[i]!;
    if (c.count === 0 || c.item !== item) continue;
    const n = Math.min(c.count, left);
    c.count -= n;
    if (c.count === 0) c.item = 0;
    left -= n;
  }
  return left;
}

function whole(n: number): boolean {
  return Number.isInteger(n) && n >= 0;
}

function known(world: World, item: number): boolean {
  return item !== 0 && (calls.table.name(world as unknown as W, item) as Got<unknown>).ok;
}

export function slots(world: World, container: Entity): readonly Entity[] {
  return get<ContainerValue>(world, container, Container) ? slotsOf(world, container) : [];
}

export function count(world: World, container: Entity, item: number): number {
  let n = 0;
  for (const e of slots(world, container)) {
    const s = get<StackValue>(world, e, ItemStack)!;
    if (s.count > 0 && s.item === item) n += s.count;
  }
  return n;
}

export function canFit(world: World, container: Entity, item: number, n: number): boolean {
  if (!whole(n) || !known(world, item)) return false;
  const p = new Plan(world);
  const cells = p.container(container);
  return cells.ok && fill(cells.value, item, n) === 0;
}

export function add(world: World, container: Entity, item: number, n: number): Done {
  if (!whole(n)) return no(`a count is a whole number from 0, not ${n}`);
  if (n === 0) return ok;
  if (!known(world, item)) return no(`no loaded table has the row ${item}`);
  const p = new Plan(world);
  const cells = p.container(container);
  if (!cells.ok) return cells;
  const left = fill(cells.value, item, n);
  if (left > 0) return no(`${left} of ${n} do not fit in the container ${container}`);
  p.commit();
  return ok;
}

export function remove(world: World, container: Entity, item: number, n: number): Done {
  if (!whole(n)) return no(`a count is a whole number from 0, not ${n}`);
  if (n === 0) return ok;
  const p = new Plan(world);
  const cells = p.container(container);
  if (!cells.ok) return cells;
  const short = drain(cells.value, item, n);
  if (short > 0) return no(`the container ${container} holds ${n - short} of the row ${item}, not ${n}`);
  p.commit();
  return ok;
}

export function move(world: World, slot: Entity, to: Entity, n: number): Done {
  if (!whole(n)) return no(`a count is a whole number from 0, not ${n}`);
  if (n === 0) return ok;
  const p = new Plan(world);
  const from = p.slot(slot);
  if (!from.ok) return from;
  const cell = from.value.cell;
  if (cell.count < n) return no(`the slot ${slot} holds ${cell.count}, not ${n}`);
  const into = p.container(to);
  if (!into.ok) return into;
  const item = cell.item;
  cell.count -= n;
  if (cell.count === 0) cell.item = 0;
  const left = fill(into.value, item, n, cell);
  if (left > 0) return no(`${left} of ${n} do not fit in the container ${to}`);
  p.commit();
  return ok;
}

export function split(world: World, slot: Entity, n: number): Done<Entity> {
  const p = new Plan(world);
  const from = p.slot(slot);
  if (!from.ok) return from;
  const cell = from.value.cell;
  if (!Number.isInteger(n) || n < 1 || n >= cell.count) return no(`a split moves 1 to ${cell.count - 1} of the slot ${slot}, not ${n}`);
  const empty = from.value.cells.find((c) => c.count === 0 && !c.closed);
  if (!empty) return no(`the container ${from.value.container} has no empty slot`);
  empty.item = cell.item;
  empty.count = n;
  cell.count -= n;
  p.commit();
  return { ok: true, value: empty.entity };
}

export function merge(world: World, fromSlot: Entity, intoSlot: Entity): Done<number> {
  const p = new Plan(world);
  const from = p.slot(fromSlot);
  if (!from.ok) return from;
  const into = p.slot(intoSlot);
  if (!into.ok) return into;
  const a = from.value.cell;
  const b = into.value.cell;
  if (a === b || a.count === 0 || b.closed || (b.count > 0 && b.item !== a.item)) return { ok: true, value: 0 };
  const n = Math.min(a.count, b.limit(a.item) - b.count);
  if (n <= 0) return { ok: true, value: 0 };
  b.item = a.item;
  b.count += n;
  a.count -= n;
  if (a.count === 0) a.item = 0;
  p.commit();
  return { ok: true, value: n };
}

export function craft(world: World, container: Entity, desc: { inputs?: readonly ItemCount[]; outputs?: readonly ItemCount[] }): Done {
  const p = new Plan(world);
  const cells = p.container(container);
  if (!cells.ok) return cells;
  for (const { item, count: n } of desc.inputs ?? []) {
    if (!whole(n)) return no(`a count is a whole number from 0, not ${n}`);
    const short = drain(cells.value, item, n);
    if (short > 0) return no(`the input ${item} is ${short} short of ${n}`);
  }
  for (const { item, count: n } of desc.outputs ?? []) {
    if (!whole(n)) return no(`a count is a whole number from 0, not ${n}`);
    if (n > 0 && !known(world, item)) return no(`no loaded table has the row ${item}`);
    const left = fill(cells.value, item, n);
    if (left > 0) return no(`${left} of the output ${item} do not fit`);
  }
  p.commit();
  return ok;
}

export function roll(world: World, desc: { entries?: readonly LootEntry[]; rolls?: number }): readonly ItemCount[] {
  const entries = desc.entries ?? [];
  const rolls = desc.rolls ?? 0;
  const out: ItemCount[] = [];
  if (entries.length === 0 || rolls <= 0) return out;
  const r = (world.resource as (t: Token) => Got<{ weighted(w: number[]): number; integer(lo: number, hi: number): number }>)(Rng);
  if (!r.ok) throw new Error(`og.inventory: ${r.reason}`);
  const weights = entries.map((e) => e.weight);
  for (let i = 0; i < rolls; i++) {
    const at = r.value.weighted(weights);
    if (at < 0) break;
    const e = entries[at]!;
    const n = e.max > e.min ? r.value.integer(e.min, e.max) : e.min;
    out.push({ item: e.item, count: n });
  }
  return out;
}

interface ViewLike {
  count: number;
  entity(i: number): number;
}

/** The entities of a query's archetypes, in entity order. */
function listed(q: { archetypes(): readonly ViewLike[] }, cap: number, what: string): Entity[] {
  const found: Entity[] = [];
  for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) found.push(a.entity(i) as Entity);
  found.sort((x, y) => (x as number) - (y as number));
  if (found.length > cap) throw new Error(`og.inventory: ${found.length} entities carry og.${what}; og.InventoryEntities lists at most ${cap}`);
  return found;
}

function store(world: World, key: keyof Lists, found: Entity[]): void {
  const r = (world.resource as (t: Token) => Got<Lists>)(InventoryEntities);
  if (!r.ok) return;
  const was = r.value[key];
  if (was.length === found.length && was.every((e, i) => e === found[i])) return;
  (r.value as unknown as Record<string, Entity[]>)[key] = found;
}

/** og.containers: lists the containers (og.inventory places their slots). */
export const containers = defineSystem(
  "og.containers",
  { reads: [Container], writes: [InventoryEntities] },
  (q: { archetypes(): readonly ViewLike[] }, _t: unknown, world: World) => store(world, "containers", listed(q, 1024, "Container")),
);

export const pickups = defineSystem(
  "og.pickups",
  { reads: [Pickup], writes: [InventoryEntities], after: ["og.containers"] },
  (q: { archetypes(): readonly ViewLike[] }, _t: unknown, world: World) => store(world, "pickups", listed(q, 1024, "Pickup")),
);

/** og.inventory: each pickup, in entity order, into the collectors inside its sensor, in entity order. */
export const inventoryStep = defineSystem(
  "og.inventory",
  { reads: [InventoryEntities, Collector, Contact, Pickup, Container], writes: [ItemStack, Parent, PickedUp], after: ["og.pickups"] },
  (_q: unknown, _t: unknown, world: World) => {
    const r = (world.resource as (t: Token) => Got<Lists>)(InventoryEntities);
    if (!r.ok) return;
    // Slots spawned here are placed when this system returns: moves into them start next tick.
    for (const e of r.value.containers) {
      const box = get<ContainerValue>(world, e, Container);
      if (box) place(world, e, box);
    }
    for (const e of r.value.pickups) {
      const stack = get<StackValue>(world, e, ItemStack);
      const pick = get<{ partial: boolean }>(world, e, Pickup);
      if (!stack || !pick || stack.count === 0) continue;
      const inside = calls.physics.sensorContents(world as unknown as W, e) as Got<Entity[]>;
      if (!inside.ok) continue;
      let left = stack.count;
      for (const who of inside.value) {
        if (left === 0) break;
        const c = get<{ container: Entity }>(world, who, Collector);
        if (!c || !c.container) continue;
        const p = new Plan(world);
        const cells = p.container(c.container);
        if (!cells.ok) continue;
        const rest = fill(cells.value, stack.item, left);
        if (rest > 0 && !pick.partial) continue;
        if (rest === left) continue;
        p.commit();
        (world.emit as (t: Token, v: object) => Got<unknown>)(PickedUp, { collector: who, pickup: e, item: stack.item, count: left - rest });
        left = rest;
      }
      if (left === stack.count) continue;
      set(world, e, ItemStack, { count: left, item: left > 0 ? stack.item : 0 });
      if (left === 0) {
        const d = world.despawn(e);
        if (!d.ok) throw new Error(`og.inventory: ${d.reason}`);
      }
    }
  },
);

export const inventory: readonly SystemToken[] = Object.freeze([containers, pickups, inventoryStep]);

/** og.inventory's functions (IDL `host false`). */
export const inventoryModule = Object.freeze({ slots, count, canFit, add, remove, move, split, merge, craft, roll });
