// Entity arguments on the native host: a value that is not an entity handle
// is refused with the call and the parameter named, and world.spawn's
// implicit writes say why a system needs them (M7 run 4, where each message
// cost an agent several iterations).
//
// Ways it could fail, and the check that catches each:
// - null reaches the addon and napi throws its own conversion error, naming
//   neither the call nor the parameter (the web host coerces it to 0), or a
//   Result passed as SpawnDesc.parent is taken as no parent → "null where an
//   entity is expected"
// - world.spawn's refusal of an undeclared og.Parent or og.Transform names
//   the component but not that spawn wrote it → "world.spawn's implicit writes"
// - a bare index (Archetype.entities) refused without the call, without a
//   hint, or with nothing in the engine log → "a bare index"
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineSystem, f32, type Entity } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function failure(system: unknown): string {
  const created = createEngine(og.game({ systems: [system] } as never) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const ran = engine.advance(1);
  engine.destroy();
  return ran.ok ? "" : ran.reason;
}

test("null where an entity is expected names the call and the parameter", () => {
  const S = defineSystem("EaSetMissing", { writes: [og.CharacterController] }, (_q, _t, world) => {
    const h = world.find("hero")!;
    world.set(h, og.CharacterController, { move: { x: 3, y: 0, z: 0 } });
  });
  const reason = failure(S);
  expect(reason).toContain("world.set: entity is null (world.find returns null when no entity has the name)");
  expect(reason).not.toContain("napi");

  const D = defineSystem("EaDespawnUndefined", { writes: [] }, (_q, _t, world) => {
    world.despawn(undefined as unknown as Entity);
  });
  expect(failure(D)).toContain("world.despawn: entity is undefined");

  // SpawnDesc.parent crosses as JSON: a Result there was silently no parent.
  const P = defineSystem("EaSpawnResultParent", { writes: [og.Transform, og.Parent] }, (_q, _t, world) => {
    const child = world.spawn({ parent: world.spawn({}) as unknown as Entity });
    if (!child.ok) throw new Error(child.reason);
  });
  expect(failure(P)).toContain("world.spawn: parent is a Result, not an entity (world.spawn answers { ok, value }: pass its value)");
});

test("world.spawn's implicit writes name their cause", () => {
  const P = defineSystem("EaSpawnParent", { writes: [og.Transform] }, (_q, _t, world) => {
    const parent = world.spawn({});
    if (!parent.ok) throw new Error(parent.reason);
    const child = world.spawn({ parent: parent.value });
    if (!child.ok) throw new Error(child.reason);
  });
  expect(failure(P)).toContain(
    "system 'EaSpawnParent' did not declare write access to 'og.Parent' (world.spawn with parent writes og.Parent; add og.Parent to the system's writes)",
  );

  const T = defineSystem("EaSpawnTransform", { writes: [] }, (_q, _t, world) => {
    const e = world.spawn({ name: "t" });
    if (!e.ok) throw new Error(e.reason);
  });
  expect(failure(T)).toContain(
    "system 'EaSpawnTransform' did not declare write access to 'og.Transform' (world.spawn writes og.Transform unless the desc says bare: true; add og.Transform to the system's writes)",
  );
});

test("a bare index (Archetype.entities) is refused naming the call, with the hint, and logged", () => {
  const Mark = defineComponent("EaMark", { v: f32(0) });
  const reasons: string[] = [];
  const S = defineSystem("EaDespawnIndex", { writes: [Mark] }, (q, _t, world) => {
    for (const a of q.archetypes()) {
      for (let i = 0; i < a.count; i++) {
        // The agent's loop: the pairs' numbers taken for handles.
        const index = a.entities[i * 2]! as unknown as Entity;
        const generation = a.entities[i * 2 + 1]! as unknown as Entity;
        for (const r of [world.despawn(index), world.get(generation, Mark)]) {
          if (!r.ok) reasons.push(r.reason);
        }
      }
    }
  });
  const created = createEngine(og.game({ systems: [S], components: [Mark] } as never) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  for (let i = 0; i < 3; i++) {
    const e = engine.world.spawn({});
    if (!e.ok) throw new Error(e.reason);
    expect(engine.world.insert(e.value, Mark, { v: 1 }).ok).toBe(true);
  }
  engine.takeLog();
  const ran = engine.advance(1);
  expect(ran.ok, ran.ok ? "" : ran.reason).toBe(true);
  const hint = "Archetype.entities holds (index, generation) pairs; use a.entity(i) for a handle";
  expect(reasons.length).toBe(6);
  expect(reasons.filter((r) => r.startsWith("world.despawn: no live entity has the handle")).length).toBe(3);
  expect(reasons.filter((r) => r.startsWith("world.get: no live entity has the handle")).length).toBe(3);
  for (const r of reasons) expect(r).toContain(hint);
  // Ignored Results still show in the engine log (og run prints it), once per system and call.
  const logged = engine.takeLog().filter((l) => l.warning && l.text.includes(hint));
  expect(logged.map((l) => l.text.split(":")[0])).toEqual(["system 'EaDespawnIndex', world.despawn", "system 'EaDespawnIndex', world.get"]);
  // Nothing was despawned.
  expect(engine.world.query({ with: [Mark] }).ok && (engine.world.query({ with: [Mark] }) as { value: { count(): number } }).value.count()).toBe(3);
  engine.destroy();
});
