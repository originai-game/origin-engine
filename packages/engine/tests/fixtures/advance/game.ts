import { defineResource, defineSystem, og, u32 } from "../../../src/index.ts";

export const State = defineResource("AdvanceState", { request: u32(), answered: u32(), pressedAt: u32(), releasedAt: u32() });
const Input = defineSystem("AdvanceInput", { reads: [og.Input], writes: [State] }, (_q, time, world) => {
  const state = world.resource(State);
  const input = world.resource(og.Input);
  if (!state.ok || !input.ok) throw new Error("advance fixture resources are missing");
  if (input.value.pressed("KeyW")) {
    state.value.pressedAt = time.tick;
    const request = og.platform.storeGet(world, "record");
    if (!request.ok) throw new Error(request.reason);
    state.value.request = request.value;
  }
  if (input.value.released("KeyW")) state.value.releasedAt = time.tick;
  if (state.value.request && og.platform.result(world, state.value.request)) state.value.answered++;
});

export default og.game({ resources: [State], systems: [Input] });
