// Every member the IDL declares exists on the runtime object a game reaches
// it through. types/bindings.json (generated from crates/og-api/api.ogt with
// the .d.ts) lists each call with its path; the declarations alone cannot
// catch a missing member, because src/index.ts assembles `og` from plain
// objects cast to the declared type. og.load.textureArray was declared,
// typed, documented and bound on both hosts, and still `not a function`
// (examples/lantern-vale/FINDINGS.md #9); EngineProbe.listen likewise.
import { beforeAll, expect, test } from "bun:test";
import bindings from "../types/bindings.json" with { type: "json" };
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
