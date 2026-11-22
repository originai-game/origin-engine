// How the web page makes its renderer's pipelines (src/web/worker/device.ts
// `choose`): what its build asked, or auto's pick at the adapter.
import { expect, test } from "bun:test";
import { choose } from "../src/web/worker/device.ts";

test("auto picks async on every GPU but Apple's, and says why", () => {
  const mode = (asked: Parameters<typeof choose>[0], info: Parameters<typeof choose>[1]) => choose(asked, info).mode;
  expect(mode(undefined, { vendor: "img-tec", architecture: "d-series" })).toBe("async");
  expect(mode(undefined, { vendor: "qualcomm", architecture: "adreno-7xx" })).toBe("async");
  expect(mode(undefined, { vendor: "", architecture: "valhall" })).toBe("async");
  // A desktop GPU too: a cold driver cache compiles synchronous ones past Chrome's GPU watchdog.
  expect(mode(undefined, { vendor: "nvidia", architecture: "lovelace" })).toBe("async");
  expect(choose(undefined, { vendor: "nvidia", architecture: "lovelace" }).reason).toMatch(/not an Apple GPU \(nvidia lovelace\)/);
  expect(mode(undefined, { vendor: "", architecture: "" })).toBe("async");
  expect(mode(undefined, undefined)).toBe("async");
  expect(choose(undefined, { vendor: "apple", architecture: "metal-3" })).toEqual({ mode: "sync", reason: "auto: an Apple GPU (apple metal-3), no consistent async advantage on M4 Pro" });
  expect(mode("sync", { vendor: "qualcomm" })).toBe("sync");
  expect(choose("prewarm", { vendor: "apple" })).toEqual({ mode: "prewarm", reason: "publish { pipelines prewarm }" });
});
