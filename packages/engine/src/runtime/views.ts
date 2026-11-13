import { componentId, forgetComponents } from "./components.ts";
import type { SystemEntry, Token } from "./declare.ts";
import { COLUMNS, type ColumnArray, type ColumnSpec, FRAME_QUERY_WORD, abi, words } from "./host.ts";
import { parse } from "./result.ts";
import { rt } from "./state.ts";

// ── views ─────────────────────────────────────────────────────────────────


export interface Running {
  readonly system: SystemEntry;
  readonly tick: number;
}



/** The engine, its world and the tick game code is running in now: og.assert records into it. */
export function currentCall(): { readonly engine: number; readonly tick: number; readonly world: object } | null {
  return rt.calling;
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
  /** The view's handle. */
  readonly #view: number;
  readonly #tick: number;
  /** The archetype's components, as their numbers in its engine (componentId). */
  readonly #types: readonly number[];
  readonly #engine: number;
  readonly #epoch: number;
  readonly #system: SystemEntry | null;
  #entities: Uint32Array | null = null;
  #columns: Map<string, object> | null = null;

  constructor(view: number, count: number, tick: number, types: readonly number[], engine: number, system: SystemEntry | null) {
    this.count = count;
    this.#view = view;
    this.#tick = tick;
    this.#types = types;
    this.#engine = engine;
    this.#epoch = rt.epoch;
    this.#system = system;
  }

  #check(): void {
    if (this.#epoch !== rt.epoch) {
      const owner = this.#system ? `system '${this.#system.name}'` : "a call outside systems";
      throw new Error(`stale ArchetypeView: it belonged to tick ${this.#tick}, ${owner}; views last for the system call they were handed to (outside systems, until the next structural change), so take them again from q.archetypes()`);
    }
  }

  /** The entity count: plain data (a hot loop's bound), so reading it never throws. */
  readonly count: number;

  get tick(): number {
    this.#check();
    return this.#tick;
  }

  get entities(): Uint32Array {
    this.#check();
    return (this.#entities ??= abi().og_archetype_view_get_entities(this.#view));
  }

  #rows: Uint32Array | null = null;

  /** The rows the query's change filters pass (every row without filters). */
  get rows(): Uint32Array {
    this.#check();
    return (this.#rows ??= abi().og_archetype_view_get_rows(this.#view));
  }

  entity(index: number): number {
    const e = this.entities;
    return e[index * 2]! + e[index * 2 + 1]! * 4294967296;
  }

  has(component: Token): boolean {
    this.#check();
    return this.#types.includes(componentId(this.#engine, component.name));
  }

  column(component: Token): object {
    this.#check();
    const cached = this.#columns?.get(component.name) as Record<string, ColumnArray> | undefined;
    // A grown wasm memory detached the arrays: make them again.
    if (cached && !(rt.host!.fresh && Object.values(cached).some((a) => !rt.host!.fresh!(a)))) return cached;
    const s = this.#system;
    if (s && !s.reads.has(component.name) && !s.writes.has(component.name)) {
      throw new Error(`system '${s.name}' did not declare '${component.name}' in its reads or writes; declare it to get its columns`);
    }
    const id = componentId(this.#engine, component.name);
    if (!this.#types.includes(id)) {
      throw new Error(`this archetype has no '${component.name}'; check a.has(${component.name}) first`);
    }
    const { names, kinds } = layoutOf(component);
    const b = rt.host!;
    const arrays = b.columns(abi().og_engine_view_column(this.#view, id), kinds, this.count);
    if (!arrays || arrays.length !== names.length) {
      throw new Error(`the ${b.name} host refused the columns of '${component.name}' (tick ${this.#tick})`);
    }
    const view = viewOf(component, names, arrays);
    (this.#columns ??= new Map()).set(component.name, view);
    return view;
  }
}

export class QueryImpl {
  readonly #handle: number;
  readonly #system: SystemEntry | null;
  readonly #engine: number;
  /** `sharedStamp` while the shared buffer holds this query's archetypes as its system call wrote them; -1 for a world query. */
  readonly #described: number;
  #cached: ArchetypeViewImpl[] | null = null;
  #cachedEpoch = -1;

  constructor(handle: number, system: SystemEntry | null, engine: number, described: number) {
    this.#handle = handle;
    this.#system = system;
    this.#engine = engine;
    this.#described = described;
  }

  /**
   * The archetypes as the engine describes them in the shared buffer
   * (`og_engine_query_archetypes`): a system's are there when it is called,
   * so reading them crosses to the engine only when something wrote the
   * buffer since, or when they did not all fit.
   */
  archetypes(): readonly ArchetypeViewImpl[] {
    if (this.#cached && this.#cachedEpoch === rt.epoch) return this.#cached;
    const list: ArchetypeViewImpl[] = [];
    let at = FRAME_QUERY_WORD;
    if (this.#described !== rt.sharedStamp) {
      rt.sharedStamp++;
      abi().og_engine_query_archetypes(this.#handle, 0);
      at = 0;
    }
    for (;;) {
      const w = words();
      const total = w[at]!;
      const written = w[at + 1]!;
      const tick = w[at + 2]!;
      if (w[at + 3] !== rt.componentIdsEpoch) {
        // Layouts were declared again (a new engine, a reload): the numbers may have moved.
        forgetComponents();
        rt.componentIdsEpoch = w[at + 3]!;
      }
      let p = at + 4;
      for (let k = 0; k < written; k++) {
        const n = w[p + 2]!;
        const types = new Array<number>(n);
        for (let i = 0; i < n; i++) types[i] = w[p + 3 + i]!;
        list.push(new ArchetypeViewImpl(w[p]!, w[p + 1]!, tick, types, this.#engine, this.#system));
        p += 3 + n;
      }
      if (written === 0 || list.length >= total) break;
      // The rest did not fit: asked for from where this stopped.
      rt.sharedStamp++;
      abi().og_engine_query_archetypes(this.#handle, list.length);
      at = 0;
    }
    this.#cached = list;
    this.#cachedEpoch = rt.epoch;
    return list;
  }

  count(): number {
    return abi().og_query_count(this.#handle);
  }
}

// ── declared access ───────────────────────────────────────────────────────


/** Whether the bound host is a publish build, once an engine was made (null before). */
export function isPublishBuild(): boolean | null {
  return rt.publishBuild;
}

export function learnBuild(engine: number): void {
  rt.publishBuild ??= (parse<{ build?: string } | null>(abi().og_engine_probe_hello(engine))?.build ?? "dev") === "publish";
}

/**
 * Inside a system, a resource or event it did not declare (`read`: among its
 * reads or writes; `write`: among its writes). Outside a publish build that
 * throws with the system's name and the record, failing the tick; a publish
 * build lets the engine refuse it, which logs it once per system.
 */
export function checkAccess(access: "read" | "write", name: string): boolean {
  if (access === "write" && rt.uiRegistrationErrors) {
    rt.uiRegistrationErrors.push(`GameDesc.ui cannot write '${name}'; gameplay setup belongs in start`);
    return false;
  }
  const r = rt.running;
  if (!r) return true;
  const s = r.system;
  if (access === "write" ? s.writes.has(name) : s.reads.has(name) || s.writes.has(name)) return true;
  if (rt.publishBuild === true) return false;
  const list = access === "write" ? "writes" : "reads or writes";
  throw new Error(`system '${s.name}' did not declare '${name}' among its ${list} (${access} refused): add it to the system's ${list}`);
}
