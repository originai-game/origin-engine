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
// - Compute pipelines take the same path, and the renderer's whole frame is
//   a make span (`makeSpan`): every pipeline it makes is asked for
//   asynchronously, with a ticket og-render asks (`ticketCount`, `ready`)
//   before a pass whose output it keeps uses one. Before the first frame
//   (`gate`) a dispatch whose pipeline still compiles skips every later draw
//   and dispatch of that frame's spans, which may read what it writes.
//
// Off, every call goes straight to the browser. Which mode a page runs is
// project.ogt `publish { pipelines }`; `auto` (the default) picks one when the
// Worker meets its adapter (`choose`): async on every GPU but Apple's. A
// browser compiles synchronous pipelines on its GPU process's main thread (a
// Pixel 10 drew no frame for about 23 s behind the gate; First Light on an
// NVIDIA L4 lost its device to Chrome's 15 s GPU watchdog); Apple GPUs stay
// sync, where async made a cold boot on Chrome/Metal 2.5-5x later, until a
// real device is measured (docs/web-export.md §Pipelines).
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
  createComputePipelineAsync(desc: Descriptor): Promise<unknown>;
}
interface Slot {
  device: Device;
  desc: Descriptor;
  compute: boolean;
  /** Its ticket, numbered from 1 (`ticketCount`, `ready`). */
  ticket: number;
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
/** Prewarm (`enable(true, true)`): only the pipelines asked for inside `warming` are asynchronous. */
let prewarmOnly = false;
/** `warming` calls open. */
let warmingDepth = 0;
/** Frame spans opened inside `warming`: under prewarm, the only ones asked to be asynchronous. */
let warmSpans = 0;
let pending = 0;
let skipped = 0;
const queue: Slot[] = [];
let draining = false;
/** `makeSpan` calls open: the renderer's frame, where every pipeline made is asked for asynchronously. */
let making = 0;
/** Before the first frame (`gate`): a dispatch whose compute pipeline still compiles skips the rest of the frame. */
let gating = true;
/** A dispatch was skipped in this frame (under `gating`): every draw and dispatch after it in a span is skipped too. */
let poisoned = false;
let tickets = 0;
/** Stand-ins not made yet, by ticket (`ready`). */
const waiting = new Map<number, Slot>();
/** The browser's own methods, taken from the prototype at `install`: what the wrappers call. */
let native: {
  createShaderModule: Device["createShaderModule"];
  createRenderPipeline: Device["createRenderPipeline"];
  createComputePipeline: Device["createComputePipeline"];
} | null = null;

/** How the page makes its frame pipelines: project.ogt `publish { pipelines }`, or `auto`'s pick. */
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
 * auto, async on every GPU but Apple's (named by the adapter's `info`), which stay sync until a real
 * iOS or macOS device is measured; a virtual machine's numbers do not move the default
 * (docs/web-export.md §Pipelines).
 */
export function choose(asked: PipelineMode | undefined, info: { vendor?: string; architecture?: string } | null | undefined): PipelineChoice {
  if (asked) return { mode: asked, reason: `publish { pipelines ${asked} }` };
  const vendor = (info?.vendor ?? "").toLowerCase();
  const architecture = (info?.architecture ?? "").toLowerCase();
  const named = [vendor, architecture].filter(Boolean).join(" ");
  if (MOBILE_VENDORS.includes(vendor) || MOBILE_ARCHITECTURE.test(architecture)) return { mode: "async", reason: `auto: a mobile GPU (${named})` };
  if (vendor === "apple") return { mode: "sync", reason: `auto: an Apple GPU (${named}), sync until iOS is measured` };
  // A desktop GPU with a cold driver cache compiles a large scene's pipelines for seconds each
  // (NVIDIA L4/Vulkan: 3 to 4.8 s), and synchronous ones in one GPU-process task past Chrome's
  // 15 s watchdog lose the device (docs/web-export.md §Pipelines).
  if (named) return { mode: "async", reason: `auto: not an Apple GPU (${named})` };
  return { mode: "async", reason: "auto: no adapter info" };
}

let chosen: PipelineChoice | null = null;

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

/** A copy of the counts so far, with the mode chosen and why (once `enable` ran). */
export function stats(): PipelineStats & Partial<PipelineChoice> {
  return { ...counts, ...(chosen ?? {}) };
}

/**
 * Why the frame pipelines did not compile asynchronously although the hook is on, or null: the
 * renderer opened frame spans and `warmed` pipelines were made in them (og-render's warm makes
 * its render pipelines inside one), yet no request reached createRenderPipelineAsync, so they
 * went to the browser some way that bypassed these wrappers.
 */
export function bypassed(s: PipelineStats, warmed: number): string | null {
  if (!enabled || (prewarmOnly ? warmSpans : s.spans) === 0 || warmed === 0 || s.async > 0) return null;
  return `${warmed} pipelines were made inside the renderer's frame spans, but none through createRenderPipelineAsync: wgpu reached the browser's createRenderPipeline without src/web/pipelines.ts, so frames compile synchronously`;
}

/**
 * Asynchronous pipelines and late modules on or off (the Worker: on the web, once the wasm is in, as the
 * build's `publish { pipelines }` says). With `prewarm`, only the renderer's warm-up behind the gate
 * (`warming`) asks for createRenderPipelineAsync: every frame makes what it draws at once and skips nothing.
 */
export function enable(on: boolean, prewarm = false): void {
  enabled = on;
  prewarmOnly = on && prewarm;
  warmSpans = 0;
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

/** The renderer begins (true) or ends (false) making its frame pipelines (og_web_frame_pipelines). */
export function frameSpan(on: boolean): void {
  // The renderer's hook is installed in every mode (auto picks one only at the adapter); spans count while on.
  if (on && span === 0 && enabled) {
    counts.spans++;
    if (warmingDepth > 0) warmSpans++;
  }
  span = Math.max(0, span + (on ? 1 : -1));
}

/** Whether a pass may skip a stand-in's draws or dispatches (and a pipeline made now is asked for asynchronously). */
const framing = () => enabled && span > 0 && whole === 0 && (!prewarmOnly || warmingDepth > 0);
/** Whether a pipeline made now is asked for asynchronously: in a frame span, or anywhere in the renderer's frame (`makeSpan`). */
const asking = () => framing() || (enabled && making > 0 && whole === 0 && !prewarmOnly);

/**
 * The renderer begins (true) or ends (false) a frame (og_web_make_pipelines): every render and
 * compute pipeline made in between is asked for asynchronously, but only a pass in a frame span
 * skips a stand-in's work; one outside makes it then. og-render asks `ready` before a pass whose
 * output it keeps uses one, and keeps that work for a later frame (og-render gpu/ready.rs).
 */
export function makeSpan(on: boolean): void {
  making = Math.max(0, making + (on ? 1 : -1));
}

/** The tickets handed out so far: each a pipeline asked for asynchronously, numbered from 1. */
export function ticketCount(): number {
  return tickets;
}

/** Whether any pipeline asked for asynchronously still compiles (og-render keeps its shadow layers only once none does). */
export function anyCompiling(): boolean {
  return pending > 0;
}

/** Whether the pipeline with `ticket` is made (or refused, which its use reports): og-render asks before using it. */
export function ready(ticket: number): boolean {
  const slot = waiting.get(ticket);
  return !slot || slot.refused || !!slot.made;
}

/**
 * Before the first frame (on, the default) a dispatch in a frame span whose compute pipeline still
 * compiles skips the rest of that frame's draws and dispatches in spans, so nothing draws from the
 * buffer it did not fill, and the gate draws the frame again. Once the first frame is drawn (off),
 * such a pipeline is made at its dispatch instead: a played frame never draws from it unfilled.
 */
export function gate(on: boolean): void {
  gating = on;
  if (!on) poisoned = false;
}

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
  // The gate asks once a frame: a frame's skip of the rest ends with it.
  poisoned = false;
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

/** A stand-in's pipeline, made now with `createRenderPipeline` (or `createComputePipeline`) when it was not yet. */
function made(slot: Slot): unknown {
  if (!slot.made) {
    counts.fallback++;
    const make = slot.compute ? native!.createComputePipeline : native!.createRenderPipeline;
    slot.made = make.call(slot.device, bindModules(slot.desc));
    waiting.delete(slot.ticket);
  }
  return slot.made;
}

/** A stand-in for `desc`, asked for asynchronously in a later task. */
function standIn(device: Device, desc: Descriptor, compute: boolean): Js {
  const slot: Slot = { device, desc, compute, ticket: ++tickets, made: undefined, refused: false };
  waiting.set(slot.ticket, slot);
  pending++;
  queue.push(slot);
  if (!draining) void drain();
  return {
    [SLOT]: slot,
    label: (desc as { label?: string }).label ?? "",
    getBindGroupLayout: (index: number) => (made(slot) as { getBindGroupLayout(i: number): unknown }).getBindGroupLayout(index),
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
        // Made meanwhile by a use that could not wait.
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
            slot.made ??= p;
            waiting.delete(slot.ticket);
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
    if (!asking()) {
      counts.sync++;
      return createRenderPipeline.call(this, bindModules(desc));
    }
    return standIn(this, desc, false);
  };
  // Compute pipelines too: a cold driver compiles one for seconds as well (fork s60's gap).
  proto.createComputePipeline = function (this: Device, desc: Descriptor) {
    if (!asking() || typeof this.createComputePipelineAsync !== "function") {
      if (enabled) counts.sync++;
      return createComputePipeline.call(this, bindModules(desc));
    }
    return standIn(this, desc, true);
  };
  patchEncoders();
}

let patched = false;
/** A skipping pass's draws: nothing. */
const skipDraw = () => {};
/** The render pass's and bundle's `setPipeline` take a stand-in; a pass skips its draws while it is not made. */
function patchEncoders(): void {
  if (patched) return;
  patched = true;
  const g = globalThis as unknown as { GPURenderPassEncoder?: { prototype: Js }; GPURenderBundleEncoder?: { prototype: Js }; GPUComputePassEncoder?: { prototype: Js } };
  const pass = g.GPURenderPassEncoder?.prototype;
  if (pass) {
    const setPipeline = pass["setPipeline"] as (this: Js, p: unknown) => void;
    // The draw methods stay the browser's own (s105): a pass that skips is given no-op draws of
    // its own until its next made pipeline gives them back, so a pass that skips nothing (every
    // pass once the pipelines are made, and every pass of a sync page) draws through no wrapper
    // (one on the prototype cost every draw of every frame: 9k a frame, 0.2 ms, in a large scene).
    const draws = ["draw", "drawIndexed", "drawIndirect", "drawIndexedIndirect"].filter((name) => pass[name]);
    const own = draws.map((name) => pass[name]);
    pass["setPipeline"] = function (this: Js, p: unknown) {
      const slot = (p as Js | null)?.[SLOT] as Slot | undefined;
      // Only a pass encoded in a frame span may skip: one outside it (a shadow map's static
      // casters, a probe's capture, a one-shot bake) keeps what it draws, so it makes it now.
      // After a skipped dispatch (`gate`) the rest of the frame's spans draw nothing.
      if ((slot && !slot.made && framing() && !slot.refused) || (poisoned && framing())) {
        skipped++;
        if (!this[SKIP]) {
          this[SKIP] = true;
          for (const name of draws) this[name] = skipDraw;
        }
        return;
      }
      if (this[SKIP]) {
        this[SKIP] = false;
        draws.forEach((name, i) => (this[name] = own[i]));
      }
      setPipeline.call(this, slot ? made(slot) : p);
    };
  }
  const compute = g.GPUComputePassEncoder?.prototype;
  if (compute) {
    const setPipeline = compute["setPipeline"] as (this: Js, p: unknown) => void;
    const dispatches = ["dispatchWorkgroups", "dispatchWorkgroupsIndirect"].filter((name) => compute[name]);
    const own = dispatches.map((name) => compute[name]);
    compute["setPipeline"] = function (this: Js, p: unknown) {
      const slot = (p as Js | null)?.[SLOT] as Slot | undefined;
      // Before the first frame, in a span: the dispatch is skipped, and with it everything after
      // it in the frame's spans, which may read what it writes (`gate`). Otherwise made now.
      const skip = gating && framing() && (poisoned || (!!slot && !slot.made && !slot.refused));
      if (skip) {
        skipped++;
        poisoned = true;
        if (!this[SKIP]) {
          this[SKIP] = true;
          for (const name of dispatches) this[name] = skipDraw;
        }
        return;
      }
      if (this[SKIP]) {
        this[SKIP] = false;
        dispatches.forEach((name, i) => (this[name] = own[i]));
      }
      setPipeline.call(this, slot ? made(slot) : p);
    };
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
