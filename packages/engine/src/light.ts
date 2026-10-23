// The depth wave's baked lighting (IDL `render`: bakedLighting): a thin
// call into the host's binding symbol, typed by the generated declarations
// through index.ts.

import type { HostAbi } from "./generated/abi.ts";
import { boundHost } from "./runtime.ts";

type W = { readonly id: number };

function abi(): HostAbi<Uint32Array, unknown> {
  const h = boundHost();
  if (!h) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return h.abi;
}

export const lightRender = Object.freeze({
  bakedLighting: (world: W) => JSON.parse(abi().og_render_baked_lighting(world.id)) as unknown,
});
