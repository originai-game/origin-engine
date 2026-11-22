// Whether this browser can run the engine at all: WebGPU (docs/design.md §8).
//
// The page asks only `webGpuAbsent`; the engine Worker's renderer asks for
// the adapter where boot first needs the device, after the wasm and the
// world (worker/start.ts, og_web_attach_canvas). Chrome's first
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
 * (og-render `power_preference`), at the `compatibility` feature level
 * (og-render `FEATURE_LEVEL`), so a GPU that runs only WebGPU's
 * compatibility mode (GLES, D3D11) is supported: the browser still hands a
 * core adapter wherever it has one. Chrome on a Linux host with an NVIDIA GPU
 * answers a request without a power preference null and one with it the GPU.
 * It asks once, so call it where the page needs the answer, not as it loads
 * (docs/ci.md §The final L4 pass), and off the page's main thread.
 */
export async function webGpuMissing(lowPower: boolean): Promise<string | null> {
  const absent = webGpuAbsent();
  if (absent) return absent;
  const gpu = (globalThis.navigator as { gpu: { requestAdapter(options: { powerPreference: string; featureLevel: string }): Promise<unknown> } }).gpu;
  try {
    const adapter = await gpu.requestAdapter({ powerPreference: lowPower ? "low-power" : "high-performance", featureLevel: "compatibility" });
    return adapter ? null : "the browser found no WebGPU adapter";
  } catch (e) {
    return `the browser's WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`;
  }
}

/**
 * A WebGPU adapter as a timing names it: `vendor/architecture` (`nvidia/lovelace`,
 * `apple/metal-3`), with ` (software)` on a CPU rasterizer (the browser's fallback
 * adapter, SwiftShader, llvmpipe, lavapipe), so a frame time measured on CPU
 * WebGPU is never read as a GPU's. `unknown` where the browser names neither.
 */
export function adapterName(info: { vendor?: unknown; architecture?: unknown; isFallbackAdapter?: unknown } | null | undefined, software = false): string {
  const vendor = typeof info?.vendor === "string" ? info.vendor : "";
  const architecture = typeof info?.architecture === "string" ? info.architecture : "";
  const name = vendor || architecture ? `${vendor || "unknown"}/${architecture || "unknown"}` : "unknown";
  return software || info?.isFallbackAdapter === true || /swiftshader|llvmpipe|lavapipe/i.test(name) ? `${name} (software)` : name;
}
