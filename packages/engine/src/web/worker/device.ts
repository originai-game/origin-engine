// The renderer's GPU device as the engine Worker meets it, how its pipelines are made, and the
// pacing of the pipelines it warms.
//
// Fork s61: a lost WebGPU device and its first validation errors are said in the console (the
// renderer is made inside the wasm, so its device is met here, where the Worker asks for it).
// A GPUInternalError popped from an error scope reaches wgpu as one it can read (gpu-errors.ts).
//
// The renderer's pipeline cache (crates/og-render/src/gpu/deferred.rs) makes its frame pipelines
// with wgpu's `create_*_pipeline_async` (the browser's createRenderPipelineAsync, which compiles
// off Chrome's GPU-process main task); the Worker only picks the mode (`choose`) and reads the
// counts (`pipelineStats`).

import * as gpuErrors from "../gpu-errors.ts";
import type { Glue } from "./run.ts";

type Adapter = { info?: { vendor?: string; architecture?: string } };

/** Called with the adapter as the renderer asks it for its device (before any pipeline is made): auto's pick. */
let onAdapter: ((adapter: Adapter) => void) | null = null;

/** What hears the adapter when the renderer asks it for its device (start.ts: auto's pick). */
export function onDeviceAdapter(heard: (adapter: Adapter) => void): void {
  onAdapter = heard;
}

{
  const g = globalThis as Partial<gpuErrors.ErrorClasses> & { GPUDevice?: { prototype: unknown } };
  const device = g.GPUDevice?.prototype;
  if (device && g.GPUInternalError && g.GPUValidationError)
    gpuErrors.install(device as Parameters<typeof gpuErrors.install>[0], { GPUInternalError: g.GPUInternalError, GPUValidationError: g.GPUValidationError });
  const proto = (globalThis as { GPUAdapter?: { prototype: { requestDevice(...a: unknown[]): Promise<GPUDeviceLike> } } }).GPUAdapter?.prototype;
  type GPUDeviceLike = { lost: Promise<{ reason: string; message: string }>; queue: GpuQueue; addEventListener(t: string, f: (e: { error?: { message?: string } }) => void): void };
  if (proto) {
    const requestDevice = proto.requestDevice;
    proto.requestDevice = async function (this: unknown, ...a: unknown[]) {
      onAdapter?.(this as Adapter);
      const device = await requestDevice.apply(this, a);
      gpuQueue = device.queue;
      void device.lost.then((l) => console.error(`[og] WebGPU device lost: reason=${l.reason} message=${l.message}`));
      let errors = 0;
      device.addEventListener("uncapturederror", (e) => {
        if (errors++ < 20) console.error(`[og] WebGPU error: ${e.error?.message}`);
      });
      return device;
    };
  }
}

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

/** Runs the page as `choice` says (the Worker, when it meets the adapter), and reports it in `pipelineStats`. */
export function usePipelines(glue: Glue, choice: PipelineChoice): void {
  chosen = choice;
  glue.og_web_pipeline_mode?.(choice.mode);
}

/** The render and compute pipelines the renderer's cache made each way (og_web_pipeline_stats). */
export interface PipelineStats {
  /** Asked for with createRenderPipelineAsync or createComputePipelineAsync. */
  async: number;
  /** Made with the synchronous call in an entry made at once. */
  sync: number;
  /** Asked for asynchronously, then made with the synchronous call at a use that waited (or refused by the browser). */
  waited: number;
  /** Entries the cache made asking asynchronously. */
  asks: number;
  /** Of `async`, those compiled or refused (`async - settled` compile now). */
  settled: number;
}

/** The counts so far, with the mode chosen and why (once `usePipelines` ran). */
export function pipelineStats(glue: Glue | null | undefined): PipelineStats & Partial<PipelineChoice> {
  const raw = glue?.og_web_pipeline_stats?.();
  const s = raw ? (JSON.parse(raw) as { requested: number; settled: number; sync: number; waited: number; asks: number }) : { requested: 0, settled: 0, sync: 0, waited: 0, asks: 0 };
  return { async: s.requested, settled: s.settled, sync: s.sync, waited: s.waited, asks: s.asks, ...(chosen ?? {}) };
}

/** Pipelines asked for asynchronously and not compiled yet. */
export function compiling(glue: Glue | null | undefined): number {
  const s = pipelineStats(glue);
  return s.async - s.settled;
}

type GpuQueue = { submit(buffers: unknown[]): void; onSubmittedWorkDone(): Promise<unknown> };
/** The renderer's device queue (set when the wasm asks for its device). */
let gpuQueue: GpuQueue | null = null;
/**
 * Warmed pipelines the GPU process has not finished yet. A synchronous createRenderPipeline returns at once in
 * the Worker; Chrome sends it to the GPU process only when something flushes its wire (a submit), and the GPU
 * process compiles it there, in order, on the task that also runs every frame's and readback's commands. Behind
 * the gate nothing submitted, so the whole warm-up reached the GPU process with the tap's first frame and froze
 * it (11 s on a Pixel 10, warm cache; 15 s for a probe capture on a cold Metal cache). So each warm call is
 * flushed (an empty submit) and the next one waits until the queue's work so far, its compile included, is done:
 * the warm-up compiles behind the gate, and a tap or a capture waits for at most WARM_AHEAD compiles.
 */
let warmsInFlight = 0;
/**
 * The warm calls whose compile may be in flight at once. One was the bound until Firefox: Gecko sends a
 * pipeline to its GPU process at once (no wire to flush), but answers onSubmittedWorkDone about 100 ms
 * after an empty submit even on an idle device, with outliers of seconds, so a warm-up paced one a
 * time made 6 pipelines in 10 s behind the gate and the tap met the rest (its first frame 1.2 s,
 * studio-win, Firefox 159). Three keep Firefox's warm-up going while a tap or a capture in Chrome
 * still waits for at most three compiles.
 */
const WARM_AHEAD = 3;
/** Whether the warm-up waits for a compile in flight before its next call. */
export const warmBusy = () => warmsInFlight >= WARM_AHEAD;
/** Runs one warm call; until its compile is done in the GPU process, it counts in `warmsInFlight`. */
export function warmPaced(warm: () => boolean): boolean {
  const more = warm();
  if (more && gpuQueue) {
    warmsInFlight++;
    const done = () => {
      warmsInFlight--;
    };
    gpuQueue.submit([]);
    gpuQueue.onSubmittedWorkDone().then(done, done);
  }
  return more;
}

/**
 * Fork s75: the warmed pipelines compiling at once (s60/s70 warm). Chrome compiles
 * `createRenderPipelineAsync` off its GPU-process main thread, so two at a time left
 * it idle: the warm tail after the 1072-cell boot was 4-8 s at two.
 */
export const WARM_COMPILING = 6;
