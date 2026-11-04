// The Worker's asynchronous render pipelines on the published wgpu
// (src/web/pipelines.ts, fork s60 and s64), against a stand-in WebGPU device
// that records what the browser was asked.
//
// Ways it could go wrong, and the check that catches each:
// - a frame pipeline still compiled synchronously → no createRenderPipeline
//   inside the span, createRenderPipelineAsync instead
// - the requests sent in one task again → PER_TASK a task
// - a pass draws with no pipeline (a validation error) or waits for it →
//   its draws skipped and counted until a made pipeline is set
// - a stand-in handed to the browser → the made pipeline in setPipeline and
//   in a bundle
// - a refused pipeline skipped for ever → made synchronously at its use
// - a late module compiled where it was registered, or once per pipeline →
//   made in the request's task, once
import { beforeAll, expect, test } from "bun:test";

type Call = { method: string; arg: unknown };
const calls: Call[] = [];

class FakePass {
  pipeline: unknown = null;
  draws = 0;
  setPipeline(p: unknown) {
    this.pipeline = p;
  }
  draw(_n: number) {
    this.draws++;
  }
  drawIndexed(_n: number) {
    this.draws++;
  }
  drawIndirect(_b: unknown, _o: number) {
    this.draws++;
  }
  drawIndexedIndirect(_b: unknown, _o: number) {
    this.draws++;
  }
}
class FakeBundle {
  pipeline: unknown = null;
  setPipeline(p: unknown) {
    this.pipeline = p;
  }
}

let refuse = false;
let next = 0;
const device = {
  createShaderModule(desc: unknown) {
    calls.push({ method: "createShaderModule", arg: desc });
    return { module: ++next };
  },
  createRenderPipeline(desc: unknown) {
    calls.push({ method: "createRenderPipeline", arg: desc });
    return { pipeline: "sync", desc, getBindGroupLayout: (i: number) => ({ layout: i, of: (desc as { label: string }).label }) };
  },
  createRenderPipelineAsync(desc: unknown) {
    calls.push({ method: "createRenderPipelineAsync", arg: desc });
    return refuse ? Promise.reject(new Error("GPUPipelineError")) : Promise.resolve({ pipeline: "async", desc });
  },
  createComputePipeline(desc: unknown) {
    calls.push({ method: "createComputePipeline", arg: desc });
    return { compute: desc };
  },
};

let p: typeof import("../src/web/pipelines.ts");
beforeAll(async () => {
  (globalThis as Record<string, unknown>)["GPURenderPassEncoder"] = FakePass;
  (globalThis as Record<string, unknown>)["GPURenderBundleEncoder"] = FakeBundle;
  p = await import("../src/web/pipelines.ts");
  p.install(device);
});

const tasks = (n: number) => new Promise<void>((r) => setTimeout(r, n));
const made = (method: string) => calls.filter((c) => c.method === method);
const desc = (module: unknown, label: string) => ({ label, vertex: { module }, fragment: { module, targets: [] } });

test("off: every call goes straight to the browser", () => {
  calls.length = 0;
  p.enable(false);
  p.frameSpan(true);
  const m = device.createShaderModule({ code: "a" });
  const pipeline = device.createRenderPipeline(desc(m, "off")) as { pipeline: string };
  p.frameSpan(false);
  expect(pipeline.pipeline).toBe("sync");
  expect(made("createShaderModule").length).toBe(1);
  expect(made("createRenderPipelineAsync").length).toBe(0);
});

test("frame pipelines compile asynchronously, PER_TASK a task, their modules made there once", async () => {
  calls.length = 0;
  p.enable(true);
  const shared = device.createShaderModule({ code: "lit" });
  expect(made("createShaderModule").length).toBe(0);
  p.frameSpan(true);
  const stands = [0, 1, 2, 3, 4].map((i) => device.createRenderPipeline(desc(shared, `p${i}`)));
  p.frameSpan(false);
  expect(made("createRenderPipeline").length).toBe(0);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(5);

  // A pass meanwhile skips the stand-in's draws and draws after a made pipeline.
  const pass = new FakePass();
  pass.setPipeline(stands[0]);
  pass.draw(3);
  pass.drawIndexed(3);
  expect(pass.draws).toBe(0);
  expect(pass.pipeline).toBe(null);
  const ready = { pipeline: "other" };
  pass.setPipeline(ready);
  pass.draw(3);
  expect(pass.draws).toBe(1);
  expect(p.takeSkipped()).toBe(1);
  expect(p.takeSkipped()).toBe(0);

  await tasks(0);
  expect(made("createRenderPipelineAsync").length).toBe(p.PER_TASK);
  // The module is made in the first request's task, once for every pipeline that shares it.
  expect(made("createShaderModule").length).toBe(1);
  const real = (made("createRenderPipelineAsync")[0]!.arg as { vertex: { module: unknown } }).vertex.module;
  expect(real).toEqual({ module: next });
  for (let i = 0; i < 5 && made("createRenderPipelineAsync").length < 5; i++) await tasks(0);
  await tasks(0);
  expect(made("createRenderPipelineAsync").length).toBe(5);
  expect(made("createShaderModule").length).toBe(1);
  expect(p.compiling()).toBe(0);

  const after = new FakePass();
  after.setPipeline(stands[3]);
  after.draw(3);
  expect(after.draws).toBe(1);
  expect((after.pipeline as { pipeline: string; desc: { label: string } }).pipeline).toBe("async");
  expect((after.pipeline as { desc: { label: string } }).desc.label).toBe("p3");
  expect(made("createRenderPipeline").length).toBe(0);
});

test("a stand-in a bundle or a bind group layout needs is made at once, and kept", async () => {
  calls.length = 0;
  p.enable(true);
  p.frameSpan(true);
  const forBundle = device.createRenderPipeline(desc({ module: "m" }, "bundle"));
  const forLayout = device.createRenderPipeline(desc({ module: "m" }, "layout")) as { getBindGroupLayout(i: number): unknown };
  p.frameSpan(false);
  const bundle = new FakeBundle();
  bundle.setPipeline(forBundle);
  expect((bundle.pipeline as { pipeline: string }).pipeline).toBe("sync");
  expect(forLayout.getBindGroupLayout(1)).toEqual({ layout: 1, of: "layout" });
  expect(made("createRenderPipeline").length).toBe(2);
  await tasks(0);
  await tasks(0);
  // Both were made before their requests' task: none is asked for again, and none still compiles.
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(0);
  const pass = new FakePass();
  pass.setPipeline(forBundle);
  expect(pass.pipeline).toBe(bundle.pipeline);
});

test("a pipeline the browser refused is made at its use, which reports why", async () => {
  calls.length = 0;
  p.enable(true);
  refuse = true;
  p.frameSpan(true);
  const stand = device.createRenderPipeline(desc({ module: "m" }, "refused"));
  p.frameSpan(false);
  await tasks(0);
  await tasks(0);
  refuse = false;
  expect(p.compiling()).toBe(0);
  const pass = new FakePass();
  pass.setPipeline(stand);
  pass.draw(1);
  expect(pass.draws).toBe(1);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("sync");
  expect(p.takeSkipped()).toBe(0);
});

test("outside the span pipelines are made at once, with their late modules", () => {
  calls.length = 0;
  p.enable(true);
  const m = device.createShaderModule({ code: "mips" });
  const pipeline = device.createRenderPipeline(desc(m, "mips")) as { pipeline: string; desc: { vertex: { module: unknown } } };
  expect(pipeline.pipeline).toBe("sync");
  expect(pipeline.desc.vertex.module).toEqual({ module: next });
  const compute = device.createComputePipeline({ compute: { module: m } }) as { compute: { compute: { module: unknown } } };
  expect(compute.compute.compute.module).toBe(pipeline.desc.vertex.module);
  expect(made("createShaderModule").length).toBe(1);
});
