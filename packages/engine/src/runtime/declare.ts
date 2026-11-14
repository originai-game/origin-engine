import bindings from "../../types/bindings.json" with { type: "json" };
import { declare } from "./host.ts";
import { type Result, json, parse } from "./result.ts";
import { rt } from "./state.ts";
import { SaveEditImpl } from "./world.ts";

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
export const callbacks: Function[] = [];

function callback(fn: Function): number {
  callbacks.push(fn);
  return callbacks.length - 1;
}

export interface SystemToken {
  readonly name: string;
  readonly reads: readonly Token[];
  readonly writes: readonly Token[];
}

export interface SystemEntry {
  readonly name: string;
  readonly reads: ReadonlySet<string>;
  readonly writes: ReadonlySet<string>;
}

/** Systems by callback id: the dispatcher refuses any other id. */
export const systems = new Map<number, SystemEntry>();
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
export const conditions = new Map<number, SystemEntry>();
/** Hooks by callback id. */
export const hookEntries = new Map<number, SystemEntry>();
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

export const gameHandles = new WeakMap<object, number>();
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
  readonly maxCatchUpTicks?: number;
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
export const gameMigrations = new WeakMap<object, readonly SaveMigrationIn[]>();
/** The game an engine runs, for its world's restores. */
export const engineGames = new Map<number, object>();
/** Each engine's CreateOptions.root: the project a spawn's source is relative to. */
export const engineRoots = new Map<number, string>();

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
      maxCatchUpTicks: desc.maxCatchUpTicks ?? 4,
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
  if (!rt.host) {
    declare((a) => void a.og_commands_register(name, id, json(desc)));
    return { ok: true, value: undefined };
  }
  return parse(rt.host.abi.og_commands_register(name, id, json(desc)));
}
