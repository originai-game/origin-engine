// The Hermes host loader: the engine's C ABI host (crates/og-host-c) as a
// native shell registers it with Hermes (crates/og-host-c/hermes/shell.cpp),
// `globalThis.__og_host`, every binding symbol. A store package's game runs
// on it (shell.ts), as does the game-frame benchmark (bench/game-frame/hermes.ts).

import type { HostAbi } from "../generated/abi.ts";
import { ARRAYS, type ColumnArray, type HostBinding } from "../runtime.ts";

declare const __og_host: HostAbi<Uint32Array, unknown>;
declare const __og_now: () => number;

const arrays = new WeakMap<ArrayBuffer, ColumnArray>();

/** The shell's host as the runtime binds it; Hermes has no `performance`, so the shell's clock stands in. */
export function hermesBinding(): HostBinding {
  const g = globalThis as unknown as { performance?: { now(): number } };
  g.performance ??= { now: __og_now };
  const abi = __og_host;
  let shared: ArrayBuffer | null = null;
  return {
    abi,
    name: "hermes",
    shared: () => ({ buffer: (shared ??= abi.og_host_shared() as ArrayBuffer), offset: 0 }),
    // Column views are ArrayBuffers over engine memory, one per field; each
    // keeps its typed array while the shell has not detached it.
    columns(raw, kinds) {
      if (!Array.isArray(raw) || raw.length === 0) return null;
      return (raw as ArrayBuffer[]).map((buffer, i) => {
        const known = arrays.get(buffer);
        if (known) return known;
        const made = new ARRAYS[kinds[i]!.array](buffer);
        arrays.set(buffer, made);
        return made;
      });
    },
  };
}
