// Whether this browser can run the engine at all: WebGPU (docs/design.md §8).
//
// The page asks only `webGpuAbsent`; the engine Worker asks for the adapter
// (`webGpuMissing`) before it fetches the wasm. Chrome's first
// requestAdapter blocks the thread that calls it while the GPU process
// starts WebGPU (seconds on a cold or busy GPU process), and on the page's
// main thread that froze the boot splash: its genesis, the progress, input.

/** Why this page has no WebGPU at all (no `navigator.gpu`), or null: synchronous, it asks for no adapter. */
export function webGpuAbsent(): string | null {
  if ((globalThis.navigator as { gpu?: unknown } | undefined)?.gpu) return null;
  return globalThis.isSecureContext === false ? "this page is not a secure context (WebGPU needs https or localhost), so navigator.gpu is missing" : "navigator.gpu is missing";
}

/**
 * Why this browser cannot run the engine, or null when it can. The engine
 * draws on WebGPU only, with no fallback (docs/design.md §8): a page with
 * no `navigator.gpu` (an older browser, or an insecure context: WebGPU
 * needs https or localhost) or no adapter is not supported, and the player
 * shell says so instead of starting the engine. A device the adapter then
 * refuses is the engine Worker's start failure (`og_web_attach_canvas`).
 * It asks for the adapter the renderer will ask for: its power preference,
 * `low-power` where the player chose it (`lowPower`), else `high-performance`
 * (og-render `power_preference`). Chrome on a Linux host with an NVIDIA GPU
 * answers a request without one null and that one the GPU. It asks for an
 * adapter: call it off the page's main thread.
 */
export async function webGpuMissing(lowPower: boolean): Promise<string | null> {
  const absent = webGpuAbsent();
  if (absent) return absent;
  const gpu = (globalThis.navigator as { gpu: { requestAdapter(options: { powerPreference: string }): Promise<unknown> } }).gpu;
  try {
    return (await gpu.requestAdapter({ powerPreference: lowPower ? "low-power" : "high-performance" })) ? null : "the browser found no WebGPU adapter";
  } catch (e) {
    return `the browser's WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`;
  }
}
