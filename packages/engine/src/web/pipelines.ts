// The web host's pipeline compiler: what og-render's pipeline cache (crates/og-render/src/gpu/
// deferred.rs) asks to compile a pipeline off the browser's GPU-process main task.
//
// `createRenderPipeline` compiles inside one task of Chrome's GPU process: a frame that makes
// dozens of pipelines (a large game's first frame) holds that process, and with it the page's
// frames and the WebGPU queue, for as long as they all take. `createRenderPipelineAsync` compiles
// on Dawn's worker threads. wgpu's WebGPU backend only calls the synchronous one, as a method of
// the device, so the Worker replaces the methods on GPUDevice.prototype when it starts, before the
// wasm's glue loads (the engine never patches wgpu, AGENTS.md).
//
// The cache owns readiness: it makes each of its entries between `asking(true)` and
// `asking(false)`, and every render or compute pipeline made in between is asked for with the
// asynchronous call and handed back at once as a stand-in, numbered from 1. The cache asks
// `compiled(n)` before it draws with one, and `finish(n)` when a use waits for it (a frame that
// waits, a capture), which makes it now with the synchronous call. So `setPipeline` here only puts
// the browser's pipeline in the stand-in's place; nothing is skipped in this file. A bundle or a
// bind group layout asked of a stand-in makes it now too.
//
// While on, a WGSL module is created at the first pipeline that uses it, so a game's dozens of
// material modules are compiled (Tint) in the pipeline requests' tasks.
//
// Which mode a page runs is project.ogt `publish { pipelines }`; `auto` (the default) picks one
// when the Worker meets its adapter (`choose`): async on every GPU but Apple's (docs/web-export.md
// §Pipelines). Under `prewarm` only the renderer's warm-up behind the gate (`warming`) is
// asynchronous: every frame makes what it draws at once.
//
// A wgpu that reached the browser's createRenderPipeline some other way would lose all this
// silently. So the pipelines made each way are counted (`stats`), the Worker sends them with the
// first frame (BootPhases `pipelines`, the probe's `boot`), and `bypassed` names the loss.

/** The asynchronous requests sent to the browser in one task. */
export const PER_TASK = 2;

const SLOT = Symbol("og.pipeline");
const MODULE = Symbol("og.module");

type Js = Record<PropertyKey, unknown>;
type Descriptor = Js & { vertex?: Js; fragment?: Js; compute?: Js };
interface Device {
  createShaderModule(desc: unknown): unknown;
  createRenderPipeline(desc: Descriptor): unknown;
  createRenderPipelineAsync(desc: Descriptor): Promise<unknown>;
  createComputePipeline(desc: Descriptor): unknown;
  createComputePipelineAsync(desc: Descriptor): Promise<unknown>;
}
interface Slot {
  device: Device;
  desc: Descriptor;
  compute: boolean;
  /** Its number, from 1 (`count`, `compiled`, `finish`). */
  n: number;
  made: unknown;
  refused: boolean;
  /** Its entry in `compiles`. */
  compile: Compile;
}

/**
 * A pipeline asked for asynchronously: its label, when it was asked for and when it was made
 * (ms on the Worker's clock; null while it compiles), and how. perf's report lists them
 * (worker/calls.ts), so a UI or overlay pipeline that finished after Play shows by name.
 */
export interface Compile {
  label: string;
  asked: number;
  made: number | null;
  /** async: the browser's promise; waited: made at a use that waited for it; refused. */
  how: "async" | "waited" | "refused" | null;
}
const compiles: Compile[] = [];

/** Every pipeline asked for asynchronously so far, in the order asked. */
export function compiled(): readonly Compile[] {
  return compiles;
}
interface LateModule {
  device: Device;
  desc: unknown;
  made: unknown;
}

let enabled = false;
/** Prewarm (`enable(true, true)`): only the pipelines asked for inside `warming` are asynchronous. */
let prewarmOnly = false;
/** `warming` calls open. */
let warmingDepth = 0;
/** `asking` calls open: the cache making an entry. */
let askingDepth = 0;
let pending = 0;
const queue: Slot[] = [];
let draining = false;
/** Every stand-in by number (index n - 1). */
const slots: Slot[] = [];
/** The browser's own methods, taken from the prototype at `install`: what the wrappers call. */
let native: {
  createShaderModule: Device["createShaderModule"];
  createRenderPipeline: Device["createRenderPipeline"];
  createComputePipeline: Device["createComputePipeline"];
} | null = null;

/** How the page makes its pipelines: project.ogt `publish { pipelines }`, or `auto`'s pick. */
export type PipelineMode = "sync" | "async" | "prewarm";

/** The mode a page runs and why: what its build asked, or what `auto` read of the adapter. */
export interface PipelineChoice {
  mode: PipelineMode;
  reason: string;
}

/** The adapter's vendors whose GPUs are mobile ones (GPUAdapterInfo.vendor: PowerVR, Mali, Adreno). */
const MOBILE_VENDORS = ["img-tec", "arm", "qualcomm"];
/** The same GPUs named by GPUAdapterInfo.architecture, for a browser that reports it without the vendor. */
const MOBILE_ARCHITECTURE = /powervr|rogue|mali|valhall|bifrost|adreno/;

/**
 * The mode a page runs: `asked` (its build's `publish { pipelines }`, absent for auto) as is, or for
 * auto, async on every GPU but Apple's (named by the adapter's `info`).
 * First Light on a physical M4 Pro showed no consistent first-frame advantage for async;
 * Apple GPUs keep sync, with iOS still unmeasured (docs/web-export.md §Pipelines).
 */
export function choose(asked: PipelineMode | undefined, info: { vendor?: string; architecture?: string } | null | undefined): PipelineChoice {
  if (asked) return { mode: asked, reason: `publish { pipelines ${asked} }` };
  const vendor = (info?.vendor ?? "").toLowerCase();
  const architecture = (info?.architecture ?? "").toLowerCase();
  const named = [vendor, architecture].filter(Boolean).join(" ");
  if (MOBILE_VENDORS.includes(vendor) || MOBILE_ARCHITECTURE.test(architecture)) return { mode: "async", reason: `auto: a mobile GPU (${named})` };
  if (vendor === "apple") return { mode: "sync", reason: `auto: an Apple GPU (${named}), no consistent async advantage on M4 Pro` };
  // A desktop GPU with a cold driver cache compiles a large scene's pipelines for seconds each
  // (NVIDIA L4/Vulkan: 3 to 4.8 s), and synchronous ones in one GPU-process task past Chrome's
  // 15 s watchdog lose the device (docs/web-export.md §Pipelines).
  if (named) return { mode: "async", reason: `auto: not an Apple GPU (${named})` };
  return { mode: "async", reason: "auto: no adapter info" };
}

let chosen: PipelineChoice | null = null;

/** The render and compute pipelines made each way since the Worker started. */
export interface PipelineStats {
  /** Asked for with createRenderPipelineAsync or createComputePipelineAsync (inside the cache's `asking`). */
  async: number;
  /** Made with the synchronous call outside `asking` (the mode off, or a pipeline the cache does not hold). */
  sync: number;
  /** Asked for asynchronously, then made with the synchronous call at a use that waited (a frame that waits, a bundle, a bind group layout, a refused one). */
  waited: number;
  /** Entries the cache made with the compiler on (its `asking` calls). */
  asks: number;
}
const counts: PipelineStats = { async: 0, sync: 0, waited: 0, asks: 0 };

/** A copy of the counts so far, with the mode chosen and why (once `use` ran). */
export function stats(): PipelineStats & Partial<PipelineChoice> {
  return { ...counts, ...(chosen ?? {}) };
}

/**
 * Why the pipelines did not compile asynchronously although the compiler is on, or null: the cache
 * made entries with it on and `warmed` pipelines were warmed, yet no request reached the
 * asynchronous calls, so they went to the browser some way that bypassed these wrappers.
 */
export function bypassed(s: PipelineStats, warmed: number): string | null {
  if (!enabled || s.asks === 0 || warmed === 0 || s.async > 0) return null;
  return `${warmed} pipelines were warmed with the pipeline compiler on, but none through createRenderPipelineAsync: wgpu reached the browser's createRenderPipeline without src/web/pipelines.ts, so pipelines compile synchronously`;
}

/** Asynchronous pipelines and late modules on or off; with `prewarm`, only inside `warming`. */
export function enable(on: boolean, prewarm = false): void {
  enabled = on;
  prewarmOnly = on && prewarm;
}

/** Runs the page as `choice` says (the Worker, when it meets the adapter), and reports it in `stats`. */
export function use(choice: PipelineChoice): void {
  chosen = choice;
  enable(choice.mode !== "sync", choice.mode === "prewarm");
}

/** Runs `f`, the renderer's warm-up of a pipeline no frame drew yet (og_web_warm): asynchronous under prewarm too. */
export function warming<T>(f: () => T): T {
  warmingDepth++;
  try {
    return f();
  } finally {
    warmingDepth--;
  }
}

/** Whether a pipeline made now is asked for asynchronously. */
const asked = () => enabled && askingDepth > 0 && (!prewarmOnly || warmingDepth > 0);

/** The cache begins (true) or ends (false) making an entry (og_web_pipeline_compiler `asking`). */
export function asking(on: boolean): void {
  if (on && askingDepth === 0 && enabled) counts.asks++;
  askingDepth = Math.max(0, askingDepth + (on ? 1 : -1));
}

/** The pipelines numbered so far: each asked for asynchronously. */
export function count(): number {
  return slots.length;
}

/** How many of the numbered pipelines are made or refused (only goes up). */
export function made(): number {
  return slots.length - pending;
}

/** Whether pipeline `n` is made (or refused, which its use reports): the cache asks before it draws with it. */
export function isCompiled(n: number): boolean {
  const slot = slots[n - 1];
  return !slot || slot.refused || !!slot.made;
}

/** Makes pipeline `n` now when it is not yet: a use that waits for it. */
export function finish(n: number): void {
  const slot = slots[n - 1];
  if (slot && !slot.refused) madeNow(slot);
}

/** The pipelines asked for so far and how many of them are made (or refused): both only go up (the splash's "41/96"). */
export function counted(): { requested: number; ready: number } {
  return { requested: slots.length, ready: made() };
}

/** Pipelines asked for asynchronously and not made yet. */
export function compiling(): number {
  return pending;
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

/** A stand-in's pipeline, made now with the synchronous call when it was not yet. */
function madeNow(slot: Slot): unknown {
  if (!slot.made) {
    counts.waited++;
    const make = slot.compute ? native!.createComputePipeline : native!.createRenderPipeline;
    slot.made = make.call(slot.device, bindModules(slot.desc));
    slot.compile.made = performance.now();
    slot.compile.how = "waited";
  }
  return slot.made;
}

/** A stand-in for `desc`, asked for asynchronously in a later task. */
function standIn(device: Device, desc: Descriptor, compute: boolean): Js {
  const compile: Compile = { label: (desc as { label?: string }).label ?? "", asked: performance.now(), made: null, how: null };
  compiles.push(compile);
  const slot: Slot = { device, desc, compute, n: slots.length + 1, made: undefined, refused: false, compile };
  slots.push(slot);
  pending++;
  queue.push(slot);
  if (!draining) void drain();
  return {
    [SLOT]: slot,
    label: compile.label,
    getBindGroupLayout: (index: number) => (madeNow(slot) as { getBindGroupLayout(i: number): unknown }).getBindGroupLayout(index),
  };
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
        // Made meanwhile by a use that waited.
        if (slot.made) {
          pending--;
          continue;
        }
        counts.async++;
        let promise: Promise<unknown>;
        try {
          const desc = bindModules(slot.desc);
          promise = slot.compute ? slot.device.createComputePipelineAsync(desc) : slot.device.createRenderPipelineAsync(desc);
        } catch {
          promise = Promise.reject(new Error("the asynchronous pipeline call threw"));
        }
        promise.then(
          (p) => {
            pending--;
            if (!slot.made) {
              slot.made = p;
              slot.compile.made = performance.now();
              slot.compile.how = "async";
            }
          },
          () => {
            pending--;
            // Made with the synchronous call at its next use, which reports the error.
            if (!slot.made) {
              slot.refused = true;
              slot.compile.made = performance.now();
              slot.compile.how = "refused";
            }
          },
        );
      }
    }
  } finally {
    draining = false;
  }
}

/** Gives GPUDevice's prototype `proto` the wrappers above (once; their behaviour follows `enable` and `asking`). */
export function install(proto: Device): void {
  if (native) return;
  const createShaderModule = proto.createShaderModule;
  const createRenderPipeline = proto.createRenderPipeline;
  const createComputePipeline = proto.createComputePipeline;
  native = { createShaderModule, createRenderPipeline, createComputePipeline };
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
    if (!asked()) {
      counts.sync++;
      return createRenderPipeline.call(this, bindModules(desc));
    }
    return standIn(this, desc, false);
  };
  proto.createComputePipeline = function (this: Device, desc: Descriptor) {
    if (!asked() || typeof this.createComputePipelineAsync !== "function") {
      counts.sync++;
      return createComputePipeline.call(this, bindModules(desc));
    }
    return standIn(this, desc, true);
  };
  patchEncoders();
}

let patched = false;
/** The encoders' `setPipeline` take a stand-in: its browser pipeline, made now if it is not yet. */
function patchEncoders(): void {
  if (patched) return;
  patched = true;
  const g = globalThis as unknown as { GPURenderPassEncoder?: { prototype: Js }; GPURenderBundleEncoder?: { prototype: Js }; GPUComputePassEncoder?: { prototype: Js } };
  for (const proto of [g.GPURenderPassEncoder?.prototype, g.GPUComputePassEncoder?.prototype, g.GPURenderBundleEncoder?.prototype]) {
    if (!proto) continue;
    const setPipeline = proto["setPipeline"] as (this: Js, p: unknown) => void;
    proto["setPipeline"] = function (this: Js, p: unknown) {
      const slot = (p as Js | null)?.[SLOT] as Slot | undefined;
      setPipeline.call(this, slot ? madeNow(slot) : p);
    };
  }
}
