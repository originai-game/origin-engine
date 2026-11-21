// Field telemetry's pure parts (docs/web-export.md §Telemetry): the page's
// browser and device classes and its memory report.
import { expect, test } from "bun:test";
import { browserOf, deviceOf, memoryOf } from "../shell/telemetry.ts";

test("browsers and devices are classed from coarse facts", () => {
  const edge = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0";
  expect(browserOf(edge)).toEqual({ name: "edge", major: 141, os: "windows" });
  const chrome = "Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36";
  expect(browserOf(chrome)).toEqual({ name: "chrome", major: 141, os: "android" });
  expect(deviceOf({ ua: chrome, coarse: true, touchPoints: 5, screen: { width: 412, height: 915 }, dpr: 2.625 }).class).toBe("phone");
  // iPadOS reports a Mac; its touch points tell it apart.
  const ipad = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Safari/605.1.15";
  expect(deviceOf({ ua: ipad, coarse: true, touchPoints: 5, screen: { width: 820, height: 1180 }, dpr: 2 }).class).toBe("tablet");
  expect(deviceOf({ ua: ipad, coarse: false, touchPoints: 0, screen: { width: 1512, height: 982 }, dpr: 2 }).class).toBe("desktop");
  expect(browserOf("Mozilla/5.0 (X11; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0")).toEqual({ name: "firefox", major: 144, os: "linux" });
});

test("a memory report carries the renderer's GPU memory and device losses, and null figures where the engine has no renderer", () => {
  const m = memoryOf({ linear: 64, heap: { live: 8, peak: 9 }, stream: null, audio: null, world: { entities: 2 }, gpu: { textures: 1000, meshes: 200, renderTargets: 30, buffers: 4, total: 1234 }, deviceLosses: 2 });
  expect([m?.gpuTotal, m?.gpuTextures, m?.gpuTargets, m?.deviceLosses]).toEqual([1234, 1000, 30, 2]);
  const none = memoryOf({ linear: 64, heap: null, stream: null, audio: null, world: { entities: 0 }, gpu: null, deviceLosses: null });
  expect([none?.gpuTotal, none?.gpuTextures, none?.heapLive, none?.deviceLosses]).toEqual([null, null, null, null]);
});
