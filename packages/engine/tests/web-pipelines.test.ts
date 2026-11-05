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
// - a pass whose output later frames keep (encoded outside a span: a static
//   shadow map, a probe capture, a bake) skips a draw → it makes the
//   pipeline at once instead, as a harness's frame (drawnWhole) does, and as
//   every frame does under prewarm (only `warming` asks asynchronously)
// - a stand-in handed to the browser → the made pipeline in setPipeline and
//   in a bundle
// - a refused pipeline skipped for ever → made synchronously at its use
// - a late module compiled where it was registered, or once per pipeline →
//   made in the request's task, once
// - a frame pipeline that reaches the browser's createRenderPipeline while the
//   hook is on (a wgpu that took the function from the prototype at load,
//   or one taken before the Worker installed): the first is still asynchronous,
//   the second is named by `bypassed` from the counts the probe reports
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
class FakeDevice {
  createShaderModule(desc: unknown) {
    calls.push({ method: "createShaderModule", arg: desc });
    return { module: ++next };
  }
  createRenderPipeline(desc: unknown) {
    calls.push({ method: "createRenderPipeline", arg: desc });
    return { pipeline: "sync", desc, getBindGroupLayout: (i: number) => ({ layout: i, of: (desc as { label: string }).label }) };
  }
  createRenderPipelineAsync(desc: unknown) {
    calls.push({ method: "createRenderPipelineAsync", arg: desc });
    return refuse ? Promise.reject(new Error("GPUPipelineError")) : Promise.resolve({ pipeline: "async", desc });
  }
  createComputePipeline(desc: unknown) {
    calls.push({ method: "createComputePipeline", arg: desc });
    return { compute: desc };
  }
}
// What a wgpu that bypassed the wrappers would call: the browser's own function, taken before install.
const browserCreate = FakeDevice.prototype.createRenderPipeline;
const device = new FakeDevice();

let p: typeof import("../src/web/pipelines.ts");
beforeAll(async () => {
  (globalThis as Record<string, unknown>)["GPURenderPassEncoder"] = FakePass;
  (globalThis as Record<string, unknown>)["GPURenderBundleEncoder"] = FakeBundle;
  p = await import("../src/web/pipelines.ts");
  p.install(FakeDevice.prototype);
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
  expect(made("createRenderPipeline").length).toBe(0);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(5);

  // A pass encoded in the span meanwhile skips the stand-in's draws and draws after a made pipeline.
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
  p.frameSpan(false);
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

test("a pass outside a span makes a stand-in at once instead of skipping its draws", async () => {
  calls.length = 0;
  p.enable(true);
  p.frameSpan(true);
  const stand = device.createRenderPipeline(desc({ module: "m" }, "kept"));
  p.frameSpan(false);
  const before = p.stats();
  const pass = new FakePass();
  pass.setPipeline(stand);
  pass.draw(3);
  expect(pass.draws).toBe(1);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("sync");
  expect(p.takeSkipped()).toBe(0);
  expect(p.stats().fallback - before.fallback).toBe(1);
  await tasks(0);
  await tasks(0);
  // Made before its request's task: never asked for again.
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(0);
});

test("a harness's frame is drawn whole: its pipelines made at once and no draw skipped", async () => {
  calls.length = 0;
  p.enable(true);
  p.frameSpan(true);
  const earlier = device.createRenderPipeline(desc({ module: "m" }, "earlier"));
  p.frameSpan(false);
  const pass = new FakePass();
  const fresh = p.drawnWhole(() => {
    p.frameSpan(true);
    const now = device.createRenderPipeline(desc({ module: "m" }, "now")) as { pipeline: string };
    pass.setPipeline(earlier);
    pass.draw(3);
    p.frameSpan(false);
    return now;
  });
  expect(fresh.pipeline).toBe("sync");
  expect(pass.draws).toBe(1);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("sync");
  expect(p.takeSkipped()).toBe(0);
  await tasks(0);
  await tasks(0);
  expect(p.compiling()).toBe(0);
});

test("prewarm: a frame makes its pipelines at once, only the warm-up asks for them asynchronously", async () => {
  calls.length = 0;
  p.enable(true, true);
  // A frame's own pipeline, in a span: made now, nothing compiling, nothing skipped.
  p.frameSpan(true);
  const framed = device.createRenderPipeline(desc({ module: "m" }, "framed")) as { pipeline: string };
  p.frameSpan(false);
  expect(framed.pipeline).toBe("sync");
  expect(p.compiling()).toBe(0);
  // The warm-up behind the gate: asked for asynchronously.
  const warmed = p.warming(() => {
    p.frameSpan(true);
    const w = device.createRenderPipeline(desc({ module: "m" }, "warmed"));
    p.frameSpan(false);
    return w;
  });
  expect(p.compiling()).toBe(1);
  // A frame that meets it before it is made makes it at once: its draws are never skipped.
  const pass = new FakePass();
  p.frameSpan(true);
  pass.setPipeline(warmed);
  pass.draw(3);
  p.frameSpan(false);
  expect(pass.draws).toBe(1);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("sync");
  expect(p.takeSkipped()).toBe(0);
  await tasks(0);
  await tasks(0);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(0);
  p.enable(true);
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

test("while the hook is on, no frame pipeline reaches the browser synchronously, however wgpu calls it", async () => {
  await tasks(0);
  calls.length = 0;
  p.enable(true);
  const before = p.stats();
  // The function as a glue that took it from the prototype after the Worker started would call it.
  const cached = FakeDevice.prototype.createRenderPipeline;
  const bound = Function.prototype.call.bind(FakeDevice.prototype.createRenderPipeline) as (d: unknown, desc: unknown) => unknown;
  p.frameSpan(true);
  device.createRenderPipeline(desc({ module: "m" }, "structural"));
  cached.call(device, desc({ module: "m" }, "cached"));
  bound(device, desc({ module: "m" }, "bound"));
  p.frameSpan(false);
  for (let i = 0; i < 4; i++) await tasks(0);
  expect(made("createRenderPipeline").length).toBe(0);
  expect(made("createRenderPipelineAsync").length).toBe(3);
  const after = p.stats();
  expect([after.async - before.async, after.sync - before.sync, after.fallback - before.fallback, after.spans - before.spans]).toEqual([3, 0, 0, 1]);
  expect(p.bypassed(after, 3)).toBe(null);
  // Outside a span the same call is a synchronous one, and counted as such.
  device.createRenderPipeline(desc({ module: "m" }, "one-shot"));
  expect(p.stats().sync - after.sync).toBe(1);
});

test("frame pipelines that reached the browser without the wrappers are named, not hidden", () => {
  calls.length = 0;
  p.enable(true);
  const s = p.stats();
  p.frameSpan(true);
  browserCreate.call(device, desc({ module: "m" }, "bypass"));
  p.frameSpan(false);
  expect(made("createRenderPipeline").length).toBe(1);
  // A Worker whose renderer warmed pipelines in its spans while no request was ever sent.
  const lost = { ...p.stats(), async: 0 };
  expect(lost.spans).toBe(s.spans + 1);
  expect(p.bypassed(lost, 4)).toMatch(/none through createRenderPipelineAsync/);
  // Nothing warmed, no span, or the hook off: nothing to say.
  expect(p.bypassed(lost, 0)).toBe(null);
  expect(p.bypassed({ ...lost, spans: 0 }, 4)).toBe(null);
  p.enable(false);
  expect(p.bypassed(lost, 4)).toBe(null);
  p.enable(true);
  expect(p.bypassed(p.stats(), 4)).toBe(null);
  // Under prewarm a frame's spans are synchronous by design: only a warm-up that asked for nothing is named.
  p.enable(true, true);
  p.frameSpan(true);
  p.frameSpan(false);
  expect(p.bypassed(lost, 4)).toBe(null);
  p.warming(() => {
    p.frameSpan(true);
    browserCreate.call(device, desc({ module: "m" }, "warm bypass"));
    p.frameSpan(false);
  });
  expect(p.bypassed(lost, 4)).toMatch(/none through createRenderPipelineAsync/);
  p.enable(true);
});

test("auto picks async on a mobile GPU only, and the page reports the pick and why", () => {
  const mode = (asked: Parameters<typeof p.choose>[0], info: Parameters<typeof p.choose>[1], mobile?: boolean) => p.choose(asked, info, mobile).mode;
  // The adapter's vendor or architecture names it; it outranks the user agent's hint.
  expect(mode(undefined, { vendor: "img-tec", architecture: "d-series" })).toBe("async");
  expect(mode(undefined, { vendor: "qualcomm", architecture: "adreno-7xx" })).toBe("async");
  expect(mode(undefined, { vendor: "arm", architecture: "" })).toBe("async");
  expect(mode(undefined, { vendor: "", architecture: "valhall" })).toBe("async");
  expect(mode(undefined, { vendor: "nvidia", architecture: "blackwell" }, true)).toBe("sync");
  // Apple GPUs stay sync until iOS is measured, an iPhone's included.
  expect(mode(undefined, { vendor: "apple", architecture: "metal-3" }, true)).toBe("sync");
  // Firefox's empty strings: navigator.userAgentData.mobile where it exists, else sync.
  expect(mode(undefined, { vendor: "", architecture: "" }, true)).toBe("async");
  expect(mode(undefined, { vendor: "", architecture: "" }, false)).toBe("sync");
  expect(mode(undefined, undefined, undefined)).toBe("sync");
  // What the build asked is kept, on any GPU.
  expect(mode("sync", { vendor: "qualcomm" }, true)).toBe("sync");
  expect(mode("prewarm", { vendor: "apple" }, false)).toBe("prewarm");
  expect(p.choose(undefined, { vendor: "qualcomm", architecture: "adreno-7xx" }, undefined).reason).toMatch(/mobile GPU \(qualcomm adreno-7xx\)/);

  // Auto's sync on a desktop: the renderer's hook stays installed, but its spans make nothing asynchronous and count nothing.
  p.use(p.choose(undefined, { vendor: "apple", architecture: "metal-3" }, false));
  const before = p.stats();
  expect([before.mode, before.reason]).toEqual(["sync", "auto: an Apple GPU (apple metal-3), sync until iOS is measured"]);
  calls.length = 0;
  p.frameSpan(true);
  device.createRenderPipeline(desc({ module: "m" }, "desktop"));
  p.frameSpan(false);
  expect(made("createRenderPipeline").length).toBe(1);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  const after = p.stats();
  expect([after.spans - before.spans, after.sync - before.sync, after.async - before.async]).toEqual([0, 1, 0]);
  p.use(p.choose(undefined, { vendor: "img-tec", architecture: "d-series" }, undefined));
  expect(p.stats().mode).toBe("async");
  p.enable(false);
});
