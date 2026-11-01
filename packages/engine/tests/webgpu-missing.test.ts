// The shell's WebGPU check (src/web/shell.ts webGpuMissing): the web is
// WebGPU only (docs/design.md §8), so a page without it gets a reason the
// player shell shows instead of starting the engine, and a page with it
// gets null.
import { expect, test } from "bun:test";
import { webGpuMissing } from "../src/web/shell.ts";

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
    expect(await webGpuMissing()).toBe("navigator.gpu is missing");
    withGpu(undefined, false);
    expect(await webGpuMissing()).toMatch(/not a secure context/);
    withGpu({ requestAdapter: async () => null });
    expect(await webGpuMissing()).toBe("the browser found no WebGPU adapter");
    withGpu({ requestAdapter: async () => Promise.reject(new Error("blocked")) });
    expect(await webGpuMissing()).toBe("the browser's WebGPU did not start: blocked");
    withGpu({ requestAdapter: async () => ({}) });
    expect(await webGpuMissing()).toBeNull();
  } finally {
    restore();
  }
});
