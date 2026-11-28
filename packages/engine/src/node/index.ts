// The native host loader: the napi addon (origin-engine.node, built from
// crates/og-host-node) under bun or node, headless. `loadGame` imports a game
// bundle and assembles its engine; `og run --headless` drives it (run.ts).

import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join, resolve } from "node:path";
import { addonPath } from "./addon.ts";
export { addonPath };
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

/** The addon's own symbols beyond the shared ABI. */
export interface NativeExtras {
  og_node_views_mode(): string;
  /** The host's cooked-format stamp (og-bundle's format_stamp, 016x); absent in a host built before it existed. */
  og_node_formats?(): string;
  /** Retain the device and compatible pipelines between sequential fresh runs. */
  og_node_resident(enabled: boolean): void;
  /** Redirect reads only; source-edit commands still write to the project. */
  og_node_input_root(root: string | null, frozen: string | null): void;
  og_node_attach_offscreen(engine: number, width: number, height: number): string;
  /** Waits for the offscreen renderer's GPU work (frame-time benches). */
  og_node_wait_gpu(engine: number): string;
  /**
   * The kind of frame the host steps (og-runtime's FrameKind): "scripted" for `og run` without --realtime (drawn at the
   * configured quality, no sample of automatic quality, never waiting), "recorded" for `og render` (every frame is
   * written out, so each settles the work due by its tick, as a capture does), "display" otherwise.
   */
  og_node_frame_kind(engine: number, kind: "display" | "scripted" | "recorded"): void;
  og_node_capture_only(engine: number, on: boolean): void;
  og_node_probe_listen(engine: number, port: number): string;
  og_node_probe_serve(engine: number): number;
  og_node_probe_close(engine: number): boolean;
  /** Connect an engine's rooms to `og relay` (`ws://host:port/<room>` joins that room). A Result. */
  og_node_room_connect(engine: number, url: string): string;
  /** Carry room ops to the relay and its events to the engine; the events delivered. */
  og_node_room_pump(engine: number): number;
  og_node_room_close(engine: number): boolean;
  /** Give an engine a static session provider (`og run --shared`): a `ws://`/`wss://` world server, or `https://` for WebTransport with its certificate's SHA-256 in hex (empty: the system's roots), and the token sent as it is. False for another URL or a hash that is not one. */
  og_node_shared_attach(engine: number, url: string, token: string, certHash: string): boolean;
  /** Carry the engine's shared-world ops to its connection and what it says back; the events delivered. */
  og_node_shared_pump(engine: number): number;
  /** Start a relay in this process on 127.0.0.1 (0 a free port) for as long as it lives. A Result with the port. */
  og_node_relay_start(port: number): string;
  /** Last drawn RGBA8 rows (empty without a frame). `header` prefixes width and height as little-endian u32s. Development only. */
  og_node_render_frame(engine: number, header?: boolean): Uint8Array;
  /** Waits for every pipeline compile the process started (the full lit variants included), so the display frames after it repeat byte for byte. Development only. */
  og_node_settle_pipelines(): void;
  /** `og replay`: the world's portable hash within a `{ type: [field, ...] }` table, `{ hash }` (016x); with an empty table within the world's own, `{ hash, table }`; `{ error }` when refused. Not in publish builds. */
  og_node_replay_hash(engine: number, tableJson: string): string;
  /** `og replay compare`: og-net's diff of a recorded save (`og-save:2:...`, local) against the world now (remote), JSON, or `{ error }`. Not in publish builds. */
  og_node_replay_diff(engine: number, saveText: string): string;
  /** `og run --restart-at`: og-net's structural restart diff of a fresh start's save against the world now, within a `{ type: [field, ...] }` ignore table (`[]` the whole type), JSON (`{ differs, hash, startHash, components, names, entities, resources, events, blocks, unchecked }`) or `{ error }`. Not in publish builds. */
  og_node_restart_diff(engine: number, saveText: string, ignoreJson: string, keepJson: string): string;
  /** `og ui check`: the UI's last laid-out frame (elements, overlaps, drawn text, fills, the accessibility tree, counts, viewport and theme facts; src/node/ui-check.ts's UiFrameSnapshot), or `{ error }`. Not in publish builds. */
  og_node_ui_frame(engine: number): string;
  /** `og dev`: in-place asset/UI refresh without a bundle reload (`{ ms, steps, warnings }` or `{ error }`). Not in publish builds. */
  og_node_dev_refresh?(engine: number): string;
}

/**
 * Why the host library at `path` cannot read what the og that launched it
 * cooks, or null when it can: `cook` is that og's cooked-format stamp
 * (OG_COOK_FORMATS, which og sets; unset outside og), `host` the library's
 * (`og_node_formats`, undefined in a library older than it). Two stamps
 * differ exactly when og and the library were built from different sources,
 * one rebuilt and the other not; every loader would then refuse the blobs
 * ("holds type ...") and no re-cook could help. The message
 * names the command that rebuilds them together.
 */
export function formatsMismatch(path: string, cook: string | undefined, host: string | undefined): string | null {
  if (!cook || host === cook) return null;
  const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
  const checkout = resolve(packageRoot, "..", "..");
  const local = resolve(path).startsWith(join(packageRoot, "native")) && existsSync(join(checkout, "Cargo.toml"));
  let fix: string;
  if (process.env.OG_NATIVE_ADDON) {
    fix = `point OG_NATIVE_ADDON at the host library built with this og, or unset it`;
  } else if (local) {
    let profile = "release";
    try {
      profile = readFileSync(join(packageRoot, "native", "build-profile"), "utf8").trim() || profile;
    } catch {}
    const command = profile === "dev" ? "OG_GATE=1 sh packages/engine/scripts/build.sh --native" : "OG_RELEASE=1 sh packages/engine/scripts/build.sh --native";
    fix = `rebuild og and the host library together from the checkout: \`${command}\` in ${checkout} (rebuilding og alone, such as cargo build -p og-cli, leaves the host behind)`;
  } else {
    fix = `og and this engine package are from different installs; reinstall the engine (curl -fsSL https://origingame.ai/downloads/engine/install.sh | sh) and run the og it installs`;
  }
  const hostStamp = host ?? "none (a library older than the format stamp)";
  return `the native host library ${path} was built from other engine sources than this og: og cooks formats ${cook}, the library reads ${hostStamp}, so it would refuse every blob og cooks and re-cooking cannot help. Fix: ${fix}`;
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
  const mismatch = formatsMismatch(path, process.env.OG_COOK_FORMATS, addon.og_node_formats?.());
  if (mismatch) throw new Error(mismatch);
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
