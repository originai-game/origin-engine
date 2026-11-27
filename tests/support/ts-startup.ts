// Native acceptance for the committed fixture. Run after og check + og cook.
import { strict as assert } from "node:assert";
import { fileURLToPath } from "node:url";
import { og } from "../../packages/engine/src/index.ts";
import { loadNativeHost } from "../../packages/engine/src/node/index.ts";
import { createEngine } from "../../packages/engine/src/runtime.ts";
import { readProject } from "../../packages/engine/src/node/project.ts";
import game from "../fixtures/ts-startup/src/main.ts";

const root = fileURLToPath(new URL("../fixtures/ts-startup/", import.meta.url));
loadNativeHost();
const project = readProject(root)!;
assert.equal(project.startup.saveVersion, 4);
assert.equal(project.startup.scene, "harbor");
const made = createEngine(game, { root, project: project.startup });
if (!made.ok) throw new Error(made.reason);
const engine = made.value;
try {
  engine.advance(15);
  const request = { jsonrpc: "2.0", id: 1, method: "state", params: {} };
  const state = JSON.parse(engine.call(JSON.stringify(request)));
  assert.equal(state.error, undefined);
  assert.equal(state.result.seed, 83);
  assert.equal(state.result.tick, 15);
  assert.ok(state.result.entities.some((entity: {id: string}) => entity.id === "harbor-marker"));
  assert.ok(!state.result.entities.some((entity: {id: string}) => entity.id === "inland-marker"));
  const time = engine.world.resource(og.Time);
  if (!time.ok) throw new Error(time.reason);
  assert.equal(time.value.rate, 30);
  assert.equal(time.value.elapsed, 0.5);
  const save = engine.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  const sameVersion = engine.world.restore(save.value);
  assert.ok(sameVersion.ok);
  const other = createEngine(og.game({}));
  if (!other.ok) throw new Error(other.reason);
  try {
    const refused = other.value.world.restore(save.value);
    assert.equal(refused.ok, false, "save version 4 must not restore as default version 1");
    if (!refused.ok) assert.match(refused.reason, /version/);
  } finally { other.value.destroy(); }
  const warnings = engine.takeLog().filter(line => line.warning);
  assert.deepEqual(warnings, []);
  console.log(JSON.stringify({ ok: true, query: request, state: state.result,
    expected: { tick: 15, rate: 30, elapsed: 0.5, seed: 83, scene: "harbor", entity: "harbor-marker", saveVersion: 4 } }));
} finally { engine.destroy(); }
