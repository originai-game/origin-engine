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
import { noPlatform, type PlatformAnswer, type PlatformCall, type PlatformHost, type PlatformLink, type PlatformNotice } from "./platform-local.ts";
import { UiReactor } from "./ui/reactive.ts";
import { spawnSite } from "./spawn-site.ts";
import { decodeField, encodeField, type RecordField } from "./runtime/record-bytes.ts";
import { checkedProbe } from "./runtime/probe-params.ts";

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
let uiRegistrationErrors: string[] | null = null;
const uiRegistrationReads = new Set([
  "og_engine_ui_patch", "og_engine_resource_layout", "og_engine_read_resource",
  "og_world_resource", "og_world_get", "og_world_has", "og_world_find",
  "og_world_name_of", "og_world_alive", "og_world_get_tick",
]);
const abiReturns = new Map(bindings.calls.filter(call => "symbol" in call)
  .map(call => [call.symbol, call.returns]));

function registrationBoundary(abi: HostAbi<Uint32Array, unknown>): HostAbi<Uint32Array, unknown> {
  return new Proxy(abi, {
    get(target, property, receiver) {
      if (!uiRegistrationErrors || typeof property !== "string" || uiRegistrationReads.has(property)) {
        return Reflect.get(target, property, receiver);
      }
      return () => {
        const reason = `GameDesc.ui may only register presentation data and read the world: ${property} is not allowed`;
        uiRegistrationErrors?.push(reason);
        if (abiReturns.get(property)?.startsWith("Result<")) return JSON.stringify(refused(reason));
        // Void/property APIs have no Result to return. The dispatcher catches
        // this and returns the registration failure to both hosts as a value.
        throw new Error(reason);
      };
    },
  });
}

/** Bind the runtime to a host and flush the declarations made so far. */
export function bindHost(binding: HostBinding): void {
  const missing = SYMBOLS.filter((s) => typeof (binding.abi as unknown as Record<string, unknown>)[s] !== "function");
  if (missing.length > 0) {
    throw new Error(`the ${binding.name} host lacks ${missing.length} binding symbol(s): ${missing.join(", ")}; rebuild it from the same api/<module>.ogt`);
  }
  // Entity arguments are checked here, so null or undefined is refused
  // naming the call on both hosts alike (generated/abi.ts).
  host = { ...binding, abi: registrationBoundary(checkEntities(binding.abi)) };
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

/** A CSS `cursor: url()` image the engine read from its texture: row-major RGBA8 (sRGB, straight alpha) in base64, and the pixel that is the pointer's position. */
export interface CursorImage {
  width: number;
  height: number;
  rgba: string;
  hotspot: [number, number];
}

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

/** A field factory's spec (`f32()`, `Vector3()`): frozen, as declarations keep it. */
export const fieldSpec = (kind: string, value: unknown, options: unknown): FieldSpec =>
  Object.freeze({ kind, default: value, options: Object.freeze({ ...((options as object | undefined) ?? {}) }) });

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

export function defineRecord(kind: Token["kind"], name: string, fields: Record<string, FieldSpec>): Token {
  const token: Token = Object.freeze({ kind, name, fields: Object.freeze({ ...fields }) });
  declare((a) => {
    if (kind === "component") a.og_reflect_define_component(name, JSON.stringify(fieldList(fields)));
    else if (kind === "resource") a.og_reflect_define_resource(name, JSON.stringify(fieldList(fields)));
    else a.og_reflect_define_event(name, JSON.stringify(fieldList(fields)));
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
  readonly changed?: readonly Token[];
  readonly added?: readonly Token[];
  readonly removed?: readonly Token[];
  readonly state?: string;
  readonly value?: string;
  readonly when?: "during" | "enter" | "exit";
  readonly runIf?: Function | null;
}

/** Run conditions by callback id, with the system they gate. */
const conditions = new Map<number, SystemEntry>();
/** Hooks by callback id. */
const hookEntries = new Map<number, SystemEntry>();
const hookHandles = new WeakMap<object, number>();

export interface HookToken {
  readonly name: string;
}

export interface HookDescIn {
  readonly on?: "add" | "remove";
  readonly reads?: readonly Token[];
  readonly writes?: readonly Token[];
}

export function defineHook(name: string, component: Token, desc: HookDescIn, run: Function): HookToken {
  const reads = [...(desc.reads ?? [])];
  const writes = [...(desc.writes ?? [])];
  const id = callback(run);
  hookEntries.set(id, { name, reads: new Set(reads.map((t) => t.name)), writes: new Set(writes.map((t) => t.name)) });
  const wire = JSON.stringify({ on: desc.on ?? "add", reads: reads.map(tokenRef), writes: writes.map(tokenRef) });
  const token = Object.freeze({ name });
  declare((a) => hookHandles.set(token, a.og_system_define_hook(name, component.name, wire, id)));
  return token;
}

const tokenRef = (t: Token) => ({ kind: t.kind, name: t.name });

export function defineSystem(name: string, desc: SystemDescIn, run: Function): SystemToken {
  const reads = [...(desc.reads ?? [])];
  const writes = [...(desc.writes ?? [])];
  const filters = [...(desc.changed ?? []), ...(desc.added ?? []), ...(desc.removed ?? [])];
  const id = callback(run);
  // The change filters' components are read (the engine declares them so).
  const entry: SystemEntry = {
    name,
    reads: new Set([...reads, ...filters].map((t) => t.name)),
    writes: new Set(writes.map((t) => t.name)),
  };
  systems.set(id, entry);
  const runIf = desc.runIf ? callback(desc.runIf) : null;
  if (runIf !== null) conditions.set(runIf, entry);
  const wire = JSON.stringify({
    reads: reads.map(tokenRef),
    writes: writes.map(tokenRef),
    without: (desc.without ?? []).map(tokenRef),
    after: desc.after ?? [],
    before: desc.before ?? [],
    changed: (desc.changed ?? []).map(tokenRef),
    added: (desc.added ?? []).map(tokenRef),
    removed: (desc.removed ?? []).map(tokenRef),
    state: desc.state ?? "",
    value: desc.value ?? "",
    when: desc.when ?? "during",
    runIf,
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
const gameLooks = new WeakMap<object, string>();
const gameExplore = new WeakMap<object, Readonly<Record<string, unknown>>>();

/** GameDesc.explore of a game token, for `og explore` (never the engine's). */
export function declaredGameExplore(game: object): Readonly<Record<string, unknown>> {
  return gameExplore.get(game) ?? {};
}

/** Tooling reads declarations without creating a world or running GameDesc.start. */
export function declaredGameLook(game: object): string {
  const look = gameLooks.get(game);
  if (look === undefined) throw new Error("not a game: expected the result of og.game()");
  return look;
}

export interface GameDescIn {
  readonly tick?: number;
  readonly seed?: number;
  readonly scenes?: Readonly<Record<string, string>>;
  readonly scene?: string;
  readonly systems?: readonly SystemToken[];
  readonly hooks?: readonly HookToken[];
  readonly states?: readonly { readonly name?: string; readonly values?: readonly string[]; readonly initial?: string }[];
  readonly resources?: readonly Token[];
  readonly commands?: Readonly<Record<string, Function>>;
  readonly commandHelp?: Readonly<Record<string, { readonly usage?: string; readonly doc?: string }>>;
  readonly maxCatchUp?: number;
  readonly facts?: Function | null;
  readonly start?: Function | null;
  readonly ui?: Function | null;
  readonly render?: Readonly<Record<string, unknown>>;
  readonly look?: string;
  readonly explore?: Readonly<Record<string, unknown>>;
  readonly saveVersion?: number;
  readonly migrations?: readonly SaveMigrationIn[];
}

export interface SaveMigrationIn {
  readonly from: number;
  readonly migrate: (save: SaveEditImpl) => Result<void> | void;
}

/** Each game's save migrations (GameDesc.migrations), by its token; the steps run here, before world.restore reaches the host. */
const gameMigrations = new WeakMap<object, readonly SaveMigrationIn[]>();
/** The game an engine runs, for its world's restores. */
const engineGames = new Map<number, object>();
/** Each engine's CreateOptions.root: the project a spawn's source is relative to. */
const engineRoots = new Map<number, string>();
/** The root of the engine being created: GameDesc.start (and the boot scene's hooks) spawn before its id is known. */
let creatingRoot: string | undefined;

/** The og.game() token engine `id` runs (og.assert reads its game's sites by it). */
export function engineGame(id: number): object | undefined {
  return engineGames.get(id);
}

export function defineGame(desc: GameDescIn = {}): GameToken {
  const list = [...(desc.systems ?? [])];
  const commands: Record<string, number> = {};
  for (const [name, fn] of Object.entries(desc.commands ?? {})) commands[name] = callback(fn);
  const facts = desc.facts ? callback(desc.facts) : null;
  const start = desc.start ? callback(desc.start) : null;
  const ui = desc.ui ? callback(desc.ui) : null;
  const token = Object.freeze({ tick: desc.tick ?? 60, systems: Object.freeze(list) });
  gameLooks.set(token, desc.look ?? "");
  gameExplore.set(token, desc.explore ?? {});
  const migrations = [...(desc.migrations ?? [])];
  gameMigrations.set(token, migrations);
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
      hooks: (desc.hooks ?? []).map((h) => hookHandles.get(h) ?? -1),
      states: (desc.states ?? []).map((s) => ({ name: s.name ?? "", values: [...(s.values ?? [])], initial: s.initial ?? "" })),
      resources: (desc.resources ?? []).map((r) => r.name),
      commands,
      commandHelp: desc.commandHelp ?? {},
      maxCatchUp: desc.maxCatchUp ?? 4,
      facts,
      start,
      ui,
      render: desc.render ?? {},
      look: desc.look ?? "",
      saveVersion: desc.saveVersion ?? null,
      migrations: migrations.map((m) => m.from),
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

/** The engine and tick of the call into game code in progress (og.assert's), or null between calls. */
let calling: { readonly engine: number; readonly tick: number; readonly world: object } | null = null;

/** The engine, its world and the tick game code is running in now: og.assert records into it. */
export function currentCall(): { readonly engine: number; readonly tick: number; readonly world: object } | null {
  return calling;
}

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

  #rows: Uint32Array | null = null;

  /** The rows the query's change filters pass (every row without filters). */
  get rows(): Uint32Array {
    this.#check();
    return (this.#rows ??= abi().og_archetype_view_get_rows(this.#info.view));
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

/** Whether the bound host is a publish build, once an engine was made (null before). */
export function isPublishBuild(): boolean | null {
  return publishBuild;
}

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
  if (access === "write" && uiRegistrationErrors) {
    uiRegistrationErrors.push(`GameDesc.ui cannot write '${name}'; gameplay setup belongs in start`);
    return false;
  }
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

/** The world's copy of a resource with a byte layout, or null for one read as JSON (og.Time, og.Rng). */
function recordOf(world: number, name: string): RecordCopy | null {
  let byName = records.get(world);
  if (!byName) records.set(world, (byName = new Map()));
  const known = byName.get(name);
  if (known !== undefined) return known;
  const layout = recordLayout(world, name);
  // An unregistered name is asked again next time (a reload may declare it).
  if (!layout) {
    if (name === "og.Time" || name === "og.Rng") byName.set(name, null);
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
      // Parameterized timing is serialized by the shared Rust parser/codec.
      // abi() flushes pending packed writes before reading the same record.
      if (field().kind === "ease") {
        if (!checkAccess("read", name)) return lastValue;
        const answer = parse<Result<Record<string, unknown>>>(abi().og_world_resource(id, name));
        if (!answer.ok) throw new Error(`${name}: ${answer.reason}`);
        return (lastValue = answer.value[key]);
      }
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
      if (field().kind === "ease" || !refresh(r)) {
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

/** A resource read as JSON (og.Time, og.Rng): computed by the engine, read only. */
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
      weighted: (weights: readonly number[]) =>
        (checkAccess("write", "og.Rng"), abi().og_rng_weighted(id, JSON.stringify(weights))),
      permutation: (n: number) => (checkAccess("write", "og.Rng"), parse<number[]>(abi().og_rng_permutation(id, n))),
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
    weighted: (weights: readonly number[]) => abi().og_rng_stream_weighted(handle, JSON.stringify(weights)),
    permutation: (n: number) => parse<number[]>(abi().og_rng_stream_permutation(handle, n)),
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
    // SpawnDesc.source: the game's line that called spawn, outside a publish build (spawn-site.ts).
    if (publishBuild !== true && (desc == null || (typeof desc === "object" && !(desc as { source?: unknown }).source))) {
      const source = spawnSite(engineRoots.get(this.id) ?? creatingRoot);
      if (source !== null) desc = { ...(desc as object | undefined), source };
    }
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

  snapshot(desc?: { with?: readonly Token[]; resources?: readonly Token[] }): Result<string> {
    const wire = desc && { with: (desc.with ?? []).map((t) => t.name), resources: (desc.resources ?? []).map((t) => t.name) };
    return parse(abi().og_world_snapshot(this.id, json(wire)));
  }

  openSave(save: string): Result<SaveEditImpl> {
    const answer = parse<Result<number>>(abi().og_world_open_save(this.id, save));
    if (!answer.ok) return answer;
    return { ok: true, value: new SaveEditImpl(answer.value) };
  }

  restore(save: string, desc?: unknown): Result<void> {
    const game = engineGames.get(this.id);
    const steps = (game && gameMigrations.get(game)) ?? [];
    if (steps.length > 0) {
      // The game's steps from the save's version on, in order (the host
      // refused a game whose steps reach past saveVersion or repeat one).
      const opened = this.openSave(save);
      if (!opened.ok) return opened;
      const edit = opened.value;
      try {
        for (const step of [...steps].sort((a, b) => a.from - b.from)) {
          if (step.from < edit.version) continue;
          let r: Result<void> | void;
          try {
            r = step.migrate(edit);
          } catch (e) {
            return { ok: false, reason: `the save migration from version ${step.from} threw: ${message(e)}` };
          }
          if (r && r.ok === false) return { ok: false, reason: `the save migration from version ${step.from} refused: ${r.reason}` };
        }
        const text = edit.text();
        if (!text.ok) return text;
        save = text.value;
      } finally {
        edit.close();
      }
    }
    const restored = structural(parse<Result<void>>(abi().og_world_restore(this.id, save, json(desc))));
    if (restored.ok) {
      const mounts = uiReactors.get(this.id)?.restoreMounts();
      if (mounts && !mounts.ok) return mounts;
    }
    return restored;
  }

  prefabOf(entity: number): unknown {
    return parse(abi().og_world_prefab_of(this.id, entity));
  }

  nameOf(entity: number): string | null {
    return parse(abi().og_world_name_of(this.id, entity));
  }

  children(entity: number): number[] {
    return parse(abi().og_world_children(this.id, entity));
  }

  removed(component: Token): Result<number[]> {
    checkAccess("read", component.name);
    return parse(abi().og_world_removed(this.id, component.name));
  }

  spatial(desc?: { with?: readonly Token[]; without?: readonly Token[]; cell?: number }): Result<SpatialIndexImpl> {
    const wire = desc && { with: (desc.with ?? []).map((t) => t.name), without: (desc.without ?? []).map((t) => t.name), cell: desc.cell ?? 0 };
    const answer = parse<Result<number>>(abi().og_world_spatial(this.id, json(wire)));
    if (!answer.ok) return answer;
    return { ok: true, value: new SpatialIndexImpl(answer.value) };
  }

  state(name: string): string | null {
    return parse(abi().og_world_state(this.id, name));
  }

  setState(name: string, value: string): Result<void> {
    return parse(abi().og_world_set_state(this.id, name, value));
  }

  query(desc?: { with?: readonly Token[]; without?: readonly Token[]; changed?: readonly Token[]; added?: readonly Token[]; removed?: readonly Token[] }): Result<QueryImpl> {
    const names = (l?: readonly Token[]) => (l ?? []).map((t) => t.name);
    const wire = desc && { with: names(desc.with), without: names(desc.without), changed: names(desc.changed), added: names(desc.added), removed: names(desc.removed) };
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

/** A spatial hash of entities' positions (World.spatial, SpatialIndex). */
export class SpatialIndexImpl {
  readonly #id: number;

  constructor(id: number) {
    this.#id = id;
  }

  get count(): number {
    return abi().og_spatial_index_get_count(this.#id);
  }

  within(center: { x: number; y: number; z: number }, radius: number): { entity: number; distance: number }[] {
    return parse(abi().og_spatial_index_within(this.#id, JSON.stringify(center), radius));
  }

  nearest(center: { x: number; y: number; z: number }, count: number): { entity: number; distance: number }[] {
    return parse(abi().og_spatial_index_nearest(this.#id, JSON.stringify(center), count));
  }
}

/** A save open for editing (World.openSave, SaveEdit). */
export class SaveEditImpl {
  readonly #id: number;

  constructor(id: number) {
    this.#id = id;
  }

  get version(): number {
    return abi().og_save_edit_get_version(this.#id);
  }

  types(): Result<readonly { name: string; version: number; fields: readonly string[] }[]> {
    return parse(abi().og_save_edit_types(this.#id));
  }

  has(name: string): boolean {
    const r = this.types();
    return r.ok && r.value.some((t) => t.name === name);
  }

  records(name: string): Result<readonly { entity: number | null; value: Record<string, unknown> }[]> {
    return parse(abi().og_save_edit_records(this.#id, name));
  }

  setRecords(name: string, desc?: { into?: string; records?: readonly { entity: number | null; value: unknown }[] }): Result<void> {
    return parse(abi().og_save_edit_set_records(this.#id, name, json(desc)));
  }

  transform(name: string, map: (value: Record<string, unknown>, entity: number | null) => unknown, desc?: { into?: string }): Result<void> {
    const old = this.records(name);
    if (!old.ok) return old;
    let records;
    try {
      records = old.value.map((r) => ({ entity: r.entity, value: map(r.value, r.entity) ?? {} }));
    } catch (e) {
      return { ok: false, reason: `SaveEdit.transform of ${name}: ${message(e)}` };
    }
    return this.setRecords(name, { into: desc?.into ?? "", records });
  }

  split(name: string, map: (value: Record<string, unknown>, entity: number | null) => Record<string, unknown> | null): Result<void> {
    const old = this.records(name);
    if (!old.ok) return old;
    const parts = new Map<string, { entity: number | null; value: unknown }[]>();
    try {
      for (const r of old.value) {
        for (const [type, value] of Object.entries(map(r.value, r.entity) ?? {})) {
          if (value === null || value === undefined) continue;
          let list = parts.get(type);
          if (!list) parts.set(type, (list = []));
          list.push({ entity: r.entity, value });
        }
      }
    } catch (e) {
      return { ok: false, reason: `SaveEdit.split of ${name}: ${message(e)}` };
    }
    // The old type goes, written as the first new one; the others are
    // added. An edit refused part way leaves the earlier ones in the open
    // save, which the restore then never takes.
    const names = [...parts.keys()];
    if (names.length === 0) return this.dropType(name);
    const [first, ...rest] = names;
    for (const type of rest) {
      if (this.has(type)) return { ok: false, reason: `SaveEdit.split of ${name}: the save already holds ${type}` };
    }
    const r = this.setRecords(name, { into: first, records: parts.get(first!)! });
    if (!r.ok) return r;
    // A type the save does not hold: setRecords adds it.
    for (const type of rest) {
      const added = this.setRecords(type, { records: parts.get(type)! });
      if (!added.ok) return added;
    }
    return { ok: true, value: undefined };
  }

  renameType(from: string, to: string): Result<void> {
    return parse(abi().og_save_edit_rename_type(this.#id, from, to));
  }

  renameField(name: string, from: string, to: string): Result<void> {
    return parse(abi().og_save_edit_rename_field(this.#id, name, from, to));
  }

  dropType(name: string): Result<void> {
    return parse(abi().og_save_edit_drop_type(this.#id, name));
  }

  spawn(desc?: { name?: string; components?: Record<string, unknown> }): Result<number> {
    return parse(abi().og_save_edit_spawn(this.#id, json(desc)));
  }

  text(): Result<string> {
    return parse(abi().og_save_edit_text(this.#id));
  }

  close(): void {
    abi().og_save_edit_close(this.#id);
  }
}

const worlds = new Map<number, WorldImpl>();
const uiReactors = new Map<number, UiReactor>();

function uiReactor(world: WorldImpl): UiReactor {
  let reactor = uiReactors.get(world.id);
  if (!reactor) {
    reactor = new UiReactor(bytes => abi().og_engine_ui_patch(world.id, bytes));
    uiReactors.set(world.id, reactor);
  }
  return reactor;
}

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

const engines = new Map<number, Engine>();

/** Dispatch kinds: system, command, facts, condition, hook, platform, UI registration. */
const dispatch: Dispatcher = (kind, engine, id, query, tick, dt, elapsed, payload) => {
  if (kind === 5) {
    try {
      engines.get(engine)?.pumpPlatform();
      return "";
    } catch (e) {
      return message(e);
    }
  }
  const fn = callbacks[id];
  const world = worldFor(engine);
  const outer = running;
  const outerCall = calling;
  calling = { engine, tick, world };
  engineCalls++;
  inGame++;
  try {
    if (kind === 6) {
      const errors: string[] = [];
      const outerRegistration = uiRegistrationErrors;
      uiRegistrationErrors = errors;
      try {
        const result = uiReactor(world).registerPresentation(() => { fn?.(world); });
        if (!result.ok) errors.push(result.reason);
        return errors.join("\n");
      } finally { uiRegistrationErrors = outerRegistration; }
    }
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
    if (kind === 3 || kind === 4) {
      const entry = (kind === 3 ? conditions : hookEntries).get(id);
      if (!entry) return `the dispatcher has no ${kind === 3 ? "run condition" : "hook"} with callback id ${id}`;
      running = { system: entry, tick };
      if (kind === 3) {
        const holds = fn(world) === true;
        flushRecords();
        return holds ? "=true" : "=false";
      }
      fn(world, Number(payload));
      flushRecords();
      return "";
    }
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
    calling = outerCall;
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
   * project.ogt's text (`og run`, `og test`, `og dev` and `og build` pass
   * it): its `tick`, `seed`, `scenes` and `scene` fill the
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
    engines.set(id, this);
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
    const host = this.#platform;
    if (calls.length === 0 && !host.poll) return;
    const link = this.#link;
    if (calls.length > 0) host.handle(calls, link.answer, link);
    host.poll?.(link);
  }

  /** What the attached host may do besides answering: notices, and pausing the game while an ad shows. */
  readonly #link: PlatformLink = (() => {
    const engine = this;
    return {
      get tick() {
        return engine.world.tick;
      },
      answer: (request: number, answer: PlatformAnswer) => void engine.answerPlatform(request, answer),
      notify: (notice: PlatformNotice) => void engine.notifyPlatform(notice),
      pause: () => void engine.probe.pause(),
      resume: () => void engine.probe.resume(),
    };
  })();

  /** Unasked host input (a purchase settled, entitlements changed, an invite): applied at the start of the next tick as og.PlatformNotice. */
  notifyPlatform(notice: PlatformNotice): boolean {
    const timed = typeof notice.time === "number" ? { ...notice, clock: "player" } : { ...notice, time: this.#clock.now() };
    return abi().og_platform_notify(this.id, JSON.stringify(timed));
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
    const answer = parse<Result<number>>(abi().og_engine_advance(this.id, steps));
    epoch++;
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
    /** The haptic patterns the UI's feedback played since the last frame, at the player's strength: [startMs, ms, low, high] pulses (web/haptics.ts plays them). */
    haptics?: [number, number, number, number][][];
    /** og.input.rumble's requests since the last frame: [pad (-1 every pad), low, high, ms] (web/gamepad.ts playRumble plays them). */
    rumble?: [number, number, number, number][];
    /** The UI's accessibility tree, on the frames where it changed, when the engine was created with `a11y` (web/a11y.ts mirrors it). */
    a11y?: Api.A11yTree;
    /** og.PointerLock.wanted: the game asks for the pointer lock (the web shell requests it on the next press on the canvas, and exits it when false). */
    pointerLock?: boolean;
    /** The pointer's shape the UI resolved over what it is on (a CSS cursor name: og.ui.frameCursor). */
    cursor?: string;
    /** On the frames where it changed: the first of the cursor's CSS `url()` images that loads, row-major RGBA8 in base64 with its hotspot, or null for none (`cursor` is its fallback). */
    cursorImage?: CursorImage | null;
    /** The frame cap og.render.setDisplay resolved to (0: the display's rate): the web Worker draws at most this many frames a second (web/pacing.ts). */
    maxFps?: number;
    /** The player's graphics settings ask the page to be fullscreen (og.render.setDisplay's `fullscreen`; the web shell enters or leaves it). */
    fullscreen?: boolean;
    /** The power preference the next load's GPU adapter is asked for: low power (the web shell keeps it in the page's storage). */
    lowPower?: boolean;
    /** Ask pointer lock for raw mouse movement (og.input.setMouse's `raw`: unadjustedMovement). */
    rawMouse?: boolean;
    /** The scene texts the inspector's Save (or a prefab Apply or Revert) made on a host that writes no files (development builds): the web shell routes them. */
    saved?: { scenes: { source: string; text: string }[] };
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
    return parse<LogLine[]>(abi().og_engine_take_log(this.id));
  }

  /** The world hash, hex: equal hashes mean equal worlds. */
  hash(): string {
    const dump = parse<{ hash?: string } | null>(abi().og_engine_probe_dump(this.id, JSON.stringify({ what: "world" })));
    return dump?.hash ?? "";
  }

  destroy(): boolean {
    epoch++;
    for (const scope of uiReactors.get(this.id)?.scopes.values() ?? []) scope.dispose();
    uiReactors.delete(this.id);
    engines.delete(this.id);
    worlds.delete(this.id);
    engineGames.delete(this.id);
    engineRoots.delete(this.id);
    forgetRecords(this.id);
    return abi().og_engine_destroy(this.id);
  }
}

/** Assemble an engine from a declared game (the bundle's default export). */
export function createEngine(game: GameToken, options: CreateOptions = {}): Result<Engine> {
  const handle = gameHandles.get(game);
  if (handle === undefined) return refused("not a game: pass the bundle's default export, the result of og.game()");
  creatingRoot = options.root;
  let answer: Result<number>;
  try {
    answer = parse<Result<number>>(abi().og_engine_create(handle, JSON.stringify(options)));
  } finally {
    creatingRoot = undefined;
  }
  if (!answer.ok) return answer;
  forgetRecords(answer.value);
  engineGames.set(answer.value, game);
  if (options.root !== undefined) engineRoots.set(answer.value, options.root);
  return { ok: true, value: new Engine(answer.value, game) };
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
    capture: (width?: number, height?: number, fit?: number, region?: unknown) =>
      parse(a().og_engine_probe_capture(id, width, height, fit, json(region ?? undefined))),
    marks: (desc?: unknown) => parse(a().og_engine_probe_marks(id, json(desc))),
    timeline: () => parse(a().og_engine_probe_timeline(id)),
    sweep: (desc?: unknown) => parse(a().og_engine_probe_sweep(id, json(desc))),
    reload: () => parse(a().og_engine_probe_reload(id)),
    snapshot: () => parse(a().og_engine_probe_snapshot(id)),
    restore: (sid: string) => {
      epoch++;
      const restored = a().og_engine_probe_restore(id, sid);
      uiReactors.get(id)?.restoreMounts();
      return restored;
    },
    dump: (desc?: unknown) => parse(a().og_engine_probe_dump(id, json(desc))),
    commands: (script: string) => parse(a().og_engine_probe_commands(id, script)),
    listen: (seconds?: number) => parse(a().og_engine_probe_listen(id, seconds)),
    memory: () => parse(a().og_engine_probe_memory(id)),
    describe: () => parse(a().og_engine_probe_describe(id)),
    expect: (what: string) => parse(a().og_engine_probe_expect(id, what)),
    expectations: () => parse(a().og_engine_probe_expectations(id)),
    assertions: (take?: boolean) => parse(a().og_engine_probe_assertions(id, take)),
    assertionEvents: () => parse(a().og_engine_probe_assertion_events(id)),
    observe: (records?: readonly string[] | null, types?: boolean) => parse(a().og_engine_probe_observe(id, json(records ?? undefined), types)),
    saveScene: (write?: boolean, spawned?: boolean) => parse(a().og_engine_probe_save_scene(id, write, spawned)),
    pickUi: (x: number, y: number) => parse(a().og_engine_probe_pick_ui(id, x, y)),
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
  setDisplay: (world: WorldImpl, settings?: unknown) => parse(abi().og_render_set_display(world.id, json(settings))),
  resetDisplay: (world: WorldImpl) => parse(abi().og_render_reset_display(world.id)),
  display: (world: WorldImpl) => parse(abi().og_render_display(world.id)),
  look: (world: WorldImpl) => parse(abi().og_render_look(world.id)),
  snap: (world: WorldImpl, entity: number) => parse(abi().og_render_snap(world.id, entity)),
  snapCamera: (world: WorldImpl) => abi().og_render_snap_camera(world.id),
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
  imageData: (world: WorldImpl, name: string, bytes: Uint8Array, desc?: unknown) => parse(abi().og_load_image_data(world.id, name, bytes, json(desc))),
  gltfData: (world: WorldImpl, name: string, bytes: Uint8Array, desc?: unknown) => parse(abi().og_load_gltf_data(world.id, name, bytes, json(desc))),
  environmentData: (world: WorldImpl, name: string, bytes: Uint8Array, desc?: unknown) => parse(abi().og_load_environment_data(world.id, name, bytes, json(desc))),
  environmentInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_environment_info(world.id, asset)),
  release: (world: WorldImpl, asset: number) => parse(abi().og_load_release(world.id, asset)),
  imageInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_image_info(world.id, asset)),
  completions: (world: WorldImpl) => parse(abi().og_load_completions(world.id)),
  replayCompletions: (world: WorldImpl, completions: unknown) => parse(abi().og_load_replay_completions(world.id, JSON.stringify(completions))),
  scene: (world: WorldImpl, path: string) => parse(abi().og_load_scene(world.id, path)),
  prefab: (world: WorldImpl, path: string) => parse(abi().og_load_prefab(world.id, path)),
  font: (world: WorldImpl, path: string) => parse(abi().og_load_font(world.id, path)),
  fontInfo: (world: WorldImpl, font: number) => parse(abi().og_load_font_info(world.id, font)),
  sound: (world: WorldImpl, path: string) => parse(abi().og_load_sound(world.id, path)),
  texture: (world: WorldImpl, path: string) => parse(abi().og_load_texture(world.id, path)),
  textureArray: (world: WorldImpl, path: string) => parse(abi().og_load_texture_array(world.id, path)),
  lut: (world: WorldImpl, path: string) => parse(abi().og_load_lut(world.id, path)),
  environment: (world: WorldImpl, path: string) => parse(abi().og_load_environment(world.id, path)),
  emitter: (world: WorldImpl, path: string) => parse(abi().og_load_emitter(world.id, path)),
  shader: (world: WorldImpl, path: string) => parse(abi().og_load_shader(world.id, path)),
  vector: (world: WorldImpl, path: string) => parse(abi().og_load_vector(world.id, path)),
  uiMaterial: (world: WorldImpl, path: string) => parse(abi().og_load_ui_material(world.id, path)),
  document: (world: WorldImpl, path: string) => parse(abi().og_load_document(world.id, path)),
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
  vfx: (world: WorldImpl, path: string) => parse(abi().og_load_vfx(world.id, path)),
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
  setGroupScale: (world: WorldImpl, group: number, scale: number) => parse(abi().og_time_set_group_scale(world.id, group, scale)),
  groupDt: (world: WorldImpl, entity: number) => abi().og_time_group_dt(world.id, entity),
});

export const camera = Object.freeze({
  shake: (world: WorldImpl, trauma: number) => parse(abi().og_camera_shake(world.id, trauma)),
  punch: (world: WorldImpl, amount: number) => parse(abi().og_camera_punch(world.id, amount)),
  kick: (world: WorldImpl, direction: unknown, distance: number) =>
    parse(abi().og_camera_kick(world.id, JSON.stringify(direction), distance)),
});

export const tween = Object.freeze({
  to: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_to(world.id, json(desc))),
  from: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_from(world.id, json(desc))),
  fromTo: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_from_to(world.id, json(desc))),
  sequence: (world: WorldImpl, steps: unknown, desc?: unknown) =>
    parse(abi().og_tween_sequence(world.id, JSON.stringify(steps), json(desc))),
  parallel: (world: WorldImpl, tweens: unknown, desc?: unknown) =>
    parse(abi().og_tween_parallel(world.id, JSON.stringify(tweens), json(desc))),
  cancel: (world: WorldImpl, t: number) => abi().og_tween_cancel(world.id, t),
  cancelEntity: (world: WorldImpl, entity: number) => abi().og_tween_cancel_entity(world.id, entity),
  active: (world: WorldImpl, t: number) => abi().og_tween_active(world.id, t),
  squash: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_squash(world.id, json(desc))),
  flash: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_flash(world.id, json(desc))),
  timeline: (world: WorldImpl, desc?: unknown) => tweenTimeline(abi().og_tween_timeline(world.id, json(desc))),
});

/** A TweenTimeline: its builders answer the timeline itself, so calls chain as GSAP's. */
function tweenTimeline(h: number): object {
  const a = abi;
  const at = (position: unknown) => json(position);
  const tl: Record<string, unknown> = {};
  const chain = (_: number) => tl;
  Object.defineProperties(tl, {
    tween: { enumerable: true, get: () => a().og_tween_timeline_get_tween(h) },
    error: { enumerable: true, get: () => parse<string | null>(a().og_tween_timeline_get_error(h)) },
  });
  Object.assign(tl, {
    to: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_to(h, JSON.stringify(targets), json(vars), at(position))),
    from: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_from(h, JSON.stringify(targets), json(vars), at(position))),
    fromTo: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_from_to(h, JSON.stringify(targets), json(vars), at(position))),
    set: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_set(h, JSON.stringify(targets), json(vars), at(position))),
    call: (name: string, position?: unknown) => chain(a().og_tween_timeline_call(h, name, at(position))),
    addLabel: (label: string, position?: unknown) => chain(a().og_tween_timeline_add_label(h, label, at(position))),
    // A timeline object nests by its handle, as GSAP's tl.add(child) does.
    add: (child: unknown, position?: unknown) => {
      const c = typeof child === "object" && child !== null && "tween" in child ? (child as { tween: number }).tween : child;
      return chain(a().og_tween_timeline_add(h, JSON.stringify(c), at(position)));
    },
    play: (from?: unknown) => chain(a().og_tween_timeline_play(h, at(from))),
    pause: (atTime?: unknown) => chain(a().og_tween_timeline_pause(h, at(atTime))),
    resume: () => chain(a().og_tween_timeline_resume(h)),
    reverse: (from?: unknown) => chain(a().og_tween_timeline_reverse(h, at(from))),
    restart: () => chain(a().og_tween_timeline_restart(h)),
    seek: (position: unknown) => chain(a().og_tween_timeline_seek(h, JSON.stringify(position))),
    progress: (value?: number) => a().og_tween_timeline_progress(h, value),
    time: () => a().og_tween_timeline_time(h),
    duration: () => a().og_tween_timeline_duration(h),
    timeScale: (value?: number) => a().og_tween_timeline_time_scale(h, value),
    isActive: () => a().og_tween_timeline_is_active(h),
    kill: () => a().og_tween_timeline_kill(h),
  });
  return Object.freeze(tl);
}

export const tilemap = Object.freeze({
  setTile: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_set_tile(world.id, json(desc))),
  fill: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_fill(world.id, json(desc))),
  create: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_create(world.id, json(desc))),
  resize: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_resize(world.id, json(desc))),
  neighbours: (world: WorldImpl, map: number, x: number, y: number) =>
    parse(abi().og_tilemap_neighbours(world.id, map, x, y)),
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
  castRay: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_cast_ray(world.id, json(desc))),
  castRayAll: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_cast_ray_all(world.id, json(desc))),
  projectPoint: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_project_point(world.id, json(desc))),
  castShape: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_cast_shape(world.id, json(desc))),
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
  garmentPoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_garment_points(world.id, entity)),
  fluidParticles: (world: WorldImpl, entity: number) => parse(abi().og_physics_fluid_particles(world.id, entity)),
  fluid3dParticles: (world: WorldImpl, entity: number) => parse(abi().og_physics_fluid3d_particles(world.id, entity)),
  waterHeight: (world: WorldImpl, entity: number, x: number, z: number) =>
    parse(abi().og_physics_water_height(world.id, entity, x, z)),
  vehicleWheels: (world: WorldImpl, entity: number) => parse(abi().og_physics_vehicle_wheels(world.id, entity)),
  softBodyPoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_soft_body_points(world.id, entity)),
  ragdollRigFromSkeleton: (world: WorldImpl, skeleton: number) =>
    parse(abi().og_physics_ragdoll_rig_from_skeleton(world.id, skeleton)),
  setRagdollRig: (world: WorldImpl, entity: number, desc?: unknown) =>
    parse(abi().og_physics_set_ragdoll_rig(world.id, entity, json(desc))),
  ragdollPose: (world: WorldImpl, entity: number) => parse(abi().og_physics_ragdoll_pose(world.id, entity)),
  stats: (world: WorldImpl) => parse(abi().og_physics_stats(world.id)),
});

export const audio = Object.freeze({
  outputLatency: (world: WorldImpl) => (checkAccess("read", "og.Audio"), abi().og_audio_output_latency(world.id)),
  playFeedback: (world: WorldImpl, sound: number, desc?: unknown) =>
    parse(abi().og_audio_play_feedback(world.id, sound, json(desc))),
  createSound: (world: WorldImpl, desc?: unknown) => {
    const d = desc !== null && typeof desc === "object" ? { ...(desc as Record<string, unknown>) } : desc;
    if (d !== null && typeof d === "object" && ArrayBuffer.isView((d as Record<string, unknown>).samples)) {
      const r = d as Record<string, unknown>;
      r.samples = Array.from(r.samples as ArrayLike<number>);
    }
    return parse(abi().og_audio_create_sound(world.id, json(d)));
  },
});

export const atmosphere = Object.freeze({
  timeOfDay: (world: WorldImpl) => abi().og_atmosphere_time_of_day(world.id),
});

export const ui = Object.freeze({
  scope: (world: WorldImpl, id: string): Api.UiScope => uiReactor(world).scope(id),
  configure: (world: WorldImpl, desc?: unknown) => parse(abi().og_ui_configure(world.id, json(desc))),
  filePicker: (world: WorldImpl): boolean => abi().og_ui_file_picker(world.id),
  viewport: (world: WorldImpl) => parse(abi().og_ui_viewport(world.id)),
  theme: (world: WorldImpl) => parse(abi().og_ui_theme(world.id)),
  frameDirection: (): "ltr" | "rtl" => abi().og_ui_frame_direction() as "ltr" | "rtl",
  frameCursor: (): Api.Cursor => abi().og_ui_frame_cursor() as Api.Cursor,
  scrollIntoView: (path: string, center: boolean): void => abi().og_ui_scroll_into_view(path, center),
  feedback: (event: string, rise: number) => abi().og_ui_feedback(event, rise),
  setHaptics: (intensity: number) => abi().og_ui_set_haptics(intensity),
});
