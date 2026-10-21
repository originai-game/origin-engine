// `og doctor`'s engine checks, as one JSON line: does this package's native
// addon load, is the web host's wasm built, and can this machine render a
// frame headless (Metal on a Mac, Vulkan on a GPU). No renderer is a
// supported configuration, not a problem: the playtests' frames are then
// unchecked (OG_RENDERER=none makes any machine so).
//
//   bun src/node/doctor.ts

import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { addonPath, loadNativeHost, native } from "./index.ts";
import { createEngine, type GameToken } from "../runtime.ts";
import { og } from "../index.ts";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const version = (JSON.parse(readFileSync(join(pkg, "package.json"), "utf8")) as { version: string }).version;

const report: Record<string, unknown> = { package: pkg, version };
const addon = addonPath();
try {
  if (!existsSync(addon)) throw new Error(`no addon at ${addon}`);
  loadNativeHost(addon);
  report.addon = { ok: true, path: addon, views: native().og_node_views_mode() };
} catch (e) {
  report.addon = { ok: false, path: addon, reason: e instanceof Error ? e.message : String(e) };
}
const wasm = join(pkg, "dist", "web", "origin-engine_bg.wasm");
report.wasm = existsSync(wasm) ? { ok: true, path: wasm, bytes: statSync(wasm).size } : { ok: false, path: wasm };
const publishWasm = join(pkg, "dist", "web-publish", "origin-engine_bg.wasm");
report.publishWasm = existsSync(publishWasm) ? { ok: true, path: publishWasm, bytes: statSync(publishWasm).size } : { ok: false, path: publishWasm };
// The 2D publish variant: what `og build --publish` ships for a 2d game (docs/web-export.md §Variants).
const publishWasm2d = join(pkg, "dist", "web-publish-2d", "origin-engine_bg.wasm");
report.publishWasm2d = existsSync(publishWasm2d) ? { ok: true, path: publishWasm2d, bytes: statSync(publishWasm2d).size } : { ok: false, path: publishWasm2d };
// The lanes variants (wasm atomics), one per physics profile beside its
// publish wasm: `og build` ships the one of the game's profile for cross-origin
// isolated pages (docs/web-export.md §Lanes, §Variants).
const lanesWasm = join(pkg, "dist", "web-publish", "origin-engine-lanes_bg.wasm");
report.lanesWasm = existsSync(lanesWasm) ? { ok: true, path: lanesWasm, bytes: statSync(lanesWasm).size } : { ok: false, path: lanesWasm };
const lanesWasm2d = join(pkg, "dist", "web-publish-2d", "origin-engine-lanes_bg.wasm");
report.lanesWasm2d = existsSync(lanesWasm2d) ? { ok: true, path: lanesWasm2d, bytes: statSync(lanesWasm2d).size } : { ok: false, path: lanesWasm2d };

if ((report.addon as { ok: boolean }).ok) {
  try {
    const t0 = performance.now();
    const created = createEngine(og.game({}) as unknown as GameToken);
    if (!created.ok) throw new Error(created.reason);
    const attached = JSON.parse(native().og_node_attach_offscreen(created.value.id, 64, 64)) as { ok: boolean; value?: string; reason?: string };
    if (!attached.ok) throw new Error(attached.reason);
    const frame = created.value.probe.capture(64, 64) as { width: number } | null;
    if (!frame) throw new Error("the renderer attached but produced no frame");
    report.renderer = { ok: true, tier: attached.value, ms: +(performance.now() - t0).toFixed(0) };
  } catch (e) {
    report.renderer = { ok: false, reason: e instanceof Error ? e.message : String(e) };
  }
} else {
  report.renderer = { ok: false, reason: "needs the native addon" };
}
process.stdout.write(JSON.stringify(report) + "\n");
