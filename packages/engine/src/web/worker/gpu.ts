// The renderer's GPU device as the engine Worker meets it, and the pacing of
// the pipelines it warms.
//
// Fork s61: a lost WebGPU device and its first validation errors are said in the console (the
// renderer is made inside the wasm, so its device is met here, where the Worker asks for it).
// Fork s60: GPUDevice makes the renderer's frame pipelines asynchronously (pipelines.ts), installed
// on the prototype before the wasm's glue loads.
// A GPUInternalError popped from an error scope reaches wgpu as one it can read (gpu-errors.ts).

import * as pipelines from "../pipelines.ts";
import * as gpuErrors from "../gpu-errors.ts";

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
  if (device) pipelines.install(device as Parameters<typeof pipelines.install>[0]);
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
