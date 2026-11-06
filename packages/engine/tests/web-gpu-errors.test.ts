// The Worker's error scopes on the published wgpu (src/web/gpu-errors.ts),
// against stand-in WebGPU error classes: wgpu 30 panics on a GPUInternalError
// popped from a scope, which is how PowerVR refuses the visibility buffer's
// resolve (the former test sample
//
// Ways it could go wrong, and the check that catches each:
// - the internal error still reaches wgpu → a GPUValidationError instead,
//   with the driver's message (the refusal RenderCapabilities reports)
// - a validation or clean scope changed on the way → the same error, or null
// - the device lost as `this` (a scope popped on another device) → the
//   original popErrorScope called on the device the renderer popped
import { expect, test } from "bun:test";
import { INTERNAL, install } from "../src/web/gpu-errors.ts";

class GPUError {
  readonly message: string;
  constructor(message: string) {
    this.message = message;
  }
}
class GPUInternalError extends GPUError {}
class GPUValidationError extends GPUError {}

class FakeDevice {
  next: unknown = null;
  popped: unknown[] = [];
  popErrorScope(): Promise<unknown> {
    this.popped.push(this);
    return Promise.resolve(this.next);
  }
}
install(FakeDevice.prototype, { GPUInternalError, GPUValidationError });

test("a GPUInternalError reaches wgpu as a GPUValidationError with its message", async () => {
  const d = new FakeDevice();
  d.next = new GPUInternalError("CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN");
  const e = await d.popErrorScope();
  expect(e instanceof GPUValidationError).toBe(true);
  expect(e instanceof GPUInternalError).toBe(false);
  expect((e as GPUError).message).toBe(`${INTERNAL}CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN`);
  expect(d.popped).toEqual([d]);
});

test("validation errors and clean scopes pass through unchanged", async () => {
  const d = new FakeDevice();
  const v = new GPUValidationError("bad layout");
  d.next = v;
  expect(await d.popErrorScope()).toBe(v);
  d.next = null;
  expect(await d.popErrorScope()).toBeNull();
});
