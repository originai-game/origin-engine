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
  og_node_attach_offscreen(engine: number, width: number, height: number): string;
  og_node_probe_listen(engine: number, port: number): string;
  og_node_probe_serve(engine: number): number;
  og_node_probe_close(engine: number): boolean;
}

let extras: NativeExtras | null = null;
const arrays = new WeakMap<ArrayBuffer, ColumnArray>();

/** Load the addon and bind the runtime to it (once per process). */
export function loadNativeHost(path: string = addonPath()): HostBinding {
  const bound = boundHost();
  if (bound) return bound;
  if (!existsSync(path)) {
    throw new Error(`no native addon at ${path}: build it with \`bun run build\` in packages/engine (cargo build --release -p og-host-node), or point OG_NATIVE_ADDON at one`);
  }
  const addon = createRequire(import.meta.url)(path) as HostAbi<Uint32Array, unknown> & NativeExtras;
  extras = addon;
  const binding: HostBinding = {
    abi: addon,
    name: "node",
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

/** Import a game bundle (its default export is `og.game(...)`) and assemble its engine (`options.root`: the project whose cooked/ it opens). */
export async function loadGame(entry: string, options: CreateOptions = {}): Promise<Result<Engine>> {
  loadNativeHost();
  const mod = (await import(pathToFileURL(resolve(entry)).href)) as { default?: GameToken };
  if (!mod.default) return { ok: false, reason: `${entry} has no default export: export default og.game({ ... })` };
  return createEngine(mod.default, options);
}

export { createEngine };
export type { Engine, Result };
