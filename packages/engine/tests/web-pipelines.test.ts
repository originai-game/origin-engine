// The web host's pipeline compiler (src/web/pipelines.ts): what og-render's
// pipeline cache (crates/og-render/src/gpu/deferred.rs) calls to compile a
// pipeline off the browser's GPU-process main task, against a stand-in WebGPU
// device that records what the browser was asked.
//
// Ways it could go wrong, and the check that catches each:
// - a pipeline the cache makes still compiled synchronously → no
//   createRenderPipeline inside `asking`, createRenderPipelineAsync instead
// - the requests sent in one task again → PER_TASK a task
// - the cache's numbers drift from the stand-ins (a wrong pipeline reported
//   compiled) → `count`, `made` and `isCompiled` per number
// - a use that waits gets nothing → `finish(n)` makes it synchronously, once,
//   and no request is sent for it later
// - a stand-in handed to the browser → the made pipeline in setPipeline, in a
//   bundle and in a compute pass
// - every draw of every frame through a wrapper → the prototype keeps the
//   browser's draws
// - a refused pipeline never reported → compiled (refused), made at its use
// - a late module compiled where it was registered, or once per pipeline →
//   made in the request's task, once
// - a pipeline that reaches the browser's createRenderPipeline while on (a
//   wgpu that took the function from the prototype at load, or one taken
//   before the Worker installed): the first is still asynchronous, the second
//   is named by `bypassed` from the counts the probe reports
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
class FakeCompute {
  pipeline: unknown = null;
  dispatches = 0;
  setPipeline(p: unknown) {
    this.pipeline = p;
  }
  dispatchWorkgroups(_x: number) {
    this.dispatches++;
  }
  dispatchWorkgroupsIndirect(_b: unknown, _o: number) {
    this.dispatches++;
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
  createComputePipelineAsync(desc: unknown) {
    calls.push({ method: "createComputePipelineAsync", arg: desc });
    return new Promise((resolve) => lateCompute.push(() => resolve({ compute: "async", desc })));
  }
}
/** The compute pipelines asked for asynchronously, each made when its function runs. */
const lateCompute: (() => void)[] = [];
// What a wgpu that bypassed the wrappers would call: the browser's own function, taken before install.
const browserCreate = FakeDevice.prototype.createRenderPipeline;
// The browser's own draw methods, taken before install.
const DRAWS = ["draw", "drawIndexed", "drawIndirect", "drawIndexedIndirect"];
const passProto = FakePass.prototype as unknown as Record<string, unknown>;
const browserDraws = DRAWS.map((name) => passProto[name]);
const device = new FakeDevice();

let p: typeof import("../src/web/pipelines.ts");
beforeAll(async () => {
  (globalThis as Record<string, unknown>)["GPURenderPassEncoder"] = FakePass;
  (globalThis as Record<string, unknown>)["GPURenderBundleEncoder"] = FakeBundle;
  (globalThis as Record<string, unknown>)["GPUComputePassEncoder"] = FakeCompute;
  p = await import("../src/web/pipelines.ts");
  p.install(FakeDevice.prototype);
});

const tasks = (n: number) => new Promise<void>((r) => setTimeout(r, n));
const made = (method: string) => calls.filter((c) => c.method === method);
const desc = (module: unknown, label: string) => ({ label, vertex: { module }, fragment: { module, targets: [] } });

test("off: every call goes straight to the browser", () => {
  calls.length = 0;
  p.enable(false);
  p.asking(true);
  const m = device.createShaderModule({ code: "a" });
  const pipeline = device.createRenderPipeline(desc(m, "off")) as { pipeline: string };
  p.asking(false);
  expect(pipeline.pipeline).toBe("sync");
  expect(made("createShaderModule").length).toBe(1);
  expect(made("createRenderPipelineAsync").length).toBe(0);
});

test("the cache's pipelines compile asynchronously, PER_TASK a task, numbered, their modules made there once", async () => {
  calls.length = 0;
  p.enable(true);
  const shared = device.createShaderModule({ code: "lit" });
  expect(made("createShaderModule").length).toBe(0);
  const first = p.count() + 1;
  const before = p.counted();
  const stands = [0, 1, 2, 3, 4].map((i) => {
    p.asking(true);
    const s = device.createRenderPipeline(desc(shared, `p${i}`));
    p.asking(false);
    return s;
  });
  expect(made("createRenderPipeline").length).toBe(0);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(5);
  expect(p.count()).toBe(first + 4);
  expect([0, 1, 2, 3, 4].map((i) => p.isCompiled(first + i))).toEqual([false, false, false, false, false]);
  // The splash's count: five more requested, none of them ready yet.
  expect(p.counted()).toEqual({ requested: before.requested + 5, ready: before.ready });

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
  expect([0, 1, 2, 3, 4].map((i) => p.isCompiled(first + i))).toEqual([true, true, true, true, true]);
  expect(p.counted()).toEqual({ requested: before.requested + 5, ready: before.requested + 5 });
  expect(p.made()).toBe(p.count());

  // A pass draws with the browser's pipeline, through the browser's own draw methods.
  DRAWS.forEach((name, i) => expect(passProto[name]).toBe(browserDraws[i]));
  const pass = new FakePass();
  pass.setPipeline(stands[3]);
  pass.draw(3);
  expect(pass.draws).toBe(1);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("async");
  expect((pass.pipeline as { desc: { label: string } }).desc.label).toBe("p3");
  expect(made("createRenderPipeline").length).toBe(0);
});

test("finish makes a pipeline still compiling at once, and it is not asked for again", async () => {
  calls.length = 0;
  p.enable(true);
  p.asking(true);
  const stand = device.createRenderPipeline(desc({ module: "m" }, "waited"));
  p.asking(false);
  const n = p.count();
  const before = p.stats();
  expect(p.isCompiled(n)).toBe(false);
  p.finish(n);
  expect(p.isCompiled(n)).toBe(true);
  p.finish(n);
  expect(made("createRenderPipeline").length).toBe(1);
  expect(p.stats().waited - before.waited).toBe(1);
  const pass = new FakePass();
  pass.setPipeline(stand);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("sync");
  await tasks(0);
  await tasks(0);
  // Made before its request's task: never asked for again.
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(0);
});

test("a stand-in a bundle or a bind group layout needs is made at once, and kept", async () => {
  calls.length = 0;
  p.enable(true);
  p.asking(true);
  const forBundle = device.createRenderPipeline(desc({ module: "m" }, "bundle"));
  const forLayout = device.createRenderPipeline(desc({ module: "m" }, "layout")) as { getBindGroupLayout(i: number): unknown };
  p.asking(false);
  const bundle = new FakeBundle();
  bundle.setPipeline(forBundle);
  expect((bundle.pipeline as { pipeline: string }).pipeline).toBe("sync");
  expect(forLayout.getBindGroupLayout(1)).toEqual({ layout: 1, of: "layout" });
  expect(made("createRenderPipeline").length).toBe(2);
  await tasks(0);
  await tasks(0);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  expect(p.compiling()).toBe(0);
  const pass = new FakePass();
  pass.setPipeline(forBundle);
  expect(pass.pipeline).toBe(bundle.pipeline);
});

test("a pipeline the browser refused counts as compiled and is made at its use, which reports why", async () => {
  calls.length = 0;
  p.enable(true);
  refuse = true;
  p.asking(true);
  const stand = device.createRenderPipeline(desc({ module: "m" }, "refused"));
  p.asking(false);
  const n = p.count();
  await tasks(0);
  await tasks(0);
  refuse = false;
  expect(p.compiling()).toBe(0);
  expect(p.isCompiled(n)).toBe(true);
  const pass = new FakePass();
  pass.setPipeline(stand);
  pass.draw(1);
  expect(pass.draws).toBe(1);
  expect((pass.pipeline as { pipeline: string }).pipeline).toBe("sync");
});

test("outside the cache's asking pipelines are made at once, with their late modules", () => {
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

test("compute pipelines take the same path: asked for asynchronously, made by finish or their dispatch", async () => {
  await tasks(0);
  calls.length = 0;
  lateCompute.length = 0;
  p.enable(true);
  p.asking(true);
  const late = device.createComputePipeline({ label: "place", compute: { module: "m" } });
  const waited = device.createComputePipeline({ label: "cull", compute: { module: "m" } });
  p.asking(false);
  const n = p.count();
  for (let i = 0; i < 3; i++) await tasks(0);
  expect(made("createComputePipeline").length).toBe(0);
  expect(made("createComputePipelineAsync").length).toBe(2);
  expect([p.isCompiled(n - 1), p.isCompiled(n)]).toEqual([false, false]);
  p.finish(n);
  expect(made("createComputePipeline").length).toBe(1);
  const c = new FakeCompute();
  c.setPipeline(waited);
  expect((c.pipeline as { compute: { label: string } }).compute.label).toBe("cull");
  for (const resolve of lateCompute.splice(0)) resolve();
  await tasks(0);
  expect(p.isCompiled(n - 1)).toBe(true);
  expect(p.compiling()).toBe(0);
  const d = new FakeCompute();
  d.setPipeline(late);
  expect((d.pipeline as { compute: string }).compute).toBe("async");
  expect(made("createComputePipeline").length).toBe(1);
});

test("prewarm: only the warm-up behind the gate asks for pipelines asynchronously", async () => {
  calls.length = 0;
  p.enable(true, true);
  p.asking(true);
  const framed = device.createRenderPipeline(desc({ module: "m" }, "framed")) as { pipeline: string };
  p.asking(false);
  expect(framed.pipeline).toBe("sync");
  expect(p.compiling()).toBe(0);
  p.warming(() => {
    p.asking(true);
    device.createRenderPipeline(desc({ module: "m" }, "warmed"));
    p.asking(false);
  });
  expect(p.compiling()).toBe(1);
  await tasks(0);
  await tasks(0);
  expect(made("createRenderPipelineAsync").length).toBe(1);
  expect(p.compiling()).toBe(0);
  p.enable(true);
});

test("while on, no pipeline the cache makes reaches the browser synchronously, however wgpu calls it", async () => {
  await tasks(0);
  calls.length = 0;
  p.enable(true);
  const before = p.stats();
  // The function as a glue that took it from the prototype after the Worker started would call it.
  const cached = FakeDevice.prototype.createRenderPipeline;
  const bound = Function.prototype.call.bind(FakeDevice.prototype.createRenderPipeline) as (d: unknown, desc: unknown) => unknown;
  p.asking(true);
  device.createRenderPipeline(desc({ module: "m" }, "structural"));
  cached.call(device, desc({ module: "m" }, "cached"));
  bound(device, desc({ module: "m" }, "bound"));
  p.asking(false);
  for (let i = 0; i < 4; i++) await tasks(0);
  expect(made("createRenderPipeline").length).toBe(0);
  expect(made("createRenderPipelineAsync").length).toBe(3);
  const after = p.stats();
  expect([after.async - before.async, after.sync - before.sync, after.waited - before.waited, after.asks - before.asks]).toEqual([3, 0, 0, 1]);
  expect(p.bypassed(after, 3)).toBe(null);
  device.createRenderPipeline(desc({ module: "m" }, "one-shot"));
  expect(p.stats().sync - after.sync).toBe(1);
});

test("pipelines that reached the browser without the wrappers are named, not hidden", () => {
  calls.length = 0;
  p.enable(true);
  const s = p.stats();
  p.asking(true);
  browserCreate.call(device, desc({ module: "m" }, "bypass"));
  p.asking(false);
  expect(made("createRenderPipeline").length).toBe(1);
  // A Worker whose cache asked with the compiler on while no request was ever sent.
  const lost = { ...p.stats(), async: 0 };
  expect(lost.asks).toBe(s.asks + 1);
  expect(p.bypassed(lost, 4)).toMatch(/none through createRenderPipelineAsync/);
  expect(p.bypassed(lost, 0)).toBe(null);
  expect(p.bypassed({ ...lost, asks: 0 }, 4)).toBe(null);
  p.enable(false);
  expect(p.bypassed(lost, 4)).toBe(null);
  p.enable(true);
});

test("auto picks async on every GPU but Apple's, and the page reports the pick and why", () => {
  const mode = (asked: Parameters<typeof p.choose>[0], info: Parameters<typeof p.choose>[1]) => p.choose(asked, info).mode;
  expect(mode(undefined, { vendor: "img-tec", architecture: "d-series" })).toBe("async");
  expect(mode(undefined, { vendor: "qualcomm", architecture: "adreno-7xx" })).toBe("async");
  expect(mode(undefined, { vendor: "", architecture: "valhall" })).toBe("async");
  // A desktop GPU too: a cold driver cache compiles synchronous ones past Chrome's GPU watchdog.
  expect(mode(undefined, { vendor: "nvidia", architecture: "lovelace" })).toBe("async");
  expect(p.choose(undefined, { vendor: "nvidia", architecture: "lovelace" }).reason).toMatch(/not an Apple GPU \(nvidia lovelace\)/);
  expect(mode(undefined, { vendor: "", architecture: "" })).toBe("async");
  expect(mode(undefined, undefined)).toBe("async");
  expect(mode(undefined, { vendor: "apple", architecture: "metal-3" })).toBe("sync");
  expect(mode("sync", { vendor: "qualcomm" })).toBe("sync");
  expect(mode("prewarm", { vendor: "apple" })).toBe("prewarm");

  // Auto's sync on Apple: the cache's asking makes nothing asynchronous and counts nothing.
  p.use(p.choose(undefined, { vendor: "apple", architecture: "metal-3" }));
  const before = p.stats();
  expect([before.mode, before.reason]).toEqual(["sync", "auto: an Apple GPU (apple metal-3), no consistent async advantage on M4 Pro"]);
  calls.length = 0;
  p.asking(true);
  device.createRenderPipeline(desc({ module: "m" }, "desktop"));
  p.asking(false);
  expect(made("createRenderPipeline").length).toBe(1);
  expect(made("createRenderPipelineAsync").length).toBe(0);
  const after = p.stats();
  expect([after.asks - before.asks, after.sync - before.sync, after.async - before.async]).toEqual([0, 1, 0]);
  p.use(p.choose(undefined, { vendor: "img-tec", architecture: "d-series" }));
  expect(p.stats().mode).toBe("async");
  p.enable(false);
});
