import { spawnSite } from "../spawn-site.ts";
import { UiReactor } from "../ui/reactive.ts";
import { componentRecord, decodeComponent, encodeComponent, forgetComponents } from "./components.ts";
import { type Token, engineGames, engineRoots, gameMigrations } from "./declare.ts";
import { message } from "./engine.ts";
import { abi } from "./host.ts";
import { plain } from "./record-bytes.ts";
import { jsonResource, recordHandle, recordOf, refresh } from "./resources.ts";
import { type Result, json, parse } from "./result.ts";
import { rt } from "./state.ts";
import { QueryImpl, checkAccess } from "./views.ts";

// ── the world ─────────────────────────────────────────────────────────────

/** A structural change outside systems: every view handed out so far goes stale. */
function structural<T>(answer: T): T {
  if (!rt.running) rt.epoch++;
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
    if (rt.publishBuild !== true && (desc == null || (typeof desc === "object" && !(desc as { source?: unknown }).source))) {
      const source = spawnSite(engineRoots.get(this.id) ?? rt.creatingRoot);
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
    const c = typeof entity === "number" ? componentRecord(this.id, component.name) : null;
    if (c) {
      const engine = abi();
      // The answer overwrites what a system's query found in the shared buffer.
      rt.sharedStamp++;
      const epoch = engine.og_engine_read_component(this.id, entity, c.id);
      if (epoch === c.epoch) {
        const value = decodeComponent(c);
        if (value !== undefined) return { ok: true, value };
      } else if (epoch !== 0) {
        forgetComponents();
      }
    }
    // Refused, or a value only JSON reads: the engine says which, as it always has.
    return parse(abi().og_world_get(this.id, entity, component.name));
  }

  set(entity: number, component: Token, value: unknown): Result<void> {
    // GameDesc.ui's refusal of a write names og_world_set.
    const c = typeof entity === "number" && !rt.uiRegistrationErrors && plain(value) ? componentRecord(this.id, component.name) : null;
    if (c) {
      const engine = abi();
      rt.sharedStamp++;
      if (encodeComponent(c, value as Record<string, unknown>)) {
        const epoch = engine.og_engine_write_component(this.id, entity, c.id, c.epoch);
        if (epoch === c.epoch) return structural({ ok: true, value: undefined });
        if (epoch !== 0) forgetComponents();
      }
    }
    return structural(parse(abi().og_world_set(this.id, entity, component.name, JSON.stringify(value ?? {}))));
  }

  resource(resource: Token): Result<object> {
    if (resource.name !== "og.Time") checkAccess("read", resource.name);
    // A resource with a byte layout: its handle, made once per world. The
    // read here is what refuses an unregistered or undeclared resource.
    const r = rt.host ? recordOf(this.id, resource.name) : null;
    if (r && (r.dirty || r.at === rt.engineCalls || refresh(r))) return { ok: true, value: recordHandle(this, r) };
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
    return { ok: true, value: new QueryImpl(answer.value, null, this.id, -1) };
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

export const worlds = new Map<number, WorldImpl>();
export const uiReactors = new Map<number, UiReactor>();

export function uiReactor(world: WorldImpl): UiReactor {
  let reactor = uiReactors.get(world.id);
  if (!reactor) {
    reactor = new UiReactor(bytes => abi().og_engine_ui_patch(world.id, bytes));
    uiReactors.set(world.id, reactor);
  }
  return reactor;
}

export function worldFor(id: number): WorldImpl {
  let w = worlds.get(id);
  if (!w) worlds.set(id, (w = new WorldImpl(id)));
  return w;
}

export const rates = new Map<number, number>();
