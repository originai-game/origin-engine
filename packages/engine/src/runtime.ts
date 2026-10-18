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
import { SYMBOLS } from "./generated/abi.ts";
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
  host = binding;
  binding.abi.og_host_set_dispatcher(dispatch);
  for (const declare of queue.splice(0)) declare(binding.abi);
}

/** The bound host, for loaders. */
export function boundHost(): HostBinding | null {
  return host;
}

function abi(): HostAbi<Uint32Array, unknown> {
  if (!host) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
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

function builtinFields(fields: ReadonlyArray<{ name: string; column?: string; type?: string; default?: unknown }>): Record<string, FieldSpec> {
  const out: Record<string, FieldSpec> = {};
  for (const f of fields) out[f.name] = Object.freeze({ kind: f.column ?? f.type ?? "", default: f.default, options: Object.freeze({}) });
  return out;
}

type Decl = { name: string; schema: string; fields: Array<{ name: string; column?: string; type?: string; default?: unknown }> };

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
  readonly maxCatchUp?: number;
  readonly facts?: Function | null;
  readonly start?: Function | null;
  readonly render?: Readonly<Record<string, unknown>>;
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
    const wire = JSON.stringify({
      tick: desc.tick ?? 60,
      seed: desc.seed ?? 0,
      scenes: desc.scenes ?? {},
      scene: desc.scene ?? "main",
      systems: list.map((s) => systemHandles.get(s) ?? -1),
      resources: (desc.resources ?? []).map((r) => r.name),
      commands,
      maxCatchUp: desc.maxCatchUp ?? 4,
      facts,
      start,
      render: desc.render ?? {},
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
      const kind = COLUMNS.get(component.fields![n]!.kind);
      if (!kind) throw new Error(`'${component.name}.${n}' is not a column field`);
      return kind;
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

// ── resources as live values ──────────────────────────────────────────────

function liveResource(world: WorldImpl, token: Token, value: Record<string, unknown>): object {
  const id = world.id;
  if (token.name === "og.Rng") {
    return Object.freeze({
      get seed() { return parse<{ ok: boolean; value?: { seed: number } }>(abi().og_world_resource(id, "og.Rng")).value?.seed ?? 0; },
      get state() { return parse<{ ok: boolean; value?: { state: number } }>(abi().og_world_resource(id, "og.Rng")).value?.state ?? 0; },
      next: () => abi().og_rng_next(id),
      range: (low: number, high: number) => abi().og_rng_range(id, low, high),
      integer: (low: number, high: number) => abi().og_rng_integer(id, low, high),
      chance: (p: number) => abi().og_rng_chance(id, p),
      fork: (label: string) => rngStream(abi().og_rng_fork(id, label)),
    });
  }
  if (token.name === "og.Time") return Object.freeze({ ...value });
  const out: Record<string, unknown> = {};
  const readonly = new Set(
    ((bindings.resources as Array<{ schema: string; fields: Array<{ name: string; readonly?: boolean }> }>).find((r) => r.schema === token.name)?.fields ?? [])
      .filter((f) => f.readonly)
      .map((f) => f.name),
  );
  for (const key of Object.keys(value)) {
    const setter = (v: unknown) => {
      const answer = parse<Result<void>>(abi().og_engine_write_resource(id, token.name, JSON.stringify({ [key]: v })));
      if (!answer.ok) throw new Error(`${token.name}.${key}: ${answer.reason}`);
      value[key] = v;
    };
    Object.defineProperty(
      out,
      key,
      readonly.has(key) ? { enumerable: true, get: () => value[key] } : { enumerable: true, get: () => value[key], set: setter },
    );
  }
  if (token.name === "og.Input") {
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
  if (token.name === "og.Pointers") {
    Object.assign(out, {
      down: (pointer: number) => abi().og_pointers_down(id, pointer),
      pressed: (pointer: number) => abi().og_pointers_pressed(id, pointer),
      released: (pointer: number) => abi().og_pointers_released(id, pointer),
      position: (pointer: number) => parse(abi().og_pointers_position(id, pointer)),
      start: (pointer: number) => parse(abi().og_pointers_start(id, pointer)),
      count: () => abi().og_pointers_count(id),
    });
  }
  if (token.name === "og.Gamepad") {
    Object.assign(out, {
      connected: (pad?: number) => abi().og_gamepad_connected(id, pad),
      down: (button: string, pad?: number) => abi().og_gamepad_down(id, button, pad),
      pressed: (button: string, pad?: number) => abi().og_gamepad_pressed(id, button, pad),
      released: (button: string, pad?: number) => abi().og_gamepad_released(id, button, pad),
      axis: (axis: string, pad?: number) => abi().og_gamepad_axis(id, axis, pad),
    });
  }
  if (token.name === "og.Camera") {
    Object.assign(out, {
      lookAt: (target: unknown) => abi().og_camera_look_at(id, JSON.stringify(target)),
      worldToScreen: (point: unknown) => parse(abi().og_camera_world_to_screen(id, JSON.stringify(point))),
      screenToRay: (x: number, y: number) => parse(abi().og_camera_screen_to_ray(id, x, y)),
    });
  }
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
    const answer = parse<Result<Record<string, unknown>>>(abi().og_world_resource(this.id, resource.name));
    if (!answer.ok) return answer;
    return { ok: true, value: liveResource(this, resource, answer.value) };
  }

  find(name: string): number | null {
    return parse(abi().og_world_find(this.id, name));
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
    return parse(abi().og_world_events(this.id, event.name));
  }

  emit(event: Token, payload: unknown): Result<void> {
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
      return "";
    }
    if (!fn) return `the dispatcher has no callback with id ${id}`;
    if (kind === 1) {
      fn(world, Object.freeze(JSON.parse(payload || "[]")));
      return "";
    }
    if (kind === 2) return "=" + JSON.stringify(fn(world) ?? null);
    return `unknown dispatch kind ${kind}`;
  } catch (e) {
    return message(e);
  } finally {
    running = outer;
    epoch++;
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
  /** Run GameDesc.start at the first tick boundary instead of now, so a host can attach its renderer and report the viewport first (the web Worker does). */
  readonly deferStart?: boolean;
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

/** A running engine: one world, its probe, and the host's lifecycle calls. */
export class Engine {
  readonly id: number;
  readonly game: GameToken;
  /** The engine's world, as game code sees it. */
  readonly world: Api.World;
  /** The engine's probe: RuntimeProbe protocol 5 and the wire members. */
  readonly probe: Api.EngineProbe;

  #platform: PlatformHost = noPlatform;

  constructor(id: number, game: GameToken) {
    this.id = id;
    this.game = game;
    this.world = worldFor(id) as unknown as Api.World;
    this.probe = makeProbe(id) as unknown as Api.EngineProbe;
    rates.set(id, game.tick);
    this.attachPlatform(noPlatform);
  }

  /** Attach what answers og.platform's calls: the native host's local store, the web shell's Player bridge. */
  attachPlatform(host: PlatformHost): void {
    this.#platform = host;
    abi().og_platform_set_host(this.id, JSON.stringify({ host: host.host, capabilities: host.capabilities }));
  }

  /** Hand the platform calls made since the last pump to the attached host (before and after every advance and frame, after a command). */
  pumpPlatform(): void {
    const calls = parse<PlatformCall[]>(abi().og_platform_take(this.id));
    if (calls.length > 0) this.#platform.handle(calls, (request, answer) => void this.answerPlatform(request, answer));
  }

  /** Answer one platform call; applied at the start of the next tick. */
  answerPlatform(request: number, answer: PlatformAnswer): boolean {
    return abi().og_platform_answer(this.id, request, JSON.stringify(answer));
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
  frame(elapsedMs: number): { ok: boolean; ticks?: number; tick?: number; rendered?: boolean; paused?: boolean; quit?: boolean; reason?: string } {
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
    return abi().og_engine_destroy(this.id);
  }
}

/** Assemble an engine from a declared game (the bundle's default export). */
export function createEngine(game: GameToken, options: CreateOptions = {}): Result<Engine> {
  const handle = gameHandles.get(game);
  if (handle === undefined) return refused("not a game: pass the bundle's default export, the result of og.game()");
  const answer = parse<Result<number>>(abi().og_engine_create(handle, JSON.stringify(options)));
  if (!answer.ok) return answer;
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
  text: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_text(world.id, json(desc))),
  sprite: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_sprite(world.id, json(desc))),
  ribbon: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_ribbon(world.id, json(desc))),
  measureText: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_measure_text(world.id, json(desc))),
  floatingText: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_floating_text(world.id, json(desc))),
  boxGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_box_geometry(world.id, json(desc))),
  planeGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_plane_geometry(world.id, json(desc))),
  sphereGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_sphere_geometry(world.id, json(desc))),
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
  lut: (world: WorldImpl, path: string) => parse(abi().og_load_lut(world.id, path)),
  emitter: (world: WorldImpl, path: string) => parse(abi().og_load_emitter(world.id, path)),
  skin: (world: WorldImpl, path: string) => parse(abi().og_load_skin(world.id, path)),
  soundInfo: (world: WorldImpl, sound: number) => parse(abi().og_load_sound_info(world.id, sound)),
  status: (world: WorldImpl, asset: number) => abi().og_load_status(world.id, asset),
  error: (world: WorldImpl, asset: number) => parse(abi().og_load_error(world.id, asset)),
  gltfInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_gltf_info(world.id, asset)),
  animation: (world: WorldImpl, path: string) => parse(abi().og_load_animation(world.id, path)),
  flipbook: (world: WorldImpl, path: string) => parse(abi().og_load_flipbook(world.id, path)),
  tilemap: (world: WorldImpl, path: string) => parse(abi().og_load_tilemap(world.id, path)),
});

export const anim = Object.freeze({
  graph: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_graph(world.id, json(desc))),
  clip: (info: { clips?: ReadonlyArray<{ clip: number; name: string }> }, name: string): number =>
    info?.clips?.find((c) => c.name === name)?.clip ?? -1,
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
  setMesh: (world: WorldImpl, mesh: number, desc?: unknown) => parse(abi().og_physics_set_mesh(world.id, mesh, json(desc))),
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
  button: (id: string, text: string, desc?: unknown) => parse(abi().og_ui_button(id, text, json(desc))),
  image: (id: string, desc?: unknown) => abi().og_ui_image(id, json(desc)),
  progress: (id: string, value: number, desc?: unknown) => abi().og_ui_progress(id, value, json(desc)),
  slider: (id: string, value: number, desc?: unknown) => parse(abi().og_ui_slider(id, value, json(desc))),
  toggle: (id: string, on: boolean, label: string) => parse(abi().og_ui_toggle(id, on, label)),
  textField: (id: string, value: string, desc?: unknown) => parse(abi().og_ui_text_field(id, value, json(desc))),
  tooltip: (text: string) => abi().og_ui_tooltip(text),
  idi: (name: string, index: number) => `${name}#${index}`,
});
