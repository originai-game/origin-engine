// The native host loader: the napi addon (origin-engine.node, built from
// crates/og-host-node) under bun or node, headless. `loadGame` imports a game
// bundle and assembles its engine; `og run --headless` drives it (run.ts).

import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join, resolve } from "node:path";
import {
  ARRAYS,
  bindHost,
  boundHost,
  type ColumnArray,
  createEngine,
  type CreateOptions,
  type Engine,
  type GameToken,
  type HostBinding,
  type Result,
} from "../runtime.ts";
import type { HostAbi } from "../generated/abi.ts";

/**
 * The addon the native loader uses: `OG_NATIVE_ADDON`, else this package's
 * `native/origin-engine.<platform>-<arch>.node` (a checkout's build), else the
 * per-platform package npm installed through optionalDependencies
 * (`@origingame/engine-<platform>-<arch>/origin-engine.node`).
 */
export function addonPath(): string {
  const fromEnv = process.env.OG_NATIVE_ADDON;
  if (fromEnv) return resolve(fromEnv);
  const here = dirname(fileURLToPath(import.meta.url));
  const platform = `${process.platform}-${process.arch}`;
  const local = join(here, "..", "..", "native", `origin-engine.${platform}.node`);
  if (existsSync(local)) return local;
  try {
    const require = createRequire(import.meta.url) as unknown as { resolve(id: string): string };
    return require.resolve(`@origingame/engine-${platform}/origin-engine.node`);
  } catch {
    return local;
  }
}

/** The addon's own symbols beyond the shared ABI. */
export interface NativeExtras {
  og_node_views_mode(): string;
  /** Retain the device and compatible pipelines between sequential fresh runs. */
  og_node_resident(enabled: boolean): void;
  /** Redirect reads only; source-edit commands still write to the project. */
  og_node_input_root(root: string | null, frozen: string | null): void;
  og_node_attach_offscreen(engine: number, width: number, height: number): string;
  /** Waits for the offscreen renderer's GPU work (frame-time benches). */
  og_node_wait_gpu(engine: number): string;
  /** `og run` without --realtime: every frame is the script's, drawn at the configured quality and no sample of automatic quality. */
  og_node_scripted_frames(engine: number, on: boolean): void;
  og_node_capture_only(engine: number, on: boolean): void;
  og_node_probe_listen(engine: number, port: number): string;
  og_node_probe_serve(engine: number): number;
  og_node_probe_close(engine: number): boolean;
  /** Connect an engine's rooms to `og relay` (`ws://host:port/<room>` joins that room). A Result. */
  og_node_room_connect(engine: number, url: string): string;
  /** Carry room ops to the relay and its events to the engine; the events delivered. */
  og_node_room_pump(engine: number): number;
  og_node_room_close(engine: number): boolean;
  /** Start a relay in this process on 127.0.0.1 (0 a free port) for as long as it lives. A Result with the port. */
  og_node_relay_start(port: number): string;
  /** Last drawn RGBA8 rows (empty without a frame). `header` prefixes width and height as little-endian u32s. Development only. */
  og_node_render_frame(engine: number, header?: boolean): Uint8Array;
  /** `og render --jobs`: the world as a render snapshot (empty when refused). */
  og_node_render_snapshot(engine: number): Uint8Array;
  /** Restores a render snapshot another process took: a wire Result of its tick. */
  og_node_render_restore(engine: number, snapshot: Uint8Array): string;
  /** Waits for every pipeline compile the process started (the full lit variants included), so the display frames after it repeat byte for byte. Development only. */
  og_node_settle_pipelines(): void;
  /** `og replay`: the world's portable hash within a `{ type: [field, ...] }` table, `{ hash }` (016x); with an empty table within the world's own, `{ hash, table }`; `{ error }` when refused. Not in publish builds. */
  og_node_replay_hash(engine: number, tableJson: string): string;
  /** `og replay compare`: og-net's diff of a recorded save (`og-save:2:...`, local) against the world now (remote), JSON, or `{ error }`. Not in publish builds. */
  og_node_replay_diff(engine: number, saveText: string): string;
  /** `og ui check`: the UI's last laid-out frame (elements, overlaps, drawn text, fills, the accessibility tree, counts, viewport and theme facts; src/node/ui-check.ts's UiFrameSnapshot), or `{ error }`. Not in publish builds. */
  og_node_ui_frame(engine: number): string;
}

let extras: NativeExtras | null = null;
const arrays = new WeakMap<ArrayBuffer, ColumnArray>();

/** Load the addon and bind the runtime to it (once per process). */
export function loadNativeHost(path: string = addonPath()): HostBinding {
  const bound = boundHost();
  if (bound) return bound;
  if (!existsSync(path)) {
    throw new Error(`no native addon at ${path}: build it with \`bun run build\` in packages/engine (scripts/build.sh), or point OG_NATIVE_ADDON at one`);
  }
  const addon = createRequire(import.meta.url)(path) as HostAbi<Uint32Array, unknown> & NativeExtras;
  extras = addon;
  process.on("exit", () => addon.og_node_resident?.(false));
  // An ArrayBuffer the addon made and keeps: it never moves.
  const shared = { buffer: addon.og_host_shared() as ArrayBuffer, offset: 0 };
  const binding: HostBinding = {
    abi: addon,
    name: "node",
    shared: () => shared,
    columns(raw, kinds, _count) {
      if (!Array.isArray(raw) || raw.length === 0) return null;
      // The host hands out the same buffer while a column stays put: so does this.
      return (raw as ArrayBuffer[]).map((buffer, i) => {
        const kind = kinds[i]!.array;
        const known = arrays.get(buffer);
        if (known && known.constructor.name === kind && known.byteLength === buffer.byteLength) return known;
        const made = new ARRAYS[kind](buffer);
        arrays.set(buffer, made);
        return made;
      });
    },
  };
  bindHost(binding);
  return binding;
}

/** The addon's extras (view mode, offscreen renderer), once loaded. */
export function native(): NativeExtras {
  if (!extras) loadNativeHost();
  return extras!;
}

/**
 * Attaches the headless renderer a native run draws with and answers its
 * tier. An unavailable adapter leaves simulation running with unchecked
 * frames; other initialization errors still fail.
 */
export function attachRenderer(
  engine: number,
  width: number,
  height: number,
  attach: (engine: number, width: number, height: number) => string = (e, w, h) => native().og_node_attach_offscreen(e, w, h),
): { ok: true; value: string; adapterError: string | null } | { ok: false; reason: string } {
  const attached = JSON.parse(attach(engine, width, height)) as { ok: boolean; value?: string; reason?: string };
  if (attached.ok) return { ok: true, value: attached.value!, adapterError: null };
  const reason = attached.reason ?? "no renderer";
  if (!reason.startsWith("no GPU adapter:")) return { ok: false, reason };
  const expected = process.env.OG_RENDERER?.trim().toLowerCase() === "none";
  process.stderr.write(`WARNING ${expected ? "" : "unexpected "}missing GPU: ${reason}; GPU checks are skipped/unchecked, not passed\n`);
  return { ok: true, value: "none", adapterError: reason };
}

/** Import a game bundle (its default export is `og.game(...)`) and assemble its engine (`options.root`: the project whose cooked/ it opens). */
export async function loadGame(entry: string, options: CreateOptions = {}): Promise<Result<Engine>> {
  loadNativeHost();
  const mod = (await import(pathToFileURL(resolve(entry)).href)) as { default?: GameToken };
  if (!mod.default) return { ok: false, reason: `${entry} has no default export: export default og.game({ ... })` };
  return createEngine(mod.default, options);
}

export { createEngine };
export type { Engine, Result };
