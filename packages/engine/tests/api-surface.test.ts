// Every member the IDL declares exists on the runtime object a game reaches
// it through. types/bindings.json (generated from crates/og-api/api.ogt with
// the .d.ts) lists each call with its path; the declarations alone cannot
// catch a missing member, because src/index.ts assembles `og` from plain
// objects cast to the declared type. og.load.textureArray was declared,
// typed, documented and bound on both hosts, and still `not a function`
// (examples/lantern-vale/FINDINGS.md #9); EngineProbe.listen likewise.
//
// And the other way round: every member a game can reach (the package's
// named exports, `og` and its namespaces, the World and its queries and
// views, the systems and the game it declares, the resources' objects, the
// probe) is declared in the IDL, whose catalog (schema/api.schema.json) the
// wiki is generated from. A member the IDL does not declare is undocumented
// for an author; the few that stay so are listed in UNDECLARED with why, so
// one is a decision, never an accident.
import { beforeAll, expect, test } from "bun:test";
import bindings from "../types/bindings.json" with { type: "json" };
import schema from "../schema/api.schema.json" with { type: "json" };
import * as engineExports from "../src/index.ts";
import { og, defineComponent, defineSystem, f32 } from "../src/index.ts";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

interface Call {
  readonly kind: "fn" | "method" | "prop";
  readonly path: string;
  readonly symbol: string;
}

const Marker = defineComponent("SurfaceMarker", { value: f32() });
// One field of every column kind, each made by its exported factory: the engine takes them all.
const factories = engineExports as unknown as Record<string, (...a: unknown[]) => unknown>;
const Every = defineComponent(
  "SurfaceEvery",
  Object.fromEntries(
    (bindings.columns as Array<{ name: string }>)
      .filter((c) => typeof factories[c.name] === "function")
      .map((c) => [
        `f${c.name}`,
        c.name === "str" ? factories[c.name]!(8) : c.name === "list" ? factories[c.name]!(factories.u32!(), 4) : factories[c.name]!(),
      ]),
  ) as never,
);
// A list of handles of one of the kinds rumpus-isle #2 lacked, and one beside it.
const Fonts = defineComponent("SurfaceFonts", { fonts: engineExports.list(engineExports.Font(), 2), style: engineExports.Style() });
const Noop = defineSystem("SurfaceNoop", { reads: [Marker] }, () => {});
const game = og.game({ systems: [Noop] }) as unknown as GameToken;

let engine: Engine;

beforeAll(() => {
  loadNativeHost();
  const created = createEngine(game);
  if (!created.ok) throw new Error(created.reason);
  engine = created.value;
  const e = engine.world.spawn({ name: "marked" });
  if (!e.ok) throw new Error(e.reason);
  engine.world.insert(e.value, Marker as never);
  const every = engine.world.insert(e.value, Every as never);
  if (!every.ok) throw new Error(`a field of every column kind: ${every.reason}`);
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/**
 * The runtime object that carries each declared type's members, as a game
 * reaches it. A type the IDL gains without an entry here fails the test
 * (below), so a new owner is never skipped silently.
 */
function owners(): Record<string, object> {
  const world = engine.world;
  const query = must(world.query({ with: [Marker] }));
  const view = query.archetypes()[0];
  if (!view) throw new Error("the query found no archetype");
  const rng = must(world.resource(og.Rng));
  return {
    World: world,
    Query: query,
    ArchetypeView: view,
    System: Noop,
    Game: game,
    Rng: rng,
    RngStream: rng.fork("surface"),
    Input: must(world.resource(og.Input)),
    Pointers: must(world.resource(og.Pointers)),
    Gamepad: must(world.resource(og.Gamepad)),
    Camera: must(world.resource(og.Camera)),
    RuntimeProbe: engine.probe,
    EngineProbe: engine.probe,
    // og.Path2 and og.Shape instances; Shape.holes is a plain array (PathList
    // declares the two members a game uses, push and length).
    Path2: new og.Path2(),
    Shape: new og.Shape(),
  };
}

/** Where a declared path lives at runtime, or a reason it cannot be found. */
function resolve(call: Call, byType: Record<string, object>): { owner: object; member: string } | string {
  const parts = call.path.split(".");
  if (parts[0] === "og") {
    // og.<fn> or og.<module>.<fn>
    let owner: unknown = og;
    for (const p of parts.slice(1, -1)) owner = (owner as Record<string, unknown>)?.[p];
    if (typeof owner !== "object" || owner === null) return `og.${parts.slice(1, -1).join(".")} is not an object`;
    return { owner, member: parts.at(-1)! };
  }
  if (parts.length === 1) return { owner: engineExports, member: parts[0]! };
  const owner = byType[parts[0]!];
  if (!owner) return `no runtime owner for type ${parts[0]} in tests/api-surface.test.ts: add how a game reaches one`;
  return { owner, member: parts[1]! };
}

test("every function, method and property the IDL declares exists on the runtime", () => {
  const byType = owners();
  const calls = bindings.calls as Call[];
  expect(calls.length).toBeGreaterThan(200);
  const missing: string[] = [];
  for (const call of calls) {
    const where = resolve(call, byType);
    if (typeof where === "string") {
      missing.push(`${call.path}: ${where}`);
      continue;
    }
    const { owner, member } = where;
    if (!(member in owner)) missing.push(`${call.path} (${call.symbol}): missing`);
    else if (call.kind !== "prop" && typeof (owner as Record<string, unknown>)[member] !== "function")
      missing.push(`${call.path} (${call.symbol}): not a function`);
  }
  expect(missing).toEqual([]);
});

test("og.load.textureArray and probe.listen reach the host", () => {
  // No bundle is open: the host answers a refusal, not a TypeError.
  const r = og.load.textureArray(engine.world, "missing.ogt") as { ok: boolean };
  expect(r.ok).toBe(false);
  expect(() => engine.probe.listen(0.5)).not.toThrow();
});

// ── the other way round ──────────────────────────────────────────────────

/**
 * Members a game can reach that the IDL does not declare, by owner and
 * name, with why. An entry that no longer exists fails the test too, so the
 * list only ever holds what is there.
 */
const UNDECLARED: Readonly<Record<string, string>> = {
  // The engine id the runtime's free functions pass to the host
  // (`og.render.text(world, …)` sends `world.id`); a game has no use for it
  // and no call takes it, so documenting it would invite one.
  "World.id": "the runtime's own plumbing: the engine id its free functions hand the host",
};

interface CatalogFn {
  readonly name: string;
  readonly prop?: string;
}
interface CatalogInterface {
  readonly name: string;
  readonly extends?: string;
  readonly members: readonly CatalogFn[];
}
interface CatalogModule {
  readonly name: string;
  readonly place: "top" | "root" | "namespace";
  readonly functions: readonly CatalogFn[];
  readonly values?: readonly CatalogFn[];
  readonly consts: readonly CatalogFn[];
  readonly interfaces: readonly CatalogInterface[];
}

const catalog = schema as unknown as {
  readonly "x-og-modules": readonly CatalogModule[];
  readonly $defs: Readonly<Record<string, { readonly "x-og-kind"?: string; readonly properties?: Record<string, unknown> }>>;
};

/** What the IDL declares on each owner a game reaches: `og`, `og.<module>`, the package's exports, each interface and resource. */
function declared(): Map<string, Set<string>> {
  const out = new Map<string, Set<string>>();
  const add = (owner: string, names: Iterable<string>) => {
    const set = out.get(owner) ?? new Set<string>();
    for (const n of names) set.add(n);
    out.set(owner, set);
  };
  const modules = catalog["x-og-modules"];
  const interfaces = new Map(modules.flatMap((m) => m.interfaces.map((i) => [i.name, i] as const)));
  const membersOf = (i: CatalogInterface): string[] => [...i.members.map((m) => m.name), ...(i.extends ? membersOf(interfaces.get(i.extends)!) : [])];
  add("exports", ["og"]);
  add("og", []);
  for (const m of modules) {
    const fns = [...m.functions, ...(m.values ?? [])].map((f) => f.name);
    if (m.place === "top") add("exports", fns);
    else if (m.place === "root") add("og", fns);
    else if (fns.length > 0) {
      add("og", [m.name]);
      add(`og.${m.name}`, fns);
    }
    // Every const is a named export (`import { BODY_STATIC } from "@origingame/engine"`).
    add("exports", m.consts.map((c) => c.name));
    for (const i of m.interfaces) add(i.name, membersOf(i));
  }
  // Column kinds are factories among the exports; records are tokens on og.
  add("exports", (bindings.columns as Array<{ name: string }>).map((c) => c.name));
  for (const kind of ["components", "resources", "events"] as const) add("og", (bindings[kind] as Array<{ name: string }>).map((r) => r.name));
  // A resource's object: its fields and its methods.
  for (const [name, def] of Object.entries(catalog.$defs)) if (def["x-og-kind"] === "resource") add(name, Object.keys(def.properties ?? {}));
  for (const c of bindings.calls as Array<Call & { owner?: string; name: string }>) if (c.owner && c.kind !== "fn") add(c.owner, [c.name]);
  return out;
}

/** Every member a game can read on `o`: own and inherited, enumerable or not, short of Object's own. */
function reachable(o: object): string[] {
  const out = new Set<string>();
  for (let p: object | null = o; p && p !== Object.prototype && p !== Function.prototype; p = Object.getPrototypeOf(p)) {
    for (const k of Reflect.ownKeys(p)) if (typeof k === "string" && k !== "constructor") out.add(k);
  }
  return [...out];
}

test("every member a game can reach is declared in the IDL, or listed in UNDECLARED with why", () => {
  const want = declared();
  const reached: Array<[string, object]> = [
    ["exports", engineExports],
    ["og", og],
    // The probe is one object: EngineProbe, which extends RuntimeProbe.
    ...Object.entries(owners()).filter(([owner]) => owner !== "RuntimeProbe"),
  ];
  for (const [key, value] of Object.entries(og as unknown as Record<string, unknown>)) {
    // A namespace (og.render), not a record token (og.Transform, whose shape the declarations' preamble fixes).
    if (/^[a-z]/.test(key) && typeof value === "object" && value !== null) reached.push([`og.${key}`, value]);
  }
  // Every engine resource's object, as world.resource answers it.
  for (const r of bindings.resources as Array<{ name: string }>) {
    const token = (og as unknown as Record<string, object>)[r.name];
    const live = engine.world.resource(token as never);
    if (!live.ok) throw new Error(`og.${r.name}: ${live.reason}`);
    if (!reached.some(([owner]) => owner === r.name)) reached.push([r.name, live.value as object]);
  }
  const undeclared: string[] = [];
  const seen = new Set<string>();
  for (const [owner, value] of reached) {
    const names = want.get(owner);
    if (!names) {
      undeclared.push(`${owner}: the IDL declares no such owner`);
      continue;
    }
    for (const member of reachable(value)) {
      // The module namespace's interop marker, not a member.
      if (owner === "exports" && member === "__esModule") continue;
      const path = `${owner}.${member}`;
      if (names.has(member)) continue;
      if (path in UNDECLARED) seen.add(path);
      else undeclared.push(path);
    }
  }
  expect(undeclared).toEqual([]);
  // The allow-list holds only what is there.
  expect(Object.keys(UNDECLARED).filter((p) => !seen.has(p))).toEqual([]);
});

test("every column factory and constant the IDL declares is a named export", () => {
  const exported = engineExports as unknown as Record<string, unknown>;
  const missing: string[] = [];
  for (const c of bindings.columns as Array<{ name: string }>) if (typeof exported[c.name] !== "function") missing.push(`${c.name} (column factory)`);
  for (const m of catalog["x-og-modules"]) for (const c of m.consts as Array<CatalogFn & { value?: unknown }>) {
    if (!(c.name in exported)) missing.push(`${c.name} (const of ${m.name})`);
    else if (exported[c.name] !== c.value) missing.push(`${c.name} is ${String(exported[c.name])}, declared ${String(c.value)}`);
  }
  expect(missing).toEqual([]);
  // Each makes a field the engine takes (beforeAll inserted them), holding the factory's default.
  const marked = engine.world.find("marked");
  if (marked === null) throw new Error("no marked entity");
  const value = must(engine.world.get(marked as never, Every as never)) as Record<string, unknown>;
  const fields = (Every as unknown as { fields: Record<string, { default: unknown }> }).fields;
  for (const [name, spec] of Object.entries(fields)) expect([name, value[name]]).toEqual([name, spec.default]);
  expect(Object.keys(fields).length).toBe(bindings.columns.length);
});

test("every handle type has a field factory of its own kind, and a handle round-trips through it", () => {
  // rumpus-isle #2: a font had no Font(), so a game kept it in an f64 or cast it to an Asset.
  const defs = schema.$defs as Record<string, { "x-og-kind"?: string }>;
  const handles = Object.keys(defs).filter((n) => defs[n]!["x-og-kind"] === "handle").sort();
  const columns = new Set((bindings.columns as Array<{ name: string; value: string }>).filter((c) => c.value === c.name).map((c) => c.name));
  expect(handles.filter((h) => !columns.has(h) || typeof factories[h] !== "function")).toEqual([]);
  for (const h of ["Font", "Style", "Look", "Shader", "Rig"]) expect(handles).toContain(h);
  // A value past 2^32 per kind, distinct per field (a lane mix-up shows), written and read back; an entity is a live one.
  const marked = engine.world.find("marked");
  if (marked === null) throw new Error("no marked entity");
  const written: Record<string, number> = {};
  handles.forEach((h, k) => (written[`f${h}`] = h === "Entity" ? (marked as number) : 2 ** 32 * (k + 1) + 7 * (k + 1)));
  must(engine.world.set(marked as never, Every as never, written as never));
  const back = must(engine.world.get(marked as never, Every as never)) as Record<string, unknown>;
  for (const h of handles) expect([h, back[`f${h}`]]).toEqual([h, written[`f${h}`]]);
  // And a list of one keeps it too.
  const e = must(engine.world.spawn({ name: "fonts" }));
  must(engine.world.insert(e, Fonts as never, { fonts: [2 ** 33 + 1, 5], style: 2 ** 34 + 3 } as never));
  expect(must(engine.world.get(e, Fonts as never))).toMatchObject({ fonts: [2 ** 33 + 1, 5], style: 2 ** 34 + 3 });
});
