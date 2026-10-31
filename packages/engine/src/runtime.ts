// The TypeScript runtime behind the generated declarations (types/index.d.ts):
// tokens, declarations, the dispatcher the engine calls, and the world, query
// and view objects game code is handed. Host-agnostic: a loader binds it to
// the web host (src/web/) or the native host (src/node/) with `bindHost`.
//
// Every engine call goes through the binding symbols of the generated ABI
// (src/generated/abi.ts). Structured values cross as JSON text; column views
// cross as whatever the host aliases engine memory with, turned into typed
// arrays by the binding's `columns`.

import bindings from "../types/bindings.json" with { type: "json" };
import type * as Api from "@origingame/engine";
import type { Dispatcher, HostAbi } from "./generated/abi.ts";
import { SYMBOLS, checkEntities } from "./generated/abi.ts";
import { noPlatform, type PlatformAnswer, type PlatformCall, type PlatformHost } from "./platform-local.ts";

// ── host binding ──────────────────────────────────────────────────────────

/** A typed array of the column kinds' element types. */
export type ColumnArray = Float32Array | Float64Array | Uint32Array | Int32Array | Uint8Array;

/** How a loader hands the runtime its host. */
export interface HostBinding {
  /** The exported binding symbols. */
  readonly abi: HostAbi<Uint32Array, unknown>;
  /** The host's name, for messages: `web` or `node`. */
  readonly name: string;
  /**
   * Typed arrays over what `og_archetype_view_column` answered, one per
   * field: `kinds[i]` is the field's column kind, `count` the entity count.
   * `null` when the host refused.
   */
  columns(raw: unknown, kinds: readonly ColumnSpec[], count: number): ColumnArray[] | null;
  /** Whether arrays made earlier still see engine memory (wasm memory growth detaches them). */
  fresh?(array: ColumnArray): boolean;
}

/** A column kind as the binding table declares it. */
export interface ColumnSpec {
  readonly name: string;
  readonly array: "Float32Array" | "Float64Array" | "Uint32Array" | "Int32Array" | "Uint8Array";
  readonly stride: number;
  /** The stride is each field's capacity (`str(32)`), not the kind's. */
  readonly capacity?: boolean;
  /** The list factory (`list(u32(), 8)`): no kind of its own, a list field's kind is its element's. */
  readonly list?: boolean;
}

export const COLUMNS: ReadonlyMap<string, ColumnSpec> = new Map(
  (bindings.columns as ColumnSpec[]).map((c) => [c.name, c]),
);

export const ARRAYS = {
  Float32Array,
  Float64Array,
  Uint32Array,
  Int32Array,
  Uint8Array,
} as const;

let host: HostBinding | null = null;
const queue: Array<(abi: HostAbi<Uint32Array, unknown>) => void> = [];

/** Bind the runtime to a host and flush the declarations made so far. */
export function bindHost(binding: HostBinding): void {
  const missing = SYMBOLS.filter((s) => typeof (binding.abi as unknown as Record<string, unknown>)[s] !== "function");
  if (missing.length > 0) {
    throw new Error(`the ${binding.name} host lacks ${missing.length} binding symbol(s): ${missing.join(", ")}; rebuild it from the same api.ogt`);
  }
  // Entity arguments are checked here, so null or undefined is refused
  // naming the call on both hosts alike (generated/abi.ts).
  host = { ...binding, abi: checkEntities(binding.abi) };
  publishBuild = null;
  binding.abi.og_host_set_dispatcher(dispatch);
  for (const declare of queue.splice(0)) declare(binding.abi);
}

/** The bound host, for loaders. */
export function boundHost(): HostBinding | null {
  return host;
}

/**
 * Counts engine calls made through `abi()` and dispatches (game code entered
 * or left): the engine changes a resource only inside one of them, so a
 * resource handle's read made while the count holds is still the store's
 * value (recordHandle).
 */
let engineCalls = 0;

function abi(): HostAbi<Uint32Array, unknown> {
  if (!host) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  // A resource handle's writes reach the engine before anything else does.
  if (dirtyRecords.length > 0) flushRecords();
  engineCalls++;
  return host.abi;
}

/** Run now if a host is bound, else when one is. */
function declare(call: (abi: HostAbi<Uint32Array, unknown>) => void): void {
  if (host) call(host.abi);
  else queue.push(call);
}

// ── results ───────────────────────────────────────────────────────────────

export type Result<T> = { readonly ok: true; readonly value: T } | { readonly ok: false; readonly reason: string };

function parse<T>(text: string): T {
  return JSON.parse(text) as T;
}

function refused<T>(reason: string): Result<T> {
  return { ok: false, reason };
}

function json(value: unknown): string | undefined {
  return value === undefined ? undefined : JSON.stringify(value);
}

// ── tokens and declarations ───────────────────────────────────────────────

export interface FieldSpec {
  readonly kind: string;
  readonly default: unknown;
  readonly options: Record<string, unknown>;
  /** A `str` field's capacity in bytes. */
  readonly capacity?: number;
  /** A list field's capacity in elements (`list(u32(), 8)`); `kind` is the element's. */
  readonly list?: number;
}

export interface Token {
  readonly kind: "component" | "resource" | "event";
  readonly name: string;
  readonly fields?: Readonly<Record<string, FieldSpec>>;
}

/** Field specs, in declaration order, as the host reads `Fields`. */
function fieldList(fields: Readonly<Record<string, FieldSpec>>): unknown[] {
  return Object.entries(fields).map(([name, spec]) => ({
    name,
    kind: spec?.kind,
    default: spec?.default,
    options: spec?.options ?? {},
    ...(spec?.capacity === undefined ? {} : { capacity: spec.capacity }),
    ...(spec?.list === undefined ? {} : { list: spec.list }),
  }));
}

export function defineRecord(kind: "component" | "resource", name: string, fields: Record<string, FieldSpec>): Token {
  const token: Token = Object.freeze({ kind, name, fields: Object.freeze({ ...fields }) });
  declare((a) => {
    if (kind === "component") a.og_reflect_define_component(name, JSON.stringify(fieldList(fields)));
    else a.og_reflect_define_resource(name, JSON.stringify(fieldList(fields)));
  });
  return token;
}

type DeclField = { name: string; column?: string; type?: string; default?: unknown; list?: number };

function builtinFields(fields: ReadonlyArray<DeclField>): Record<string, FieldSpec> {
  const out: Record<string, FieldSpec> = {};
  for (const f of fields) {
    const spec = { kind: f.column ?? f.type ?? "", default: f.default, options: Object.freeze({}) };
    out[f.name] = Object.freeze(f.list === undefined ? spec : { ...spec, list: f.list });
  }
  return out;
}

type Decl = { name: string; schema: string; fields: Array<DeclField> };

/** The engine-owned tokens on `og`, under their schema names (`og.Transform`). */
export const BUILTINS: Readonly<Record<string, Token>> = Object.freeze(
  Object.fromEntries([
    ...(bindings.components as Decl[]).map((c) => [c.name, Object.freeze({ kind: "component", name: c.schema, fields: Object.freeze(builtinFields(c.fields)) })]),
    ...(bindings.resources as Decl[]).map((r) => [r.name, Object.freeze({ kind: "resource", name: r.schema, fields: Object.freeze(builtinFields(r.fields)) })]),
    ...(bindings.events as Decl[]).map((e) => [e.name, Object.freeze({ kind: "event", name: e.schema })]),
  ]),
);

/** Game functions the engine calls back, by id. */
const callbacks: Function[] = [];

function callback(fn: Function): number {
  callbacks.push(fn);
  return callbacks.length - 1;
}

export interface SystemToken {
  readonly name: string;
  readonly reads: readonly Token[];
  readonly writes: readonly Token[];
}

interface SystemEntry {
  readonly name: string;
  readonly reads: ReadonlySet<string>;
  readonly writes: ReadonlySet<string>;
}

/** Systems by callback id: the dispatcher refuses any other id. */
const systems = new Map<number, SystemEntry>();
const systemHandles = new WeakMap<object, number>();

export interface SystemDescIn {
  readonly reads?: readonly Token[];
  readonly writes?: readonly Token[];
  readonly without?: readonly Token[];
  readonly after?: readonly string[];
  readonly before?: readonly string[];
}

const tokenRef = (t: Token) => ({ kind: t.kind, name: t.name });

export function defineSystem(name: string, desc: SystemDescIn, run: Function): SystemToken {
  const reads = [...(desc.reads ?? [])];
  const writes = [...(desc.writes ?? [])];
  const id = callback(run);
  systems.set(id, {
    name,
    reads: new Set(reads.map((t) => t.name)),
    writes: new Set(writes.map((t) => t.name)),
  });
  const wire = JSON.stringify({
    reads: reads.map(tokenRef),
    writes: writes.map(tokenRef),
    without: (desc.without ?? []).map(tokenRef),
    after: desc.after ?? [],
    before: desc.before ?? [],
  });
  const token = Object.freeze({ name, reads: Object.freeze(reads), writes: Object.freeze(writes) });
  declare((a) => systemHandles.set(token, a.og_system_define_system(name, wire, id)));
  return token;
}

export interface GameToken {
  readonly tick: number;
  readonly systems: readonly SystemToken[];
}

const gameHandles = new WeakMap<object, number>();

export interface GameDescIn {
  readonly tick?: number;
  readonly seed?: number;
  readonly scenes?: Readonly<Record<string, string>>;
  readonly scene?: string;
  readonly systems?: readonly SystemToken[];
  readonly resources?: readonly Token[];
  readonly commands?: Readonly<Record<string, Function>>;
  readonly commandHelp?: Readonly<Record<string, { readonly usage?: string; readonly doc?: string }>>;
  readonly maxCatchUp?: number;
  readonly facts?: Function | null;
  readonly start?: Function | null;
  readonly render?: Readonly<Record<string, unknown>>;
  readonly look?: string;
}

export function defineGame(desc: GameDescIn = {}): GameToken {
  const list = [...(desc.systems ?? [])];
  const commands: Record<string, number> = {};
  for (const [name, fn] of Object.entries(desc.commands ?? {})) commands[name] = callback(fn);
  const facts = desc.facts ? callback(desc.facts) : null;
  const start = desc.start ? callback(desc.start) : null;
  const token = Object.freeze({ tick: desc.tick ?? 60, systems: Object.freeze(list) });
  // Built when the host takes it, so the systems' host handles are known.
  declare((a) => {
    // Unset tick, seed, scenes and scene stay null: the engine takes
    // project.ogt's (CreateOptions.project), else 60, 0, none and "main".
    const wire = JSON.stringify({
      tick: desc.tick ?? null,
      seed: desc.seed ?? null,
      scenes: desc.scenes ?? null,
      scene: desc.scene ?? null,
      systems: list.map((s) => systemHandles.get(s) ?? -1),
      resources: (desc.resources ?? []).map((r) => r.name),
      commands,
      commandHelp: desc.commandHelp ?? {},
      maxCatchUp: desc.maxCatchUp ?? 4,
      facts,
      start,
      render: desc.render ?? {},
      look: desc.look ?? "",
    });
    gameHandles.set(token, a.og_game_game(wire));
  });
  return token;
}

export function registerCommand(name: string, handler: Function, desc?: { usage?: string; doc?: string }): Result<void> {
  const id = callback(handler);
  if (!host) {
    declare((a) => void a.og_commands_register(name, id, json(desc)));
    return { ok: true, value: undefined };
  }
  return parse(host.abi.og_commands_register(name, id, json(desc)));
}

// ── views ─────────────────────────────────────────────────────────────────

/** Moves on whenever views may have gone stale: after every call into game code, and at every structural change outside systems. */
let epoch = 0;

interface Running {
  readonly system: SystemEntry;
  readonly tick: number;
}

let running: Running | null = null;

interface ViewInfo {
  readonly view: number;
  readonly count: number;
  readonly tick: number;
  readonly types: readonly string[];
}

/** A component token's field names and column kinds, worked out once. */
const layouts = new WeakMap<Token, { names: string[]; kinds: ColumnSpec[] }>();

function layoutOf(component: Token): { names: string[]; kinds: ColumnSpec[] } {
  let l = layouts.get(component);
  if (!l) {
    const names = Object.keys(component.fields ?? {});
    const kinds = names.map((n) => {
      const field = component.fields![n]!;
      const kind = COLUMNS.get(field.kind);
      if (!kind) throw new Error(`'${component.name}.${n}' is not a column field`);
      // A str column's stride is the field's capacity: one byte lane per byte.
      if (kind.capacity) return { ...kind, stride: field.capacity ?? 0 };
      // A list's is its element's, once for the count and once per element.
      return field.list === undefined ? kind : { ...kind, stride: kind.stride * (field.list + 1) };
    });
    layouts.set(component, (l = { names, kinds }));
  }
  return l;
}

/**
 * The view object of a component's arrays. Hosts hand out the same arrays
 * while a column stays put, so the same object is handed back with them:
 * a system's views cost no allocation from one tick to the next. The object
 * is plain data; the ArchetypeView is what refuses stale use.
 */
const views = new WeakMap<Token, Array<{ arrays: ColumnArray[]; view: Record<string, ColumnArray> }>>();

function viewOf(component: Token, names: string[], arrays: ColumnArray[]): Record<string, ColumnArray> {
  let list = views.get(component);
  if (!list) views.set(component, (list = []));
  for (const entry of list) {
    if (entry.arrays.length === arrays.length && entry.arrays.every((a, i) => a === arrays[i])) return entry.view;
  }
  const view: Record<string, ColumnArray> = {};
  for (let i = 0; i < names.length; i++) view[names[i]!] = arrays[i]!;
  if (list.length >= 64) list.shift();
  list.push({ arrays, view });
  return view;
}

class ArchetypeViewImpl {
  readonly #info: ViewInfo;
  readonly #epoch: number;
  readonly #system: SystemEntry | null;
  #entities: Uint32Array | null = null;
  readonly #columns = new Map<string, object>();

  constructor(info: ViewInfo, system: SystemEntry | null) {
    this.count = info.count;
    this.#info = info;
    this.#epoch = epoch;
    this.#system = system;
  }

  #check(): void {
    if (this.#epoch !== epoch) {
      const owner = this.#system ? `system '${this.#system.name}'` : "a call outside systems";
      throw new Error(`stale ArchetypeView: it belonged to tick ${this.#info.tick}, ${owner}; views last for the system call they were handed to (outside systems, until the next structural change), so take them again from q.archetypes()`);
    }
  }

  /** The entity count: plain data (a hot loop's bound), so reading it never throws. */
  readonly count: number;

  get tick(): number {
    this.#check();
    return this.#info.tick;
  }

  get entities(): Uint32Array {
    this.#check();
    return (this.#entities ??= abi().og_archetype_view_get_entities(this.#info.view));
  }

  entity(index: number): number {
    const e = this.entities;
    return e[index * 2]! + e[index * 2 + 1]! * 4294967296;
  }

  has(component: Token): boolean {
    this.#check();
    return this.#info.types.includes(component.name);
  }

  column(component: Token): object {
    this.#check();
    const cached = this.#columns.get(component.name) as Record<string, ColumnArray> | undefined;
    // A grown wasm memory detached the arrays: make them again.
    if (cached && !(host!.fresh && Object.values(cached).some((a) => !host!.fresh!(a)))) return cached;
    const s = this.#system;
    if (s && !s.reads.has(component.name) && !s.writes.has(component.name)) {
      throw new Error(`system '${s.name}' did not declare '${component.name}' in its reads or writes; declare it to get its columns`);
    }
    if (!this.#info.types.includes(component.name)) {
      throw new Error(`this archetype has no '${component.name}'; check a.has(${component.name}) first`);
    }
    const { names, kinds } = layoutOf(component);
    const b = host!;
    const arrays = b.columns(abi().og_archetype_view_column(this.#info.view, component.name), kinds, this.#info.count);
    if (!arrays || arrays.length !== names.length) {
      throw new Error(`the ${b.name} host refused the columns of '${component.name}' (tick ${this.#info.tick})`);
    }
    const view = viewOf(component, names, arrays);
    this.#columns.set(component.name, view);
    return view;
  }
}

class QueryImpl {
  readonly #handle: number;
  readonly #system: SystemEntry | null;
  #cached: ArchetypeViewImpl[] | null = null;
  #cachedEpoch = -1;

  constructor(handle: number, system: SystemEntry | null) {
    this.#handle = handle;
    this.#system = system;
  }

  archetypes(): readonly ArchetypeViewImpl[] {
    if (this.#cached && this.#cachedEpoch === epoch) return this.#cached;
    const list = parse<ViewInfo[]>(abi().og_query_archetypes(this.#handle));
    this.#cached = list.map((info) => new ArchetypeViewImpl(info, this.#system));
    this.#cachedEpoch = epoch;
    return this.#cached;
  }

  count(): number {
    return abi().og_query_count(this.#handle);
  }
}

// ── declared access ───────────────────────────────────────────────────────

/** Whether the bound host is a publish build (the probe's hello says), learnt when the first engine is made. */
let publishBuild: boolean | null = null;

function learnBuild(engine: number): void {
  publishBuild ??= (parse<{ build?: string } | null>(abi().og_engine_probe_hello(engine))?.build ?? "dev") === "publish";
}

/**
 * Inside a system, a resource or event it did not declare (`read`: among its
 * reads or writes; `write`: among its writes). Outside a publish build that
 * throws with the system's name and the record, failing the tick; a publish
 * build lets the engine refuse it, which logs it once per system.
 */
function checkAccess(access: "read" | "write", name: string): boolean {
  const r = running;
  if (!r) return true;
  const s = r.system;
  if (access === "write" ? s.writes.has(name) : s.reads.has(name) || s.writes.has(name)) return true;
  if (publishBuild === true) return false;
  const list = access === "write" ? "writes" : "reads or writes";
  throw new Error(`system '${s.name}' did not declare '${name}' among its ${list} (${access} refused): add it to the system's ${list}`);
}

// ── resources as live values ──────────────────────────────────────────────
//
// A resource handle is a typed view of the resource's packed record: the
// runtime keeps one copy of each record's bytes per world, shared by every
// handle of it, and a field read or write is a DataView access at the
// field's offset, no JSON. The copy is fetched again (one engine call
// answering the bytes) only when an engine call or a dispatch happened since
// it was read, the only times the engine can change a resource; writes mark
// it dirty and are flushed in one call before the next engine call, at the
// end of the system or command that made them, and at once outside game
// code. So two handles agree, a read after an engine call sees the engine's
// value, and the engine sees every write before it can read the resource:
// the world (and its hash) is what writing field by field gave.

/** A field of a resource's packed layout, as the engine answers it. */
interface RecordField {
  readonly name: string;
  readonly kind: string;
  readonly offset: number;
  readonly size: number;
  readonly readonly: boolean;
  readonly variants?: readonly string[];
  /** A list's element kind (`f32`, `f64`, `u32`, `i32`, `handle`) and capacity in elements. */
  readonly element?: string;
  readonly capacity?: number;
}

interface RecordLayout {
  readonly epoch: number;
  readonly size: number;
  readonly fields: readonly RecordField[];
}

/** One world's copy of one resource's bytes. */
interface RecordCopy {
  readonly world: number;
  readonly name: string;
  layout: RecordLayout;
  /** The packed record then the layout epoch (u32), as the engine answered and is written back. */
  bytes: Uint8Array;
  view: DataView;
  /** `engineCalls` when the bytes were the engine's; -1: never. */
  at: number;
  dirty: boolean;
  handle: object | null;
}

/** Per world, per resource: its copy, or null for one read as JSON. */
const records = new Map<number, Map<string, RecordCopy | null>>();
const dirtyRecords: RecordCopy[] = [];
/** Calls into game code in progress (systems, commands, facts, start): writes wait for their end. */
let inGame = 0;

const utf8in = new TextEncoder();
const utf8out = new TextDecoder();

function forgetRecords(world: number): void {
  records.delete(world);
  for (let i = dirtyRecords.length - 1; i >= 0; i--) if (dirtyRecords[i]!.world === world) dirtyRecords.splice(i, 1);
}

function writeRecord(r: RecordCopy): void {
  r.dirty = false;
  const answer = parse<Result<void>>(host!.abi.og_engine_write_resource_bytes(r.world, r.name, r.bytes));
  engineCalls++;
  if (!answer.ok) {
    r.at = -1;
    throw new Error(`${r.name}: ${answer.reason}`);
  }
  r.at = engineCalls;
}

/** Writes every dirty record back, oldest first; throws the first refusal after writing the rest. */
function flushRecords(): void {
  let first: unknown = null;
  for (const r of dirtyRecords.splice(0)) {
    try {
      writeRecord(r);
    } catch (e) {
      first ??= e;
    }
  }
  if (first !== null) throw first;
}

function recordLayout(world: number, name: string): RecordLayout | null {
  const answer = parse<Result<RecordLayout>>(host!.abi.og_engine_resource_layout(world, name));
  return answer.ok ? answer.value : null;
}

/** The world's copy of a resource with a byte layout, or null for one read as JSON (og.Time, og.Rng, og.Input). */
function recordOf(world: number, name: string): RecordCopy | null {
  let byName = records.get(world);
  if (!byName) records.set(world, (byName = new Map()));
  const known = byName.get(name);
  if (known !== undefined) return known;
  const layout = recordLayout(world, name);
  // An unregistered name is asked again next time (a reload may declare it).
  if (!layout) {
    if (name === "og.Time" || name === "og.Rng" || name === "og.Input") byName.set(name, null);
    return null;
  }
  const bytes = new Uint8Array(layout.size + 4);
  const r: RecordCopy = { world, name, layout, bytes, view: new DataView(bytes.buffer), at: -1, dirty: false, handle: null };
  byName.set(name, r);
  return r;
}

/** Brings the copy up to the engine's value when an engine call happened since; false when the engine refused. */
function refresh(r: RecordCopy): boolean {
  if (r.dirty || r.at === engineCalls) return true;
  const fetched = host!.abi.og_engine_read_resource(r.world, r.name);
  if (fetched.length === 0) return false;
  const epoch = new DataView(fetched.buffer, fetched.byteOffset, fetched.byteLength).getUint32(fetched.length - 4, true);
  if (epoch !== r.layout.epoch || fetched.length !== r.layout.size + 4) {
    // A reload or a new engine declared the layouts again.
    const layout = recordLayout(r.world, r.name);
    if (!layout || fetched.length !== layout.size + 4) return false;
    r.layout = layout;
    // The next world.resource hands out a handle with the new fields.
    r.handle = null;
  }
  // Into the copy's own buffer: no allocation from one read to the next, and
  // what is written back is memory the runtime made (a host's external
  // buffer crosses back slower).
  if (r.bytes.length !== fetched.length) {
    r.bytes = new Uint8Array(fetched.length);
    r.view = new DataView(r.bytes.buffer);
  }
  r.bytes.set(fetched);
  r.at = engineCalls;
  return true;
}

const LANES: Readonly<Record<string, readonly string[]>> = {
  Vector2: ["x", "y"],
  Vector3: ["x", "y", "z"],
  Quaternion: ["x", "y", "z", "w"],
  Color: ["r", "g", "b"],
};

/** Bytes of one lane of a list's element kind. */
const LANE_SIZE: Readonly<Record<string, number>> = { f32: 4, f64: 8, u32: 4, i32: 4, handle: 8 };

function readLane(v: DataView, o: number, element: string): number {
  switch (element) {
    case "f32": return v.getFloat32(o, true);
    case "f64": return v.getFloat64(o, true);
    case "u32": return v.getUint32(o, true);
    case "i32": return v.getInt32(o, true);
    default: return v.getUint32(o, true) + v.getUint32(o + 4, true) * 4294967296;
  }
}

function writeLane(v: DataView, o: number, element: string, n: number): void {
  switch (element) {
    case "f32": return v.setFloat32(o, n, true);
    case "f64": return v.setFloat64(o, n, true);
    case "u32": return v.setUint32(o, n, true);
    case "i32": return v.setInt32(o, n, true);
    default:
      v.setUint32(o, n % 4294967296, true);
      v.setUint32(o + 4, Math.floor(n / 4294967296), true);
  }
}

/** A list element checked as the engine's JSON path checks it; throws what it refuses. */
function listElement(value: unknown, element: string, i: number): number {
  const n = number(value, `element ${i}`);
  const whole = (lo: number, hi: number, what: string) => {
    if (!Number.isInteger(n) || n < lo || n > hi) throw new Error(`element ${i}: ${n} is not ${what}`);
  };
  if (element === "u32") whole(0, 4294967295, "a u32");
  else if (element === "i32") whole(-2147483648, 2147483647, "an i32");
  else if (element === "handle") whole(0, 9007199254740991, "a handle");
  return n;
}

function decodeField(v: DataView, bytes: Uint8Array, f: RecordField): unknown {
  const o = f.offset;
  switch (f.kind) {
    case "list": {
      // A frozen copy: `push` throws instead of changing nothing.
      const element = f.element ?? "f32";
      const lane = LANE_SIZE[element] ?? 4;
      const count = Math.min(readLane(v, o, element), f.capacity ?? 0);
      const out: number[] = [];
      for (let i = 0; i < count; i++) out.push(readLane(v, o + lane * (i + 1), element));
      return Object.freeze(out);
    }
    case "f32": return v.getFloat32(o, true);
    case "f64": return v.getFloat64(o, true);
    case "u32": return v.getUint32(o, true);
    case "i32": return v.getInt32(o, true);
    case "bool": return bytes[o] !== 0;
    case "handle": return v.getUint32(o, true) + v.getUint32(o + 4, true) * 4294967296;
    case "enum": {
      const i = v.getUint32(o, true);
      return f.variants?.[i] ?? i;
    }
    case "str": {
      const lane = bytes.subarray(o, o + f.size);
      const end = lane.indexOf(0);
      return utf8out.decode(end < 0 ? lane : lane.subarray(0, end));
    }
    case "Matrix4": {
      const elements: number[] = [];
      for (let i = 0; i < 16; i++) elements.push(v.getFloat32(o + i * 4, true));
      return { elements };
    }
    default: {
      const out: Record<string, number> = {};
      (LANES[f.kind] ?? []).forEach((k, i) => (out[k] = v.getFloat32(o + i * 4, true)));
      return out;
    }
  }
}

function number(value: unknown, what: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`${what} is a number, got ${JSON.stringify(value) ?? String(value)}`);
  return value;
}

/** Writes `value` into a field's bytes, refusing (throwing) what the engine's JSON path refuses. */
function encodeField(v: DataView, bytes: Uint8Array, f: RecordField, value: unknown): void {
  const o = f.offset;
  switch (f.kind) {
    case "list": {
      // The whole list: an array, at most the capacity, every element checked before a byte moves.
      const element = f.element ?? "f32";
      const capacity = f.capacity ?? 0;
      const items = value === null || value === undefined ? [] : value;
      if (!Array.isArray(items)) throw new Error(`a list is written whole, as an array, got ${JSON.stringify(value)}`);
      if (items.length > capacity) {
        const kind = element === "handle" ? "Asset" : element;
        throw new Error(`${items.length} elements and the field holds ${capacity}: remove ${items.length - capacity}, or declare the field list(${kind}(), ${items.length}) or larger`);
      }
      const lanes = items.map((x, i) => listElement(x, element, i));
      const lane = LANE_SIZE[element] ?? 4;
      bytes.fill(0, o, o + f.size);
      writeLane(v, o, element, lanes.length);
      lanes.forEach((n, i) => writeLane(v, o + lane * (i + 1), element, n));
      return;
    }
    case "f32": return v.setFloat32(o, number(value, "the value"), true);
    case "f64": return v.setFloat64(o, number(value, "the value"), true);
    case "u32": {
      const n = number(value, "the value");
      if (!Number.isInteger(n) || n < 0 || n > 4294967295) throw new Error(`${n} is not a u32`);
      return v.setUint32(o, n, true);
    }
    case "i32": {
      const n = number(value, "the value");
      if (!Number.isInteger(n) || n < -2147483648 || n > 2147483647) throw new Error(`${n} is not an i32`);
      return v.setInt32(o, n, true);
    }
    case "bool": {
      if (value !== null && value !== undefined && typeof value !== "boolean") throw new Error(`expected true or false, got ${JSON.stringify(value)}`);
      bytes[o] = value === true ? 1 : 0;
      return;
    }
    case "handle": {
      const n = number(value, "a handle");
      if (!Number.isInteger(n) || n < 0 || n >= 9007199254740992) throw new Error(`${n} is not a handle`);
      v.setUint32(o, n % 4294967296, true);
      v.setUint32(o + 4, Math.floor(n / 4294967296), true);
      return;
    }
    case "enum": {
      const variants = f.variants ?? [];
      let i: number;
      if (value === null || value === undefined) i = 0;
      else if (typeof value === "string") {
        i = variants.indexOf(value);
        if (i < 0) throw new Error(`'${value}' is not one of ${JSON.stringify(variants)}`);
      } else {
        i = number(value, "an enum index");
        if (!Number.isInteger(i) || i < 0 || i >= variants.length) throw new Error(`enum index ${i}, but there are ${variants.length} variants`);
      }
      return v.setUint32(o, i, true);
    }
    case "str": {
      if (value !== null && value !== undefined && typeof value !== "string") throw new Error(`expected a string, got ${JSON.stringify(value)}`);
      const text = (value ?? "") as string;
      const encoded = utf8in.encode(text);
      if (encoded.length > f.size) {
        const shown = [...text].slice(0, 24).join("");
        const more = shown.length < text.length ? "..." : "";
        const fix = encoded.length <= 4096
          ? `shorten the text, or declare the field str(${encoded.length}) or larger`
          : "shorten the text: a string field holds at most 4096 bytes";
        throw new Error(`'${shown}${more}' is ${encoded.length} bytes of UTF-8 and the field holds ${f.size}: ${fix}`);
      }
      if (encoded.includes(0)) throw new Error("a string field cannot hold a NUL character (\\0)");
      bytes.fill(0, o, o + f.size);
      bytes.set(encoded, o);
      return;
    }
    case "Matrix4": {
      const elements = (value as { elements?: unknown } | null)?.elements;
      if (!Array.isArray(elements) || elements.length !== 16) throw new Error(`expected { elements: [16 numbers] }, got ${JSON.stringify(value)}`);
      const lanes = elements.map((e) => number(e, "an element"));
      lanes.forEach((n, i) => v.setFloat32(o + i * 4, n, true));
      return;
    }
    default: {
      const keys = LANES[f.kind] ?? [];
      if (typeof value !== "object" || value === null) throw new Error(`expected { ${keys.join(", ")} }, got ${JSON.stringify(value)}`);
      const lanes = keys.map((k) => number((value as Record<string, unknown>)[k] ?? null, k));
      lanes.forEach((n, i) => v.setFloat32(o + i * 4, n, true));
    }
  }
}

/** The handle of a resource read through its packed bytes, shared by every `world.resource` of it. */
function recordHandle(world: WorldImpl, r: RecordCopy): object {
  if (r.handle) return r.handle;
  const id = world.id;
  const name = r.name;
  const out: Record<string, unknown> = {};
  const fields = r.layout.fields;
  for (let i = 0; i < fields.length; i++) {
    const key = fields[i]!.name;
    // The field by name in the copy's current layout (a reload may move it).
    const field = (): RecordField => {
      const f = r.layout.fields[i];
      return f && f.name === key ? f : (r.layout.fields.find((x) => x.name === key) ?? fields[i]!);
    };
    let lastValue: unknown;
    const get = (): unknown => {
      if (r.at !== engineCalls && !r.dirty) {
        // Undeclared in a publish build: the engine refuses and logs it once;
        // the handle answers what it last read and the game keeps running.
        if (!checkAccess("read", name)) return lastValue;
        if (!refresh(r)) {
          const answer = parse<Result<Record<string, unknown>>>(host!.abi.og_world_resource(id, name));
          if (!answer.ok) throw new Error(`${name}: ${answer.reason}`);
          const value = answer.value[key];
          // A list reads as a frozen copy on this path too.
          return (lastValue = Array.isArray(value) ? Object.freeze(value) : value);
        }
      }
      return (lastValue = decodeField(r.view, r.bytes, field()));
    };
    const set = (value: unknown): void => {
      // Undeclared in a publish build: the engine refuses and logs it once; the game keeps running.
      if (!checkAccess("write", name)) return void abi().og_engine_write_resource(id, name, JSON.stringify({ [key]: value }));
      if (!refresh(r)) {
        const answer = parse<Result<void>>(abi().og_engine_write_resource(id, name, JSON.stringify({ [key]: value })));
        if (!answer.ok) throw new Error(`${name}.${key}: ${answer.reason}`);
        return;
      }
      try {
        // Every check comes before the first byte is written: a refused value leaves the copy as it was.
        encodeField(r.view, r.bytes, field(), value);
      } catch (e) {
        throw new Error(`${name}.${key}: ${(e as Error).message}`);
      }
      if (inGame === 0) return writeRecord(r);
      if (!r.dirty) {
        r.dirty = true;
        dirtyRecords.push(r);
      }
    };
    Object.defineProperty(out, key, fields[i]!.readonly ? { enumerable: true, get } : { enumerable: true, get, set });
  }
  methods(id, name, out);
  return (r.handle = Object.seal(out));
}

/** The methods a built-in resource's handle carries beside its fields. */
function methods(id: number, name: string, out: Record<string, unknown>): void {
  if (name === "og.Input") {
    Object.assign(out, {
      down: (key: string) => abi().og_input_down(id, key),
      pressed: (key: string) => abi().og_input_pressed(id, key),
      released: (key: string) => abi().og_input_released(id, key),
      axis: (negative: string, positive: string) => abi().og_input_axis(id, negative, positive),
      axis2: (left: string, right: string, down: string, up: string) => parse(abi().og_input_axis2(id, left, right, down, up)),
      buttonDown: (button: string) => abi().og_input_button_down(id, button),
      buttonPressed: (button: string) => abi().og_input_button_pressed(id, button),
      buttonReleased: (button: string) => abi().og_input_button_released(id, button),
    });
  }
  if (name === "og.Pointers") {
    Object.assign(out, {
      down: (pointer: number) => abi().og_pointers_down(id, pointer),
      pressed: (pointer: number) => abi().og_pointers_pressed(id, pointer),
      released: (pointer: number) => abi().og_pointers_released(id, pointer),
      position: (pointer: number) => parse(abi().og_pointers_position(id, pointer)),
      start: (pointer: number) => parse(abi().og_pointers_start(id, pointer)),
      count: () => abi().og_pointers_count(id),
    });
  }
  if (name === "og.Gamepad") {
    Object.assign(out, {
      connected: (pad?: number) => abi().og_gamepad_connected(id, pad),
      down: (button: string, pad?: number) => abi().og_gamepad_down(id, button, pad),
      pressed: (button: string, pad?: number) => abi().og_gamepad_pressed(id, button, pad),
      released: (button: string, pad?: number) => abi().og_gamepad_released(id, button, pad),
      axis: (axis: string, pad?: number) => abi().og_gamepad_axis(id, axis, pad),
    });
  }
  if (name === "og.Camera") {
    Object.assign(out, {
      lookAt: (target: unknown) => abi().og_camera_look_at(id, JSON.stringify(target)),
      worldToScreen: (point: unknown) => parse(abi().og_camera_world_to_screen(id, JSON.stringify(point))),
      screenToRay: (x: number, y: number) => parse(abi().og_camera_screen_to_ray(id, x, y)),
    });
  }
}

/** A resource read as JSON (og.Time, og.Rng, og.Input): computed by the engine, read only. */
function jsonResource(world: WorldImpl, token: Token, value: Record<string, unknown>): object {
  const id = world.id;
  if (token.name === "og.Rng") {
    return Object.freeze({
      get seed() { return parse<{ ok: boolean; value?: { seed: number } }>(abi().og_world_resource(id, "og.Rng")).value?.seed ?? 0; },
      get state() { return parse<{ ok: boolean; value?: { state: number } }>(abi().og_world_resource(id, "og.Rng")).value?.state ?? 0; },
      // A draw moves the stream: it needs og.Rng among the system's writes. A fork reads the seed only.
      next: () => (checkAccess("write", "og.Rng"), abi().og_rng_next(id)),
      range: (low: number, high: number) => (checkAccess("write", "og.Rng"), abi().og_rng_range(id, low, high)),
      integer: (low: number, high: number) => (checkAccess("write", "og.Rng"), abi().og_rng_integer(id, low, high)),
      chance: (p: number) => (checkAccess("write", "og.Rng"), abi().og_rng_chance(id, p)),
      fork: (label: string) => (checkAccess("read", "og.Rng"), rngStream(abi().og_rng_fork(id, label))),
    });
  }
  // Every getter reads through to the store, kept while no other engine
  // call or dispatch happened since (`engineCalls`).
  const name = token.name;
  let last = value;
  let at = engineCalls;
  const read = (): Record<string, unknown> => {
    if (at === engineCalls) return last;
    if (name !== "og.Time" && !checkAccess("read", name)) return last;
    const answer = parse<Result<Record<string, unknown>>>(host!.abi.og_world_resource(id, name));
    if (answer.ok) {
      last = answer.value;
      at = engineCalls;
    }
    return last;
  };
  const out: Record<string, unknown> = {};
  for (const key of Object.keys(value)) Object.defineProperty(out, key, { enumerable: true, get: () => read()[key] });
  methods(id, name, out);
  return Object.seal(out);
}

function rngStream(handle: number): object {
  return Object.freeze({
    get seed() { return abi().og_rng_stream_get_seed(handle); },
    next: () => abi().og_rng_stream_next(handle),
    range: (low: number, high: number) => abi().og_rng_stream_range(handle, low, high),
    integer: (low: number, high: number) => abi().og_rng_stream_integer(handle, low, high),
    chance: (p: number) => abi().og_rng_stream_chance(handle, p),
    fork: (label: string) => rngStream(abi().og_rng_stream_fork(handle, label)),
  });
}

// ── the world ─────────────────────────────────────────────────────────────

/** A structural change outside systems: every view handed out so far goes stale. */
function structural<T>(answer: T): T {
  if (!running) epoch++;
  return answer;
}

export class WorldImpl {
  readonly id: number;

  constructor(id: number) {
    this.id = id;
  }

  get tick(): number {
    return abi().og_world_get_tick(this.id);
  }

  spawn(desc?: unknown): Result<number> {
    return structural(parse(abi().og_world_spawn(this.id, json(desc))));
  }

  despawn(entity: number): Result<void> {
    return structural(parse(abi().og_world_despawn(this.id, entity)));
  }

  alive(entity: number): boolean {
    return abi().og_world_alive(this.id, entity);
  }

  has(entity: number, component: Token): boolean {
    return abi().og_world_has(this.id, entity, component.name);
  }

  insert(entity: number, component: Token, value?: unknown): Result<void> {
    return structural(parse(abi().og_world_insert(this.id, entity, component.name, json(value))));
  }

  remove(entity: number, component: Token): Result<void> {
    return structural(parse(abi().og_world_remove(this.id, entity, component.name)));
  }

  get(entity: number, component: Token): Result<unknown> {
    return parse(abi().og_world_get(this.id, entity, component.name));
  }

  set(entity: number, component: Token, value: unknown): Result<void> {
    return structural(parse(abi().og_world_set(this.id, entity, component.name, JSON.stringify(value ?? {}))));
  }

  resource(resource: Token): Result<object> {
    if (resource.name !== "og.Time") checkAccess("read", resource.name);
    // A resource with a byte layout: its handle, made once per world. The
    // read here is what refuses an unregistered or undeclared resource.
    const r = host ? recordOf(this.id, resource.name) : null;
    if (r && (r.dirty || r.at === engineCalls || refresh(r))) return { ok: true, value: recordHandle(this, r) };
    const answer = parse<Result<Record<string, unknown>>>(abi().og_world_resource(this.id, resource.name));
    if (!answer.ok) return answer;
    return { ok: true, value: r ? recordHandle(this, r) : jsonResource(this, resource, answer.value) };
  }

  find(name: string): number | null {
    return parse(abi().og_world_find(this.id, name));
  }

  snapshot(): Result<string> {
    return parse(abi().og_world_snapshot(this.id));
  }

  restore(save: string, desc?: unknown): Result<void> {
    return structural(parse(abi().og_world_restore(this.id, save, json(desc))));
  }

  nameOf(entity: number): string | null {
    return parse(abi().og_world_name_of(this.id, entity));
  }

  query(desc?: { with?: readonly Token[]; without?: readonly Token[] }): Result<QueryImpl> {
    const wire = desc && { with: (desc.with ?? []).map((t) => t.name), without: (desc.without ?? []).map((t) => t.name) };
    const answer = parse<Result<number>>(abi().og_world_query(this.id, json(wire)));
    if (!answer.ok) return answer;
    return { ok: true, value: new QueryImpl(answer.value, null) };
  }

  events(event: Token): readonly unknown[] {
    checkAccess("read", event.name);
    return parse(abi().og_world_events(this.id, event.name));
  }

  emit(event: Token, payload: unknown): Result<void> {
    checkAccess("write", event.name);
    return parse(abi().og_world_emit(this.id, event.name, JSON.stringify(payload ?? {})));
  }

  instantiate(asset: number, desc?: unknown): Result<number> {
    return structural(parse(abi().og_world_instantiate(this.id, asset, json(desc))));
  }
}

const worlds = new Map<number, WorldImpl>();

function worldFor(id: number): WorldImpl {
  let w = worlds.get(id);
  if (!w) worlds.set(id, (w = new WorldImpl(id)));
  return w;
}

const rates = new Map<number, number>();

// ── the dispatcher ────────────────────────────────────────────────────────

function message(e: unknown): string {
  const text = e instanceof Error ? (e.stack ?? e.message) : String(e);
  if (text.length === 0) return "game code threw an empty error";
  return text.startsWith("=") ? ` ${text}` : text;
}

/** What the engine calls: kind 0 a system, 1 a command, 2 the facts function. */
const dispatch: Dispatcher = (kind, engine, id, query, tick, dt, elapsed, payload) => {
  const fn = callbacks[id];
  const world = worldFor(engine);
  const outer = running;
  engineCalls++;
  inGame++;
  try {
    if (kind === 0) {
      const system = systems.get(id);
      if (!system || !fn) return `the dispatcher has no system with callback id ${id}: the bundle declared ${systems.size} system(s); was the engine built from another bundle?`;
      running = { system, tick };
      // The payload carries the time scale: scale,hitStop,scaledDt,scaledElapsed.
      const ts = (payload || "1,0,0,0").split(",").map(Number);
      const time = Object.freeze({
        tick, dt, elapsed, rate: rates.get(engine) ?? Math.round(1 / dt),
        scale: ts[0] ?? 1, hitStop: ts[1] ?? 0, scaledDt: ts[2] ?? dt, scaledElapsed: ts[3] ?? elapsed,
      });
      fn(new QueryImpl(query, system), time, world);
      flushRecords();
      return "";
    }
    if (!fn) return `the dispatcher has no callback with id ${id}`;
    if (kind === 1) {
      fn(world, Object.freeze(JSON.parse(payload || "[]")));
      flushRecords();
      return "";
    }
    if (kind === 2) {
      const facts = JSON.stringify(fn(world) ?? null);
      flushRecords();
      return "=" + facts;
    }
    return `unknown dispatch kind ${kind}`;
  } catch (e) {
    return message(e);
  } finally {
    // What game code wrote before it threw is written too, as a write
    // straight through would have been.
    if (dirtyRecords.length > 0) {
      try {
        flushRecords();
      } catch {
        // The throw already failed the call; this refusal adds nothing.
      }
    }
    inGame--;
    running = outer;
    epoch++;
    engineCalls++;
  }
};

// ── engines ───────────────────────────────────────────────────────────────

export interface CreateOptions {
  /** Overrides GameDesc.seed (a replay seed). */
  readonly seed?: number;
  /** The project directory (native) or URL base (web) whose `cooked/` bundle the engine opens before the first tick. */
  readonly root?: string;
  /** The physics profile, project.ogt's `profile`: `3d` (the default) or `2d`. */
  readonly profile?: "2d" | "3d";
  /**
   * project.ogt's text (`og run`, `og test`, `og dev`, `og polish` and `og
   * build` pass it): its `tick`, `seed`, `scenes` and `scene` fill the
   * GameDesc fields the game leaves unset, and one set in both places to
   * different values refuses the engine.
   */
  readonly project?: string;
  /** Run GameDesc.start at the first tick boundary instead of now, so a host can attach its renderer and report the viewport first (the web Worker does). */
  readonly deferStart?: boolean;
  /** The locale the host asks og.i18n for: the browser's language, `og run --locale`. */
  readonly locale?: string;
  /** The host mirrors the UI's accessibility tree (the web Worker): a frame's answer carries `a11y` when the tree changed. */
  readonly a11y?: boolean;
}

/** A file the command channel produced (`og_engine_take_outputs`). */
export interface CommandOutput {
  readonly tick: number;
  readonly action: string;
  /** Where to write it: the script's path, or `<action>-<tick>.<ext>`. */
  readonly file: string;
  readonly kind: "png" | "json" | "text" | "wav";
  /** Base64 for a PNG, the text otherwise. */
  readonly data: string;
}

/** One log line (`og_engine_take_log`). */
export interface LogLine {
  readonly tick: number;
  readonly warning: boolean;
  readonly text: string;
}

/** Start a new bundle's declarations before importing it again (hot reload). */
export function beginBundle(): void {
  abi().og_host_begin_bundle();
}

/**
 * The host's wall clock as og.platform hands it to the game (og.platform.now,
 * PlatformResult.time): read only between ticks, when the platform attaches
 * and when an answer arrives, so it reaches the world as host input.
 */
export interface HostClock {
  /** PlatformInfo.clock: `system`, `browser`, `fixed` or `player`. */
  readonly kind: string;
  /** UTC milliseconds since 1970. */
  now(): number;
}

/** The machine's clock: the native host's (`system`), the web host's (`browser`). */
export const systemClock = (kind = "system"): HostClock => ({ kind, now: () => Date.now() });

/** A clock that starts at `startMs` on tick 0 and moves with the engine's ticks (`og run --clock`, playtests): reproducible. */
export function fixedClock(engine: Engine, startMs: number): HostClock {
  return { kind: "fixed", now: () => startMs + (engine.world.tick * 1000) / (rates.get(engine.id) ?? 60) };
}

/** A running engine: one world, its probe, and the host's lifecycle calls. */
export class Engine {
  readonly id: number;
  readonly game: GameToken;
  /** The engine's world, as game code sees it. */
  readonly world: Api.World;
  /** The engine's probe: RuntimeProbe protocol 5 and the wire members. */
  readonly probe: Api.EngineProbe;

  #platform: PlatformHost = noPlatform;
  #clock: HostClock = systemClock();

  constructor(id: number, game: GameToken) {
    this.id = id;
    this.game = game;
    this.world = worldFor(id) as unknown as Api.World;
    this.probe = makeProbe(id) as unknown as Api.EngineProbe;
    // The rate the engine runs at: GameDesc.tick, else project.ogt's, else 60.
    const time = parse<Result<{ rate?: number }>>(abi().og_world_resource(id, "og.Time"));
    rates.set(id, (time.ok ? time.value.rate : undefined) ?? game.tick);
    learnBuild(id);
    this.attachPlatform(noPlatform);
  }

  /** The fixed tick rate the engine runs at, in hertz (GameDesc.tick, else project.ogt's `tick`, else 60). */
  get rate(): number {
    return rates.get(this.id) ?? this.game.tick;
  }

  /** Attach what answers og.platform's calls: the native host's local store, the web shell's Player bridge. */
  attachPlatform(host: PlatformHost): void {
    this.#platform = host;
    abi().og_platform_set_host(this.id, JSON.stringify({ host: host.host, capabilities: host.capabilities, time: host.time ?? this.#clock.now(), clock: host.time === undefined ? this.#clock.kind : "player" }));
  }

  /** Set the clock og.platform tells the game (before attachPlatform, which tells it the time). */
  setClock(clock: HostClock): void {
    this.#clock = clock;
    abi().og_platform_set_host(this.id, JSON.stringify({ host: this.#platform.host, capabilities: this.#platform.capabilities, time: clock.now(), clock: clock.kind }));
  }

  /** Hand the platform calls made since the last pump to the attached host (before and after every advance and frame, after a command). */
  pumpPlatform(): void {
    const calls = parse<PlatformCall[]>(abi().og_platform_take(this.id));
    if (calls.length > 0) this.#platform.handle(calls, (request, answer) => void this.answerPlatform(request, answer));
  }

  /** og.platform's calls the world has no answer to yet: in flight at the host, or answered and applied at the next tick's start. */
  platformPending(): number {
    return (parse<{ pending?: number }>(abi().og_platform_info(this.id)).pending ?? 0) as number;
  }

  /** Answer one platform call; applied at the start of the next tick. */
  answerPlatform(request: number, answer: PlatformAnswer): boolean {
    // An answer that carries no time (a Player before the time ask, the local platform) gets the host clock's.
    const timed = typeof answer.time === "number" ? { ...answer, clock: "player" } : { ...answer, time: this.#clock.now() };
    return abi().og_platform_answer(this.id, request, JSON.stringify(timed));
  }

  /** Take fixed steps: the tick reached, or the first failing system with its tick. */
  advance(steps: number): Result<number> {
    // Calls made between ticks are answered before the first of these ticks applies answers.
    this.pumpPlatform();
    const answer = parse<Result<number>>(abi().og_engine_advance(this.id, steps));
    epoch++;
    this.pumpPlatform();
    return answer;
  }

  /** Run a registered command action now, between ticks. */
  command(action: string, args: readonly string[] = []): Result<void> {
    const answer = parse<Result<void>>(abi().og_engine_command(this.id, action, JSON.stringify(args)));
    epoch++;
    this.pumpPlatform();
    return answer;
  }

  /** One display frame: tick by the wall clock (through the probe: due command steps run), then render. */
  frame(elapsedMs: number): {
    ok: boolean;
    ticks?: number;
    tick?: number;
    rendered?: boolean;
    paused?: boolean;
    quit?: boolean;
    behind?: number;
    /** Wall milliseconds a tick took (this frame's mean, or the last frame's that ticked) and the last render took. */
    stepMs?: number;
    renderMs?: number;
    /** The command channel's steps still to run (scheduled, or a screenshot waiting for its frame): a script has ended at 0, however a restore moved og.Time. */
    pending?: number;
    reason?: string;
    /** The UI's focused text control (web/text-input.ts's TextTarget), or null. */
    text?: { path: string; rect: [number, number, number, number]; multiline: boolean; selection: string; composing: boolean } | null;
    /** Text a copy or cut took this frame. */
    copy?: string;
    /** The UI's accessibility tree, on the frames where it changed, when the engine was created with `a11y` (web/a11y.ts mirrors it). */
    a11y?: Api.A11yTree;
    /** og.PointerLock.wanted: the game asks for the pointer lock (the web shell requests it on the next press on the canvas, and exits it when false). */
    pointerLock?: boolean;
  } {
    this.pumpPlatform();
    const answer = parse<{ ok: boolean; ticks?: number }>(abi().og_engine_frame(this.id, elapsedMs));
    epoch++;
    this.pumpPlatform();
    return answer;
  }

  /** The probe's JSON-RPC 2.0 entry: one request frame in, its answer frame out. */
  call(frame: string): string {
    const answer = abi().og_engine_call(this.id, frame);
    epoch++;
    return answer;
  }

  /** Files the command channel produced since the last call. */
  takeOutputs(): CommandOutput[] {
    return parse(abi().og_engine_take_outputs(this.id));
  }

  /** Log lines since the last call. */
  takeLog(): LogLine[] {
    return parse(abi().og_engine_take_log(this.id));
  }

  /** The world hash, hex: equal hashes mean equal worlds. */
  hash(): string {
    const dump = parse<{ hash?: string } | null>(abi().og_engine_probe_dump(this.id, JSON.stringify({ what: "world" })));
    return dump?.hash ?? "";
  }

  destroy(): boolean {
    epoch++;
    worlds.delete(this.id);
    forgetRecords(this.id);
    return abi().og_engine_destroy(this.id);
  }
}

/** Assemble an engine from a declared game (the bundle's default export). */
export function createEngine(game: GameToken, options: CreateOptions = {}): Result<Engine> {
  const handle = gameHandles.get(game);
  if (handle === undefined) return refused("not a game: pass the bundle's default export, the result of og.game()");
  const answer = parse<Result<number>>(abi().og_engine_create(handle, JSON.stringify(options)));
  if (!answer.ok) return answer;
  forgetRecords(answer.value);
  return { ok: true, value: new Engine(answer.value, game) };
}

/** A probe member's parameter as the binding table declares it. */
interface ProbeParam {
  readonly name: string;
  readonly type: string;
  readonly optional: boolean;
}

/**
 * The probe members' string and number parameters, with their positions. A
 * probe is driven by untyped callers (the engine Worker's `call("probe", …)`,
 * a console): on the web host a non-string where the ABI takes a string traps
 * the wasm inside wasm-bindgen's string passing (out-of-bounds memory), and
 * the native host throws a conversion error, so both hosts refuse it here,
 * before the host sees it, with the probe's own words ("script must be a
 * string").
 */
const PROBE_PARAMS = new Map(
  (bindings.calls as Array<{ module: string; kind: string; name: string; params?: ProbeParam[] }>)
    .filter((c) => c.module === "probe" && c.kind === "method")
    .map((c) => [
      c.name,
      (c.params ?? [])
        .map((p, at) => ({ ...p, at, want: p.type === "String" ? "string" : ["u32", "i32", "f32", "f64"].includes(p.type) ? "number" : "" }))
        .filter((p) => p.want !== ""),
    ]),
);

function checkedProbe<T extends Record<string, unknown>>(members: T): T {
  const out: Record<string, unknown> = {};
  for (const [name, member] of Object.entries(Object.getOwnPropertyDescriptors(members))) {
    const params = PROBE_PARAMS.get(name) ?? [];
    const f = member.value as ((...args: unknown[]) => unknown) | undefined;
    if (params.length === 0 || typeof f !== "function") {
      Object.defineProperty(out, name, member);
      continue;
    }
    out[name] = (...args: unknown[]) => {
      for (const p of params) {
        const v = args[p.at];
        if (v === undefined && p.optional) continue;
        if (typeof v !== p.want) throw new TypeError(`probe.${name}: ${p.name} must be a ${p.want}, not ${v === null ? "null" : typeof v}`);
      }
      return f(...args);
    };
  }
  return Object.freeze(out) as T;
}

function makeProbe(id: number) {
  const a = abi;
  return checkedProbe({
    get version() { return a().og_runtime_probe_get_version(id); },
    tick: () => a().og_runtime_probe_tick(id),
    state: () => parse(a().og_runtime_probe_state(id)),
    advance: (steps: number) => { const t = a().og_runtime_probe_advance(id, steps); epoch++; return t; },
    pause: () => a().og_runtime_probe_pause(id),
    resume: () => a().og_runtime_probe_resume(id),
    input: (event: unknown) => a().og_runtime_probe_input(id, JSON.stringify(event)),
    accepted: () => a().og_runtime_probe_accepted(id),
    pick: (x: number, y: number) => parse(a().og_runtime_probe_pick(id, x, y)),
    entity: (pid: string) => parse(a().og_runtime_probe_entity(id, pid)),
    bounds: (pid: string) => parse(a().og_runtime_probe_bounds(id, pid)),
    patch: (pid: string, parameter: string, value: unknown) => a().og_runtime_probe_patch(id, pid, parameter, JSON.stringify(value)),
    run: (program: unknown) => { const r = parse(a().og_runtime_probe_run(id, JSON.stringify(program))); epoch++; return r; },
    hello: () => parse(a().og_engine_probe_hello(id)),
    capture: (width?: number, height?: number) => parse(a().og_engine_probe_capture(id, width, height)),
    sweep: (desc?: unknown) => parse(a().og_engine_probe_sweep(id, json(desc))),
    reload: () => parse(a().og_engine_probe_reload(id)),
    snapshot: () => parse(a().og_engine_probe_snapshot(id)),
    restore: (sid: string) => { epoch++; return a().og_engine_probe_restore(id, sid); },
    dump: (desc?: unknown) => parse(a().og_engine_probe_dump(id, json(desc))),
    commands: (script: string) => parse(a().og_engine_probe_commands(id, script)),
    listen: (seconds?: number) => parse(a().og_engine_probe_listen(id, seconds)),
    memory: () => parse(a().og_engine_probe_memory(id)),
    describe: () => parse(a().og_engine_probe_describe(id)),
    expect: (what: string) => parse(a().og_engine_probe_expect(id, what)),
    expectations: () => parse(a().og_engine_probe_expectations(id)),
    saveScene: (write?: boolean, spawned?: boolean) => parse(a().og_engine_probe_save_scene(id, write, spawned)),
  });
}

// ── free functions ────────────────────────────────────────────────────────

export const transform = Object.freeze({
  compose: (position: unknown, quaternion: unknown, scale: unknown) =>
    parse(abi().og_transform_compose(JSON.stringify(position), JSON.stringify(quaternion), JSON.stringify(scale))),
  decompose: (matrix: unknown) => parse(abi().og_transform_decompose(JSON.stringify(matrix))),
  lookAt: (eye: unknown, target: unknown, up?: unknown) =>
    parse(abi().og_transform_look_at(JSON.stringify(eye), JSON.stringify(target), json(up))),
  fromAxisAngle: (axis: unknown, angle: number) => parse(abi().og_transform_from_axis_angle(JSON.stringify(axis), angle)),
});

export const render = Object.freeze({
  standardMaterial: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_standard_material(world.id, json(desc))),
  updateMaterial: (world: WorldImpl, material: number, desc?: unknown) =>
    parse(abi().og_render_update_material(world.id, material, json(desc))),
  capabilities: (world: WorldImpl) => parse(abi().og_render_capabilities(world.id)),
  applyLook: (world: WorldImpl, look: number) => parse(abi().og_render_apply_look(world.id, look)),
  applyQuality: (world: WorldImpl, preset: string) => parse(abi().og_render_apply_quality(world.id, preset)),
  look: (world: WorldImpl) => parse(abi().og_render_look(world.id)),
  text: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_text(world.id, json(desc))),
  sprite: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_sprite(world.id, json(desc))),
  ribbon: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_ribbon(world.id, json(desc))),
  measureText: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_measure_text(world.id, json(desc))),
  floatingText: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_floating_text(world.id, json(desc))),
  boxGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_box_geometry(world.id, json(desc))),
  planeGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_plane_geometry(world.id, json(desc))),
  sphereGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_sphere_geometry(world.id, json(desc))),
  shaderMaterial: (world: WorldImpl, shader: number, desc?: unknown) =>
    parse(abi().og_render_shader_material(world.id, shader, json(desc))),
  setMaterialShader: (world: WorldImpl, material: number, shader: number | null) =>
    parse(abi().og_render_set_material_shader(world.id, material, JSON.stringify(shader ?? null))),
  setShaderParam: (world: WorldImpl, material: number, name: string, value: ReadonlyArray<number>) =>
    parse(abi().og_render_set_shader_param(world.id, material, name, JSON.stringify(value))),
});

export const load = Object.freeze({
  bundle: (world: WorldImpl, path: string) => parse(abi().og_load_bundle(world.id, path)),
  bundleInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_bundle_info(world.id, asset)),
  defaultBundle: (world: WorldImpl) => parse(abi().og_load_default_bundle(world.id)),
  gltf: (world: WorldImpl, path: string, desc?: unknown) => parse(abi().og_load_gltf(world.id, path, json(desc))),
  scene: (world: WorldImpl, path: string) => parse(abi().og_load_scene(world.id, path)),
  font: (world: WorldImpl, path: string) => parse(abi().og_load_font(world.id, path)),
  fontInfo: (world: WorldImpl, font: number) => parse(abi().og_load_font_info(world.id, font)),
  sound: (world: WorldImpl, path: string) => parse(abi().og_load_sound(world.id, path)),
  texture: (world: WorldImpl, path: string) => parse(abi().og_load_texture(world.id, path)),
  textureArray: (world: WorldImpl, path: string) => parse(abi().og_load_texture_array(world.id, path)),
  lut: (world: WorldImpl, path: string) => parse(abi().og_load_lut(world.id, path)),
  environment: (world: WorldImpl, path: string) => parse(abi().og_load_environment(world.id, path)),
  emitter: (world: WorldImpl, path: string) => parse(abi().og_load_emitter(world.id, path)),
  shader: (world: WorldImpl, path: string) => parse(abi().og_load_shader(world.id, path)),
  style: (world: WorldImpl, path: string) => parse(abi().og_load_style(world.id, path)),
  look: (world: WorldImpl, path: string) => parse(abi().og_load_look(world.id, path)),
  lookInfo: (world: WorldImpl, look: number) => parse(abi().og_load_look_info(world.id, look)),
  palette: (world: WorldImpl, path: string) => parse(abi().og_load_palette(world.id, path)),
  soundInfo: (world: WorldImpl, sound: number) => parse(abi().og_load_sound_info(world.id, sound)),
  video: (world: WorldImpl, path: string) => parse(abi().og_load_video(world.id, path)),
  videoInfo: (world: WorldImpl, video: number) => parse(abi().og_load_video_info(world.id, video)),
  status: (world: WorldImpl, asset: number) => abi().og_load_status(world.id, asset),
  error: (world: WorldImpl, asset: number) => parse(abi().og_load_error(world.id, asset)),
  gltfInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_gltf_info(world.id, asset)),
  animation: (world: WorldImpl, path: string) => parse(abi().og_load_animation(world.id, path)),
  timeline: (world: WorldImpl, path: string) => parse(abi().og_load_timeline(world.id, path)),
  flipbook: (world: WorldImpl, path: string) => parse(abi().og_load_flipbook(world.id, path)),
  rig: (world: WorldImpl, path: string) => parse(abi().og_load_rig(world.id, path)),
  rig2d: (world: WorldImpl, path: string) => parse(abi().og_load_rig2d(world.id, path)),
  tilemap: (world: WorldImpl, path: string) => parse(abi().og_load_tilemap(world.id, path)),
});

export const anim = Object.freeze({
  graph: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_graph(world.id, json(desc))),
  humanoid: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_humanoid(world.id, json(desc))),
  retarget: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_retarget(world.id, json(desc))),
  mirror: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_mirror(world.id, json(desc))),
  rig2dInfo: (world: WorldImpl, rig: number) => parse(abi().og_anim_rig2d_info(world.id, rig)),
  skeleton2dBoxes: (world: WorldImpl, entity: number) => parse(abi().og_anim_skeleton2d_boxes(world.id, entity)),
  // One lookup for a flipbook's or a 2D rig's clips (an index, -1 when
  // missing) and a loaded model's (og.load.animation: a ClipAsset, 0 when missing).
  clip: (info: { clips?: ReadonlyArray<{ clip: number; name: string }>; skeleton?: number }, name: string): number =>
    info?.clips?.find((c) => c.name === name)?.clip ?? (info?.skeleton !== undefined ? 0 : -1),
});

// ── E4: time scale, camera feel, tweens, tilemaps ───────────────────────────

export const time = Object.freeze({
  setScale: (world: WorldImpl, scale: number) => parse(abi().og_time_set_scale(world.id, scale)),
  hitStop: (world: WorldImpl, ticks: number) => parse(abi().og_time_hit_stop(world.id, ticks)),
});

export const camera = Object.freeze({
  shake: (world: WorldImpl, trauma: number) => parse(abi().og_camera_shake(world.id, trauma)),
  punch: (world: WorldImpl, amount: number) => parse(abi().og_camera_punch(world.id, amount)),
  kick: (world: WorldImpl, direction: unknown, distance: number) =>
    parse(abi().og_camera_kick(world.id, JSON.stringify(direction), distance)),
});

export const tween = Object.freeze({
  to: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_to(world.id, json(desc))),
  sequence: (world: WorldImpl, steps: unknown, desc?: unknown) =>
    parse(abi().og_tween_sequence(world.id, JSON.stringify(steps), json(desc))),
  parallel: (world: WorldImpl, tweens: unknown, desc?: unknown) =>
    parse(abi().og_tween_parallel(world.id, JSON.stringify(tweens), json(desc))),
  cancel: (world: WorldImpl, t: number) => abi().og_tween_cancel(world.id, t),
  cancelEntity: (world: WorldImpl, entity: number) => abi().og_tween_cancel_entity(world.id, entity),
  active: (world: WorldImpl, t: number) => abi().og_tween_active(world.id, t),
  squash: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_squash(world.id, json(desc))),
  flash: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_flash(world.id, json(desc))),
});

export const tilemap = Object.freeze({
  setTile: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_set_tile(world.id, json(desc))),
  getTile: (world: WorldImpl, entity: number, layer: string, x: number, y: number) =>
    abi().og_tilemap_get_tile(world.id, entity, layer, x, y),
  cellAt: (world: WorldImpl, entity: number, point: unknown) =>
    parse(abi().og_tilemap_cell_at(world.id, entity, JSON.stringify(point))),
  cellCenter: (world: WorldImpl, entity: number, x: number, y: number) =>
    parse(abi().og_tilemap_cell_center(world.id, entity, x, y)),
});

export const commands = Object.freeze({
  register: registerCommand,
  parse: (script: string) => parse(abi().og_commands_parse(script)),
  schedule: (world: WorldImpl, script: string) => parse(abi().og_commands_schedule(world.id, script)),
  help: (world: WorldImpl) => parse(abi().og_commands_help(world.id)),
});

export const physics = Object.freeze({
  raycast: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_raycast(world.id, json(desc))),
  shapeCast: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_shape_cast(world.id, json(desc))),
  overlap: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_overlap(world.id, json(desc))),
  applyForce: (world: WorldImpl, entity: number, force: unknown, point?: unknown) =>
    parse(abi().og_physics_apply_force(world.id, entity, JSON.stringify(force), json(point))),
  applyImpulse: (world: WorldImpl, entity: number, impulse: unknown, point?: unknown) =>
    parse(abi().og_physics_apply_impulse(world.id, entity, JSON.stringify(impulse), json(point))),
  applyTorque: (world: WorldImpl, entity: number, torque: unknown) =>
    parse(abi().og_physics_apply_torque(world.id, entity, JSON.stringify(torque))),
  applyTorqueImpulse: (world: WorldImpl, entity: number, torque: unknown) =>
    parse(abi().og_physics_apply_torque_impulse(world.id, entity, JSON.stringify(torque))),
  setMesh: (world: WorldImpl, mesh: number, desc?: unknown) => parse(abi().og_physics_set_mesh(world.id, mesh, json(desc))),
  ropePoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_rope_points(world.id, entity)),
  clothPoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_cloth_points(world.id, entity)),
  fluidParticles: (world: WorldImpl, entity: number) => parse(abi().og_physics_fluid_particles(world.id, entity)),
  fluid3dParticles: (world: WorldImpl, entity: number) => parse(abi().og_physics_fluid3d_particles(world.id, entity)),
  waterHeight: (world: WorldImpl, entity: number, x: number, z: number) =>
    parse(abi().og_physics_water_height(world.id, entity, x, z)),
  vehicleWheels: (world: WorldImpl, entity: number) => parse(abi().og_physics_vehicle_wheels(world.id, entity)),
  ragdollRigFromSkeleton: (world: WorldImpl, skeleton: number) =>
    parse(abi().og_physics_ragdoll_rig_from_skeleton(world.id, skeleton)),
  setRagdollRig: (world: WorldImpl, entity: number, desc?: unknown) =>
    parse(abi().og_physics_set_ragdoll_rig(world.id, entity, json(desc))),
  ragdollPose: (world: WorldImpl, entity: number) => parse(abi().og_physics_ragdoll_pose(world.id, entity)),
  stats: (world: WorldImpl) => parse(abi().og_physics_stats(world.id)),
});

export const audio = Object.freeze({
  outputLatency: (world: WorldImpl) => (checkAccess("read", "og.Audio"), abi().og_audio_output_latency(world.id)),
});

export const atmosphere = Object.freeze({
  timeOfDay: (world: WorldImpl) => abi().og_atmosphere_time_of_day(world.id),
});

/** Runs `build` between an open and its close, closing even when it throws. */
function within<T>(open: () => void, build: () => void): T {
  open();
  try {
    build();
  } catch (e) {
    abi().og_ui_close();
    throw e;
  }
  return parse<T>(abi().og_ui_close());
}

export const ui = Object.freeze({
  configure: (world: WorldImpl, desc?: unknown) => parse(abi().og_ui_configure(world.id, json(desc))),
  viewport: (world: WorldImpl) => parse(abi().og_ui_viewport(world.id)),
  theme: (world: WorldImpl) => parse(abi().og_ui_theme(world.id)),
  open: (id: string, desc?: unknown) => abi().og_ui_open(id, json(desc)),
  openPanel: (id: string, desc?: unknown) => abi().og_ui_open_panel(id, json(desc)),
  openScroll: (id: string, desc?: unknown) => abi().og_ui_open_scroll(id, json(desc)),
  close: () => parse(abi().og_ui_close()),
  row: (id: string, desc: Record<string, unknown> | undefined, build: () => void) =>
    within(() => abi().og_ui_open(id, JSON.stringify({ ...(desc ?? {}), direction: "row" })), build),
  column: (id: string, desc: Record<string, unknown> | undefined, build: () => void) =>
    within(() => abi().og_ui_open(id, JSON.stringify({ ...(desc ?? {}), direction: "column" })), build),
  panel: (id: string, desc: unknown, build: () => void): void => {
    within(() => abi().og_ui_open_panel(id, json(desc)), build);
  },
  scroll: (id: string, desc: unknown, build: () => void): void => {
    within(() => abi().og_ui_open_scroll(id, json(desc)), build);
  },
  label: (id: string, text: string, desc?: unknown) => abi().og_ui_label(id, text, json(desc)),
  richText: (id: string, markup: string, desc?: unknown) => parse(abi().og_ui_rich_text(id, markup, json(desc))),
  button: (id: string, text: string, desc?: unknown) => parse(abi().og_ui_button(id, text, json(desc))),
  image: (id: string, desc?: unknown) => abi().og_ui_image(id, json(desc)),
  progress: (id: string, value: number, desc?: unknown) => abi().og_ui_progress(id, value, json(desc)),
  slider: (id: string, value: number, desc?: unknown) => parse(abi().og_ui_slider(id, value, json(desc))),
  toggle: (id: string, on: boolean, label: string) => parse(abi().og_ui_toggle(id, on, label)),
  textField: (id: string, value: string, desc?: unknown) => parse(abi().og_ui_text_field(id, value, json(desc))),
  textArea: (id: string, value: string, desc?: unknown) => parse(abi().og_ui_text_area(id, value, json(desc))),
  tooltip: (text: string) => abi().og_ui_tooltip(text),
  idi: (name: string, index: number) => `${name}#${index}`,
});
