import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";

const Tuning = defineComponent("Tuning", { speedMax: f32(8, { min: 4, max: 20 }) });
const Store = defineResource("Store", { request: u32(), best: u32() });
const Read = defineSystem("Read", { writes: [Store] }, (_q, time, world) => {
  const state = world.resource(Store);
  if (!state.ok) throw new Error(state.reason);
  if (time.tick === 0) {
    const asked = og.platform.storeGet(world, "best");
    if (asked.ok) state.value.request = asked.value;
  }
  if (!state.value.request) return;
  const answer = og.platform.result(world, state.value.request);
  if (!answer) return;
  state.value.request = 0;
  const data: unknown = answer.data;
  if (answer.ok && typeof data === "object" && data !== null && "value" in data && typeof data.value === "number") state.value.best = data.value;
});

export default og.game({
  resources: [Store],
  systems: [Read],
  start(world) {
    const entity = world.spawn({ name: "runner" });
    if (!entity.ok) throw new Error(entity.reason);
    const inserted = world.insert(entity.value, Tuning);
    if (!inserted.ok) throw new Error(inserted.reason);
  },
  facts(world) {
    const state = world.resource(Store);
    return { best: state.ok ? state.value.best : 0 };
  },
});
