// Render pipelines compiled off the browser's GPU-process main task, on the
// published wgpu (fork s60 and s64, which patched wgpu's WebGPU backend for
// this; the engine never patches wgpu, AGENTS.md).
//
// `createRenderPipeline` compiles inside one task of Chrome's GPU process: a
// frame that makes dozens of pipelines (a large game's first frame) holds
// that process, and with it the page's frames and the WebGPU queue, for as
// long as they all take. `createRenderPipelineAsync` compiles on Dawn's
// worker threads. wgpu's WebGPU backend only calls the synchronous one, and
// it calls it as a method of the device (wasm-bindgen's structural calls),
// so the Worker gives each device it hands the wasm its own methods:
//
// - While the renderer makes its frame pipelines (og-render's
//   `set_frame_pipelines_hook`, `span`), `createRenderPipeline` asks for
//   `createRenderPipelineAsync` and hands back a stand-in at once, which the
//   browser's pipeline fills in when the promise resolves. The requests are
//   sent `PER_TASK` a task, so a frame's dozens are not one GPU-process task
//   again.
// - A render pass's `setPipeline` of a stand-in not filled in yet skips that
//   pipeline's draws (counted) instead of waiting; a render bundle, a bind
//   group layout asked of it, or one the browser refused makes it then, with
//   `createRenderPipeline` (which reports a refused one's error as before).
// - While on (`enable`), a WGSL module is created at the first pipeline that
//   uses it (s64), so a game's dozens of material modules are compiled
//   (Tint) in the pipeline requests' tasks, not all in the task that
//   registered them; any other use makes it then.
//
// Off (the default), every call goes straight to the browser.

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
let pending = 0;
let skipped = 0;
const queue: Slot[] = [];
let draining = false;
/** The browser's own methods, by device: what the wrappers call. */
const own = new WeakMap<object, { createShaderModule: Device["createShaderModule"]; createRenderPipeline: Device["createRenderPipeline"] }>();

/** Asynchronous pipelines and late modules on or off (the Worker: on the web, once the wasm is in). */
export function enable(on: boolean): void {
  enabled = on;
}

/** The renderer begins (true) or ends (false) making its frame pipelines (og_web_frame_pipelines). */
export function frameSpan(on: boolean): void {
  span = Math.max(0, span + (on ? 1 : -1));
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
  late.made ??= own.get(late.device as object)!.createShaderModule.call(late.device, late.desc);
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
  slot.made ??= own.get(slot.device as object)!.createRenderPipeline.call(slot.device, bindModules(slot.desc));
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

/** Gives `device` the wrappers above (once; their behaviour follows `enable` and `frameSpan`). */
export function install(device: Device): void {
  if (own.has(device as object)) return;
  const createShaderModule = device.createShaderModule;
  const createRenderPipeline = device.createRenderPipeline;
  const createComputePipeline = device.createComputePipeline;
  own.set(device as object, { createShaderModule, createRenderPipeline });
  device.createShaderModule = function (this: Device, desc: unknown) {
    if (!enabled) return createShaderModule.call(this, desc);
    const late: LateModule = { device: this, desc, made: undefined };
    return {
      [MODULE]: late,
      label: (desc as { label?: string } | null)?.label ?? "",
      getCompilationInfo: () => (lateMade(late) as { getCompilationInfo(): Promise<unknown> }).getCompilationInfo(),
    };
  };
  device.createRenderPipeline = function (this: Device, desc: Descriptor) {
    if (!(enabled && span > 0)) return createRenderPipeline.call(this, bindModules(desc));
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
  device.createComputePipeline = function (this: Device, desc: Descriptor) {
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
      if (slot && !slot.made && enabled && !slot.refused) {
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
