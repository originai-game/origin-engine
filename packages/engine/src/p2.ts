// The phase 2 and depth modules' namespaces (IDL `terrain`, `stream`, `path`, `timeline`, `motion`): thin calls into the host's binding symbols,
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

export const stream = Object.freeze({
  status: (world: W, group: string) => parse(abi().og_stream_status(world.id, group)),
  request: (world: W, group: string) => parse(abi().og_stream_request(world.id, group)),
  release: (world: W, group: string) => parse(abi().og_stream_release(world.id, group)),
  budget: (world: W) => parse(abi().og_stream_budget(world.id)),
  setBudget: (world: W, bytes: number) => parse(abi().og_stream_set_budget(world.id, bytes)),
});

// Depth S2 (IDL `path`, `timeline`).
/** og.procgen: pure generators, every draw from the seed they are given. */
export const procgen = Object.freeze({
  noise: (desc: unknown, x: number, y: number, z: number) => abi().og_procgen_noise(JSON.stringify(desc ?? {}), x, y, z),
  noiseGrid: (desc?: unknown) => parse(abi().og_procgen_noise_grid(json(desc))),
  poissonDisk: (desc?: unknown) => parse(abi().og_procgen_poisson_disk(json(desc))),
  bsp: (desc?: unknown) => parse(abi().og_procgen_bsp(json(desc))),
  wfc: (desc?: unknown) => parse(abi().og_procgen_wfc(json(desc))),
});

export const path = Object.freeze({
  create: (world: W, desc?: unknown) => parse(abi().og_path_create(world.id, json(desc))),
  find: (world: W, name: string) => parse(abi().og_path_find(world.id, name)),
  sample: (world: W, p: number, t: number) => parse(abi().og_path_sample(world.id, p, t)),
  length: (world: W, p: number) => parse(abi().og_path_length(world.id, p)),
  nearest: (world: W, p: number, point: unknown) =>
    parse(abi().og_path_nearest(world.id, p, JSON.stringify(point))),
});

export const timeline = Object.freeze({
  info: (world: W, asset: number) => parse(abi().og_timeline_info(world.id, asset)),
  play: (world: W, entity: number) => parse(abi().og_timeline_play(world.id, entity)),
  pause: (world: W, entity: number) => parse(abi().og_timeline_pause(world.id, entity)),
  seek: (world: W, entity: number, time: number) => parse(abi().og_timeline_seek(world.id, entity, time)),
  stop: (world: W, entity: number) => parse(abi().og_timeline_stop(world.id, entity)),
});

/** og.video: how streams play on this host (the module's other calls are og.load.video and og.VideoPlayer). */
export const video = Object.freeze({
  capabilities: (world: W) => parse(abi().og_video_capabilities(world.id)),
});

// og.motion (IDL `motion`): goals for humanoids; the model draws them.
export const motion = Object.freeze({
  pose: (world: W, entity: number, desc?: unknown) => parse(abi().og_motion_pose(world.id, entity, json(desc))),
  place: (world: W, entity: number, desc?: unknown) => parse(abi().og_motion_place(world.id, entity, json(desc))),
  face: (world: W, entity: number, desc?: unknown) => parse(abi().og_motion_face(world.id, entity, json(desc))),
  walkTo: (world: W, entity: number, desc?: unknown) => parse(abi().og_motion_walk_to(world.id, entity, json(desc))),
  clear: (world: W, entity: number) => parse(abi().og_motion_clear(world.id, entity)),
  status: (world: W, entity: number) => parse(abi().og_motion_status(world.id, entity)),
  capabilities: (world: W) => parse(abi().og_motion_capabilities(world.id)),
  poses: () => parse(abi().og_motion_poses()),
});
