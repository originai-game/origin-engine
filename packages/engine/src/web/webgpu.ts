// Whether this browser can run the engine at all: WebGPU (docs/design.md §8).

/**
 * Why this browser cannot run the engine, or null when it can. The engine
 * draws on WebGPU only, with no fallback (docs/design.md §8): a page with
 * no `navigator.gpu` (an older browser, or an insecure context: WebGPU
 * needs https or localhost) or no adapter is not supported, and the player
 * shell says so instead of starting the engine. A device the adapter then
 * refuses is the engine Worker's start failure (`og_web_attach_canvas`).
 */
export async function webGpuMissing(): Promise<string | null> {
  const gpu = (globalThis.navigator as { gpu?: { requestAdapter(): Promise<unknown> } } | undefined)?.gpu;
  if (!gpu) {
    return globalThis.isSecureContext === false
      ? "this page is not a secure context (WebGPU needs https or localhost), so navigator.gpu is missing"
      : "navigator.gpu is missing";
  }
  try {
    return (await gpu.requestAdapter()) ? null : "the browser found no WebGPU adapter";
  } catch (e) {
    return `the browser's WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`;
  }
}
