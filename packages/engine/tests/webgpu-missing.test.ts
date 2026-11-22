// The shell's WebGPU check (src/web/shell.ts webGpuMissing): the web is
// WebGPU only (docs/design.md §8), so a page without it gets a reason the
// player shell shows instead of starting the engine, and a page with it
// gets null.
import { expect, test } from "bun:test";
import { webGpuMissing } from "../src/web/shell.ts";
import { adapterName } from "../src/web/webgpu.ts";

const original = Object.getOwnPropertyDescriptor(globalThis, "navigator");
const secure = Object.getOwnPropertyDescriptor(globalThis, "isSecureContext");
const withGpu = (gpu: unknown, isSecureContext = true) => {
  Object.defineProperty(globalThis, "navigator", { value: gpu === undefined ? {} : { gpu }, configurable: true, writable: true });
  Object.defineProperty(globalThis, "isSecureContext", { value: isSecureContext, configurable: true, writable: true });
};
const restore = () => {
  if (original) Object.defineProperty(globalThis, "navigator", original);
  if (secure) Object.defineProperty(globalThis, "isSecureContext", secure);
  else delete (globalThis as { isSecureContext?: boolean }).isSecureContext;
};

test("no navigator.gpu, an insecure page, no adapter: each a reason; an adapter: null", async () => {
  try {
    withGpu(undefined);
    expect(await webGpuMissing(false)).toBe("navigator.gpu is missing");
    withGpu(undefined, false);
    expect(await webGpuMissing(false)).toMatch(/not a secure context/);
    withGpu({ requestAdapter: async () => null });
    expect(await webGpuMissing(false)).toBe("the browser found no WebGPU adapter");
    withGpu({ requestAdapter: async () => Promise.reject(new Error("blocked")) });
    expect(await webGpuMissing(false)).toBe("the browser's WebGPU did not start: blocked");
    withGpu({ requestAdapter: async () => ({}) });
    expect(await webGpuMissing(false)).toBeNull();
  } finally {
    restore();
  }
});

test("it asks for the renderer's adapter: high-performance, or low-power where the player chose it", async () => {
  try {
    // Chrome on a Linux host with an NVIDIA GPU: no adapter without a preference, the GPU with one.
    const asked: unknown[] = [];
    withGpu({ requestAdapter: async (o?: { powerPreference?: string }) => (asked.push(o?.powerPreference), o?.powerPreference ? {} : null) });
    expect(await webGpuMissing(false)).toBeNull();
    expect(await webGpuMissing(true)).toBeNull();
    expect(asked).toEqual(["high-performance", "low-power"]);
  } finally {
    restore();
  }
});

test("a timing's adapter: vendor/architecture, software on a CPU rasterizer however it is told", () => {
  expect(adapterName({ vendor: "nvidia", architecture: "lovelace", isFallbackAdapter: false })).toBe("nvidia/lovelace");
  // Chrome's SwiftShader is the fallback adapter; Dawn on llvmpipe is not, and is known by name; the renderer's own flag counts too.
  expect(adapterName({ vendor: "google", architecture: "swiftshader", isFallbackAdapter: true })).toBe("google/swiftshader (software)");
  expect(adapterName({ vendor: "mesa", architecture: "llvmpipe", isFallbackAdapter: false })).toBe("mesa/llvmpipe (software)");
  expect(adapterName({ vendor: "apple", architecture: "" }, true)).toBe("apple/unknown (software)");
  expect(adapterName(null)).toBe("unknown");
});
