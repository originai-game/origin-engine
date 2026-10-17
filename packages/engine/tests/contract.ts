// Type-level assertions on the generated declarations: each `@ts-expect-error`
// line must be an error, so a declaration that grows too loose fails the
// type check ("unused @ts-expect-error"), and every other line must compile.
import { og, defineComponent, defineResource, defineSystem, f32, u32, bool, Vector3 } from "@origingame/engine";
import type { World, EngineProbe, RuntimeProbe, InjectedInput, Entity, Value, View } from "@origingame/engine";
import { PROBE_VERSION } from "@origingame/engine";

const Counter = defineComponent("Counter", { n: u32(0), alive: bool(true), at: Vector3({ x: 0, y: 1, z: 0 }) });
const Tuning = defineResource("Tuning", { speed: f32(4, { min: 0, max: 20, unit: "m", tunable: true }) });

// Columns map to the typed array their column kind declares.
defineSystem("Count", { reads: [Tuning], writes: [Counter] }, (q, time, world) => {
  for (const a of q.archetypes()) {
    const n: Uint32Array = a.column(Counter).n;
    const alive: Uint8Array = a.column(Counter).alive;
    const at: Float32Array = a.column(Counter).at;
    // @ts-expect-error a u32 column is a Uint32Array, not a Float32Array
    const wrong: Float32Array = a.column(Counter).n;
    // @ts-expect-error a field that does not exist
    a.column(og.Transform).positon;
    const e: Entity = a.entity(0);
    void [n, alive, at, wrong, e];
  }
  // @ts-expect-error Time is read only for game code
  time.tick = 3;
  const tuning = world.resource(Tuning);
  if (tuning.ok) {
    const speed: number = tuning.value.speed;
    void speed;
  }
});

// A desc is optional when it is last, and refuses fields it does not declare.
og.game();
// @ts-expect-error `tik` is not a GameDesc field
og.game({ tik: 60 });
// @ts-expect-error the run function cannot be omitted after a desc
defineSystem("Empty", {});

// @ts-expect-error a layout field must come from a column factory
defineComponent("Bad", { hp: 5 });

function world(w: World): void {
  const spawned = w.spawn({ name: "a" });
  // @ts-expect-error a Result must be checked before its value is read
  spawned.value;
  if (!spawned.ok) {
    const reason: string = spawned.reason;
    void reason;
    return;
  }
  w.insert(spawned.value, Counter, { n: 3 });
  // @ts-expect-error insert takes the component's own field types
  w.insert(spawned.value, Counter, { n: "3" });
  const got = w.get(spawned.value, og.Transform);
  if (got.ok) {
    const x: number = got.value.position.x;
    void x;
  }
  const input = w.resource(og.Input);
  if (input.ok) {
    const held: boolean = input.value.down("KeyW");
    // @ts-expect-error pointer buttons are the protocol's names
    input.value.buttonDown("primary");
    void held;
  }
  const view: View<typeof og.Transform> = null as unknown as View<typeof og.Transform>;
  const value: Value<typeof og.Transform> = { position: { x: 0, y: 0, z: 0 }, quaternion: { x: 0, y: 0, z: 0, w: 1 }, scale: { x: 1, y: 1, z: 1 } };
  void [view, value];
}

// The probe: the engine's probe is a protocol-5 RuntimeProbe with its extensions.
function probe(p: EngineProbe): void {
  const asRuntime: RuntimeProbe = p;
  const version: 5 = PROBE_VERSION;
  const event: InjectedInput = { kind: "key", key: "Space", action: "press" };
  asRuntime.input(event);
  // @ts-expect-error a key event names its key
  asRuntime.input({ kind: "key", action: "press" });
  p.capture();
  p.sweep({ id: "player", parameter: "Health.hp", values: [10, 50], steps: 30 });
  p.dump({ what: "entity", id: "player" });
  const result = p.run([{ op: "advance", steps: 60 }, { op: "read" }, { op: "pick", x: 10, y: 20 }]);
  const refused: number | null = result.refused;
  void [version, refused];
}

void world;
void probe;
