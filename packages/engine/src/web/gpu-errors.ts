// Error scopes the published wgpu can read (installed on GPUDevice.prototype
// by the Worker, before the wasm's glue loads, as pipelines.ts is).
//
// wgpu 30's WebGPU backend decodes only a GPUValidationError or a
// GPUOutOfMemoryError from popErrorScope and panics ("Unexpected error") on
// a GPUInternalError (gfx-rs/wgpu#9914; fixed upstream by #9919, in no
// release yet). An internal error is how a driver refuses a valid pipeline
// it cannot compile: Chrome on the Pixel 10's PowerVR refuses the
// visibility buffer's resolve so (og-render meshlet_gpu/vis.rs,
// the former test sample and the renderer's `internal`
// scope around it then panicked the wasm instead of turning the visibility
// buffer off. wgpu is used as published (AGENTS.md), so the Worker hands such
// an error to wgpu as a GPUValidationError carrying its message, which the
// renderer treats alike (the pipeline refused). Drop this once a wgpu
// release decodes GPUInternalError.

/** The prefix a GPUInternalError's message is handed to wgpu with. */
export const INTERNAL = "internal error: ";

export interface ErrorClasses {
  readonly GPUInternalError: abstract new (...a: never[]) => { readonly message: string };
  readonly GPUValidationError: new (message: string) => object;
}

export function install(proto: { popErrorScope(): Promise<unknown> }, classes: ErrorClasses): void {
  const pop = proto.popErrorScope;
  proto.popErrorScope = async function (this: unknown) {
    const error = await pop.call(this);
    return error instanceof classes.GPUInternalError ? new classes.GPUValidationError(INTERNAL + error.message) : error;
  };
}
