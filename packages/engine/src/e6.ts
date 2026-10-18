// The E6 modules' namespaces (IDL `nav`, `ai`): thin calls into the host's
// binding symbols, typed by the generated declarations through index.ts.

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

export const nav = Object.freeze({
  grid: (world: W, desc?: unknown) => parse(abi().og_nav_grid(world.id, json(desc))),
  gridFromTilemap: (world: W, desc?: unknown) => parse(abi().og_nav_grid_from_tilemap(world.id, json(desc))),
  setCost: (world: W, grid: number, x: number, y: number, cost: number) => parse(abi().og_nav_set_cost(world.id, grid, x, y, cost)),
  cellAt: (world: W, grid: number, point: unknown) => parse(abi().og_nav_cell_at(world.id, grid, JSON.stringify(point))),
  findPath: (world: W, desc?: unknown) => parse(abi().og_nav_find_path(world.id, json(desc))),
  flowField: (world: W, desc?: unknown) => parse(abi().og_nav_flow_field(world.id, json(desc))),
  setFlowTargets: (world: W, field: number, targets: unknown) => parse(abi().og_nav_set_flow_targets(world.id, field, JSON.stringify(targets))),
  flowAt: (world: W, field: number, point: unknown) => parse(abi().og_nav_flow_at(world.id, field, JSON.stringify(point))),
  load: (world: W, path: string) => parse(abi().og_nav_load(world.id, path)),
  build: (world: W, desc?: unknown) => parse(abi().og_nav_build(world.id, json(desc))),
  setObstacle: (world: W, mesh: number, id: number, obstacle?: unknown) => parse(abi().og_nav_set_obstacle(world.id, mesh, id, json(obstacle))),
  nearest: (world: W, mesh: number, point: unknown, extent?: unknown) => parse(abi().og_nav_nearest(world.id, mesh, JSON.stringify(point), json(extent))),
  pathOf: (world: W, entity: number) => parse(abi().og_nav_path_of(world.id, entity)),
});

export const ai = Object.freeze({
  load: (world: W, path: string) => parse(abi().og_ai_load(world.id, path)),
  parse: (world: W, text: string) => parse(abi().og_ai_parse(world.id, text)),
  get: (world: W, entity: number, key: string) => parse(abi().og_ai_get(world.id, entity, key)),
  set: (world: W, entity: number, key: string, value: unknown) => parse(abi().og_ai_set(world.id, entity, key, JSON.stringify(value))),
  noise: (world: W, desc?: unknown) => parse(abi().og_ai_noise(world.id, json(desc))),
  describe: (world: W, entity: number) => parse(abi().og_ai_describe(world.id, entity)),
});
