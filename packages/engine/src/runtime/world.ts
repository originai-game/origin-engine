import { type Custom, SaveEdit, SpatialIndex, World } from "../generated/calls.ts";
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

// ── the world: its `custom` members over the generated class ──────────────

/** A structural change outside systems: every view handed out so far goes stale. */
function structural<T>(answer: T): T {
  if (!rt.running) rt.epoch++;
  return answer;
}

/** The game's line that called `world` (spawn-site.ts), outside a publish build. */
function siteOf(world: World): string | null {
  return rt.publishBuild === true ? null : spawnSite(engineRoots.get(world.id) ?? rt.creatingRoot);
}

/** A borrow conflict's refusal with the game's line that made the call. */
function sited<T>(world: World, answer: Result<T>): Result<T> {
  if (answer.ok || !answer.reason.includes("took a column view")) return answer;
  const site = siteOf(world);
  return site === null ? answer : { ok: false, reason: `${answer.reason} (at ${site})` };
}

/** World's `custom` members (IDL `world`): the shared-buffer fast paths, access checks, views going stale, save migrations. */
export const worldCustom: Custom["World"] = {
  spawn(world, desc) {
    // SpawnDesc.source: the game's line that called spawn, outside a publish build.
    if (desc == null || (typeof desc === "object" && !(desc as { source?: unknown }).source)) {
      const source = siteOf(world);
      if (source !== null) desc = { ...(desc as object | undefined), source };
    }
    return structural(parse(abi().og_world_spawn(world.id, json(desc))));
  },
  despawn: (world, entity) => structural(parse(abi().og_world_despawn(world.id, entity))),
  insert: (world, entity, component, value) => structural(parse(abi().og_world_insert(world.id, entity, component.name, json(value)))),
  remove: (world, entity, component) => structural(parse(abi().og_world_remove(world.id, entity, component.name))),
  get(world, entity, component) {
    const c = typeof entity === "number" ? componentRecord(world.id, component.name) : null;
    if (c) {
      const engine = abi();
      // The answer overwrites what a system's query found in the shared buffer.
      rt.sharedStamp++;
      const epoch = engine.og_engine_read_component(world.id, entity, c.id);
      if (epoch === c.epoch) {
        const value = decodeComponent(c);
        if (value !== undefined) return { ok: true, value };
      } else if (epoch !== 0) {
        forgetComponents();
      }
    }
    // Refused, or a value only JSON reads: the engine says which, as it always has.
    return sited(world, parse(abi().og_world_get(world.id, entity, component.name)));
  },
  set(world, entity, component, value) {
    // GameDesc.ui's refusal of a write names og_world_set.
    const c = typeof entity === "number" && !rt.uiRegistrationErrors && plain(value) ? componentRecord(world.id, component.name) : null;
    if (c) {
      const engine = abi();
      rt.sharedStamp++;
      if (encodeComponent(c, value as Record<string, unknown>)) {
        const epoch = engine.og_engine_write_component(world.id, entity, c.id, c.epoch);
        if (epoch === c.epoch) return structural({ ok: true, value: undefined });
        if (epoch !== 0) forgetComponents();
      }
    }
    return structural(sited<void>(world, parse(abi().og_world_set(world.id, entity, component.name, JSON.stringify(value ?? {})))));
  },
  resource(world, resource) {
    if (resource.name !== "og.Time") checkAccess("read", resource.name);
    // A resource with a byte layout: its handle, made once per world. The
    // read here is what refuses an unregistered or undeclared resource.
    const r = rt.host ? recordOf(world.id, resource.name) : null;
    if (r && (r.dirty || r.at === rt.engineCalls || refresh(r))) return { ok: true, value: recordHandle(world, r) };
    const answer = parse<Result<Record<string, unknown>>>(abi().og_world_resource(world.id, resource.name));
    if (!answer.ok) return answer;
    return { ok: true, value: r ? recordHandle(world, r) : jsonResource(world, resource as Token, answer.value) };
  },
  snapshot(world, desc) {
    const d = desc as { with?: readonly Token[]; resources?: readonly Token[] } | undefined;
    const wire = d && { with: (d.with ?? []).map((t) => t.name), resources: (d.resources ?? []).map((t) => t.name) };
    return parse(abi().og_world_snapshot(world.id, json(wire)));
  },
  openSave(world, save) {
    const answer = parse<Result<number>>(abi().og_world_open_save(world.id, save));
    if (!answer.ok) return answer;
    return { ok: true, value: new SaveEdit(answer.value) };
  },
  restore(world, save, desc) {
    const game = engineGames.get(world.id);
    const steps = (game && gameMigrations.get(game)) ?? [];
    if (steps.length > 0) {
      // The game's steps from the save's version on, in order (the host
      // refused a game whose steps reach past saveVersion or repeat one).
      const opened = worldCustom.openSave(world, save) as Result<SaveEdit>;
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
        const text = edit.text() as Result<string>;
        if (!text.ok) return text;
        save = text.value;
      } finally {
        edit.close();
      }
    }
    const restored = structural(parse<Result<void>>(abi().og_world_restore(world.id, save, json(desc))));
    if (restored.ok) {
      const mounts = uiReactors.get(world.id)?.restoreMounts();
      if (mounts && !mounts.ok) return mounts;
    }
    return restored;
  },
  removed(world, component) {
    checkAccess("read", component.name);
    return parse(abi().og_world_removed(world.id, component.name));
  },
  spatial(world, desc) {
    const d = desc as { with?: readonly Token[]; without?: readonly Token[]; cell?: number } | undefined;
    const wire = d && { with: (d.with ?? []).map((t) => t.name), without: (d.without ?? []).map((t) => t.name), cell: d.cell ?? 0 };
    const answer = parse<Result<number>>(abi().og_world_spatial(world.id, json(wire)));
    if (!answer.ok) return answer;
    return { ok: true, value: new SpatialIndex(answer.value) };
  },
  query(world, desc) {
    const d = desc as { with?: readonly Token[]; without?: readonly Token[]; changed?: readonly Token[]; added?: readonly Token[]; removed?: readonly Token[] } | undefined;
    const names = (l?: readonly Token[]) => (l ?? []).map((t) => t.name);
    const wire = d && { with: names(d.with), without: names(d.without), changed: names(d.changed), added: names(d.added), removed: names(d.removed) };
    const answer = parse<Result<number>>(abi().og_world_query(world.id, json(wire)));
    if (!answer.ok) return answer;
    return { ok: true, value: new QueryImpl(answer.value, null, world.id, -1) };
  },
  events(world, event) {
    checkAccess("read", event.name);
    return parse(abi().og_world_events(world.id, event.name));
  },
  emit(world, event, payload) {
    checkAccess("write", event.name);
    return parse(abi().og_world_emit(world.id, event.name, JSON.stringify(payload ?? {})));
  },
  instantiate: (world, asset, desc) => structural(parse(abi().og_world_instantiate(world.id, asset, json(desc)))),
};

type Records = readonly { entity: number | null; value: Record<string, unknown> }[];

/** SaveEdit's members the runtime implements over its others (IDL `host false`). */
export const saveEditCustom: Custom["SaveEdit"] = {
  has(edit, name) {
    const r = edit.types() as Result<readonly { name: string }[]>;
    return r.ok && r.value.some((t) => t.name === name);
  },
  transform(edit, name, map, desc) {
    const old = edit.records(name) as Result<Records>;
    if (!old.ok) return old;
    const f = map as (value: Record<string, unknown>, entity: number | null) => unknown;
    let records;
    try {
      records = old.value.map((r) => ({ entity: r.entity, value: f(r.value, r.entity) ?? {} }));
    } catch (e) {
      return { ok: false, reason: `SaveEdit.transform of ${name}: ${message(e)}` };
    }
    return edit.setRecords(name, { into: (desc as { into?: string } | undefined)?.into ?? "", records });
  },
  split(edit, name, map) {
    const old = edit.records(name) as Result<Records>;
    if (!old.ok) return old;
    const f = map as (value: Record<string, unknown>, entity: number | null) => Record<string, unknown> | null;
    const parts = new Map<string, { entity: number | null; value: unknown }[]>();
    try {
      for (const r of old.value) {
        for (const [type, value] of Object.entries(f(r.value, r.entity) ?? {})) {
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
    if (names.length === 0) return edit.dropType(name);
    const [first, ...rest] = names;
    for (const type of rest) {
      if (saveEditCustom.has(edit, type)) return { ok: false, reason: `SaveEdit.split of ${name}: the save already holds ${type}` };
    }
    const r = edit.setRecords(name, { into: first, records: parts.get(first!)! }) as Result<void>;
    if (!r.ok) return r;
    // A type the save does not hold: setRecords adds it.
    for (const type of rest) {
      const added = edit.setRecords(type, { records: parts.get(type)! }) as Result<void>;
      if (!added.ok) return added;
    }
    return { ok: true, value: undefined };
  },
};

export const worlds = new Map<number, World>();
export const uiReactors = new Map<number, UiReactor>();

export function uiReactor(world: { readonly id: number }): UiReactor {
  let reactor = uiReactors.get(world.id);
  if (!reactor) {
    reactor = new UiReactor(bytes => abi().og_engine_ui_patch(world.id, bytes));
    uiReactors.set(world.id, reactor);
  }
  return reactor;
}

export function worldFor(id: number): World {
  let w = worlds.get(id);
  if (!w) worlds.set(id, (w = new World(id)));
  return w;
}

const rates = new Map<number, number>();

/** An engine's tick rate, og.Time's `rate` (GameDesc.tick, else project.ogt's, else 60), read once: the one source every time here counts from. */
export function rateOf(engine: number): number {
  let rate = rates.get(engine);
  if (rate === undefined) {
    const time = parse<Result<{ rate: number }>>(abi().og_world_resource(engine, "og.Time"));
    if (!time.ok) throw new Error(`engine ${engine} has no clock: ${time.reason}`);
    rates.set(engine, (rate = time.value.rate));
  }
  return rate;
}
