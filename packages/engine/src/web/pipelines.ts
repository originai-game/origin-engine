// Render pipelines compiled off the browser's GPU-process main task, on the
// published wgpu (fork s60 and s64, which patched wgpu's WebGPU backend for
// this; the engine never patches wgpu, AGENTS.md).
//
// `createRenderPipeline` compiles inside one task of Chrome's GPU process: a
// frame that makes dozens of pipelines (a large game's first frame) holds
// that process, and with it the page's frames and the WebGPU queue, for as
// long as they all take. `createRenderPipelineAsync` compiles on Dawn's
// worker threads. wgpu's WebGPU backend only calls the synchronous one, as a
// method of the device (wasm-bindgen's structural calls), so the Worker
// replaces the methods on GPUDevice.prototype when it starts, before the
// wasm's glue loads (a glue that took the function from the prototype at
// load gets these too):
//
// - While the renderer makes its frame pipelines (og-render's
//   `set_frame_pipelines_hook`, `span`), `createRenderPipeline` asks for
//   `createRenderPipelineAsync` and hands back a stand-in at once, which the
//   browser's pipeline fills in when the promise resolves. The requests are
//   sent `PER_TASK` a task, so a frame's dozens are not one GPU-process task
//   again.
// - The renderer encodes the passes it draws again every frame inside a span
//   too (og-render `FrameSpan`, `FramePass::redrawn`), so the pipelines a
//   pass makes when it first sets one (og-render deferred.rs: a 2D game's
//   sprites, text and tonemap) are asked for this way as well.
// - A render pass's `setPipeline` of a stand-in not filled in yet, inside a
//   span, skips that pipeline's draws (counted) instead of waiting; outside
//   one (a pass whose output later frames keep), a render bundle, a bind
//   group layout asked of it, or one the browser refused makes it then, with
//   `createRenderPipeline` (which reports a refused one's error as before).
//   A harness's frame (`drawnWhole`) is drawn whole the same way, as natively.
// - While on (`enable`), a WGSL module is created at the first pipeline that
//   uses it (s64), so a game's dozens of material modules are compiled
//   (Tint) in the pipeline requests' tasks, not all in the task that
//   registered them; any other use makes it then.
//
// Off (the default), every call goes straight to the browser.
//
// A wgpu that reached the browser's createRenderPipeline some other way would
// lose all this silently: frames would compile synchronously again. So the
// pipelines made each way are counted (`stats`), the Worker sends them with
// the first frame (BootPhases `pipelines`, the probe's `boot`), and `bypassed`
// names the loss: warm pipelines made inside a span that no request heard.

/** The asynchronous requests sent to the browser in one task. */
export const PER_TASK = 2;

const SLOT = Symbol("og.pipeline");
const MODULE = Symbol("og.module");
const SKIP = Symbol("og.skip");

type Js = Record<PropertyKey, unknown>;
type Descriptor = Js & { vertex?: Js; fragment?: Js; compute?: Js };
interface Device {
  createShaderModule(desc: unknown): unknown;
  createRenderPipeline(desc: Descriptor): unknown;
  createRenderPipelineAsync(desc: Descriptor): Promise<unknown>;
  createComputePipeline(desc: Descriptor): unknown;
}
interface Slot {
  device: Device;
  desc: Descriptor;
  made: unknown;
  refused: boolean;
}
interface LateModule {
  device: Device;
  desc: unknown;
  made: unknown;
}

let enabled = false;
let span = 0;
/** `drawnWhole` calls open. */
let whole = 0;
let pending = 0;
let skipped = 0;
const queue: Slot[] = [];
let draining = false;
/** The browser's own methods, taken from the prototype at `install`: what the wrappers call. */
let native: { createShaderModule: Device["createShaderModule"]; createRenderPipeline: Device["createRenderPipeline"] } | null = null;

/** The render pipelines made each way since the Worker started. */
export interface PipelineStats {
  /** Asked for with createRenderPipelineAsync (inside a frame span). */
  async: number;
  /** Made with createRenderPipeline outside a frame span (one-shot passes, or the hook off). */
  sync: number;
  /** Asked for asynchronously, then made with createRenderPipeline at a use that could not wait (a bundle, a bind group layout, a refused one). */
  fallback: number;
  /** Frame spans the renderer opened (og-render's hook). */
  spans: number;
}
const counts: PipelineStats = { async: 0, sync: 0, fallback: 0, spans: 0 };

/** A copy of the counts so far. */
export function stats(): PipelineStats {
  return { ...counts };
}

/**
 * Why the frame pipelines did not compile asynchronously although the hook is on, or null: the
 * renderer opened frame spans and `warmed` pipelines were made in them (og-render's warm makes
 * its render pipelines inside one), yet no request reached createRenderPipelineAsync, so they
 * went to the browser some way that bypassed these wrappers.
 */
export function bypassed(s: PipelineStats, warmed: number): string | null {
  if (!enabled || s.spans === 0 || warmed === 0 || s.async > 0) return null;
  return `${warmed} pipelines were made inside the renderer's frame spans, but none through createRenderPipelineAsync: wgpu reached the browser's createRenderPipeline without src/web/pipelines.ts, so frames compile synchronously`;
}

/** Asynchronous pipelines and late modules on or off (the Worker: on the web, once the wasm is in). */
export function enable(on: boolean): void {
  enabled = on;
}

/** The renderer begins (true) or ends (false) making its frame pipelines (og_web_frame_pipelines). */
export function frameSpan(on: boolean): void {
  if (on && span === 0) counts.spans++;
  span = Math.max(0, span + (on ? 1 : -1));
}

/** Whether a render pipeline made now is asked for asynchronously, and a pass may skip a stand-in's draws. */
const framing = () => enabled && span > 0 && whole === 0;

/**
 * Runs `f` with every pipeline it meets made at once and every draw drawn, spans or not: a
 * harness's frame (`frame` calls: the reference pages, og run's one tick and one frame), which is
 * compared with the same frame drawn natively.
 */
export function drawnWhole<T>(f: () => T): T {
  whole++;
  try {
    return f();
  } finally {
    whole--;
  }
}

/** Render pipelines asked for asynchronously and not made yet. */
export function compiling(): number {
  return pending;
}

/** Draws skipped since the last call, for a pipeline that was not made yet. */
export function takeSkipped(): number {
  const n = skipped;
  skipped = 0;
  return n;
}

/** A late module's module, made now when it was not yet. */
function lateMade(late: LateModule): unknown {
  late.made ??= native!.createShaderModule.call(late.device, late.desc);
  return late.made;
}

/** `m`, or the module a late one stands for. */
function moduleOf(m: unknown): unknown {
  const late = (m as Js | null)?.[MODULE] as LateModule | undefined;
  return late ? lateMade(late) : m;
}

/** The stages' late modules made (each once) and set in `desc`. */
function bindModules(desc: Descriptor): Descriptor {
  for (const stage of [desc.vertex, desc.fragment, desc.compute]) if (stage && stage["module"]) stage["module"] = moduleOf(stage["module"]);
  return desc;
}

/** A stand-in's pipeline, made now with `createRenderPipeline` when it was not yet. */
function made(slot: Slot): unknown {
  if (!slot.made) {
    counts.fallback++;
    slot.made = native!.createRenderPipeline.call(slot.device, bindModules(slot.desc));
  }
  return slot.made;
}

const nextTask = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

async function drain(): Promise<void> {
  draining = true;
  try {
    for (;;) {
      await nextTask();
      const batch = queue.splice(0, PER_TASK);
      if (batch.length === 0) return;
      for (const slot of batch) {
        // Made meanwhile by a use that could not wait.
        if (slot.made) {
          pending--;
          continue;
        }
        counts.async++;
        let promise: Promise<unknown>;
        try {
          promise = slot.device.createRenderPipelineAsync(bindModules(slot.desc));
        } catch {
          promise = Promise.reject(new Error("createRenderPipelineAsync threw"));
        }
        promise.then(
          (p) => {
            pending--;
            slot.made ??= p;
          },
          () => {
            pending--;
            slot.refused = true;
          },
        );
      }
    }
  } finally {
    draining = false;
  }
}

/** Gives GPUDevice's prototype `proto` the wrappers above (once; their behaviour follows `enable` and `frameSpan`). */
export function install(proto: Device): void {
  if (native) return;
  const createShaderModule = proto.createShaderModule;
  const createRenderPipeline = proto.createRenderPipeline;
  const createComputePipeline = proto.createComputePipeline;
  native = { createShaderModule, createRenderPipeline };
  proto.createShaderModule = function (this: Device, desc: unknown) {
    if (!enabled) return createShaderModule.call(this, desc);
    const late: LateModule = { device: this, desc, made: undefined };
    return {
      [MODULE]: late,
      label: (desc as { label?: string } | null)?.label ?? "",
      getCompilationInfo: () => (lateMade(late) as { getCompilationInfo(): Promise<unknown> }).getCompilationInfo(),
    };
  };
  proto.createRenderPipeline = function (this: Device, desc: Descriptor) {
    if (!framing()) {
      counts.sync++;
      return createRenderPipeline.call(this, bindModules(desc));
    }
    const slot: Slot = { device: this, desc, made: undefined, refused: false };
    pending++;
    queue.push(slot);
    if (!draining) void drain();
    return {
      [SLOT]: slot,
      label: (desc as { label?: string }).label ?? "",
      getBindGroupLayout: (index: number) => (made(slot) as { getBindGroupLayout(i: number): unknown }).getBindGroupLayout(index),
    };
  };
  proto.createComputePipeline = function (this: Device, desc: Descriptor) {
    return createComputePipeline.call(this, bindModules(desc));
  };
  patchEncoders();
}

let patched = false;
/** The render pass's and bundle's `setPipeline` take a stand-in; a pass skips its draws while it is not made. */
function patchEncoders(): void {
  if (patched) return;
  patched = true;
  const g = globalThis as unknown as { GPURenderPassEncoder?: { prototype: Js }; GPURenderBundleEncoder?: { prototype: Js } };
  const pass = g.GPURenderPassEncoder?.prototype;
  if (pass) {
    const setPipeline = pass["setPipeline"] as (this: Js, p: unknown) => void;
    pass["setPipeline"] = function (this: Js, p: unknown) {
      const slot = (p as Js | null)?.[SLOT] as Slot | undefined;
      // Only a pass encoded in a frame span may skip: one outside it (a shadow map's static
      // casters, a probe's capture, a one-shot bake) keeps what it draws, so it makes it now.
      if (slot && !slot.made && framing() && !slot.refused) {
        skipped++;
        this[SKIP] = true;
        return;
      }
      if (this[SKIP]) this[SKIP] = false;
      setPipeline.call(this, slot ? made(slot) : p);
    };
    for (const name of ["draw", "drawIndexed", "drawIndirect", "drawIndexedIndirect"]) {
      const draw = pass[name] as ((this: Js, a: unknown, b: unknown, c: unknown, d: unknown, e: unknown) => void) | undefined;
      if (!draw) continue;
      pass[name] = function (this: Js, a: unknown, b: unknown, c: unknown, d: unknown, e: unknown) {
        if (this[SKIP]) return;
        draw.call(this, a, b, c, d, e);
      };
    }
  }
  const bundle = g.GPURenderBundleEncoder?.prototype;
  if (bundle) {
    // A bundle is replayed: its pipeline is made now, never skipped.
    const setPipeline = bundle["setPipeline"] as (this: Js, p: unknown) => void;
    bundle["setPipeline"] = function (this: Js, p: unknown) {
      const slot = (p as Js | null)?.[SLOT] as Slot | undefined;
      setPipeline.call(this, slot ? made(slot) : p);
    };
  }
}
