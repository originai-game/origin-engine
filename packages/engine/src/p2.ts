// The phase 2 modules' namespaces (IDL `terrain`): thin calls into the host's binding symbols,
// typed by the generated declarations through index.ts.

import type { HostAbi } from "./generated/abi.ts";
import { boundHost } from "./runtime.ts";

type W = { readonly id: number };

function abi(): HostAbi<Uint32Array, unknown> {
  const h = boundHost();
  if (!h) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return h.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));

export const terrain = Object.freeze({
  load: (world: W, path: string) => parse(abi().og_terrain_load(world.id, path)),
  sample: (world: W, desc?: unknown) => parse(abi().og_terrain_sample(world.id, json(desc))),
  bounds: (world: W, data: number, origin?: unknown, quaternion?: unknown, scale?: unknown) =>
    parse(abi().og_terrain_bounds(world.id, data, json(origin), json(quaternion), json(scale))),
  edit: (world: W, data: number, brush?: unknown) => parse(abi().og_terrain_edit(world.id, data, json(brush))),
  layer: (world: W, data: number, index: number) => parse(abi().og_terrain_layer(world.id, data, index)),
  setLayer: (world: W, data: number, index: number, layer?: unknown) =>
    parse(abi().og_terrain_set_layer(world.id, data, index, json(layer))),
});
