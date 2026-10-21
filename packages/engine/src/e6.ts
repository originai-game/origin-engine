// The E6 modules' namespaces (IDL `nav`, `ai`, `platform`, `i18n`): thin calls into the host's
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
  restart: (world: W, entity: number) => parse(abi().og_ai_restart(world.id, entity)),
  noise: (world: W, desc?: unknown) => parse(abi().og_ai_noise(world.id, json(desc))),
  describe: (world: W, entity: number) => parse(abi().og_ai_describe(world.id, entity)),
});

export const platform = Object.freeze({
  storeGet: (world: W, key: string) => parse(abi().og_platform_store_get(world.id, key)),
  storeSet: (world: W, desc?: unknown) => parse(abi().og_platform_store_set(world.id, json(desc))),
  storeList: (world: W) => abi().og_platform_store_list(world.id),
  leaderboardSubmit: (world: W, desc?: unknown) => parse(abi().og_platform_leaderboard_submit(world.id, json(desc))),
  leaderboardTop: (world: W, desc?: unknown) => abi().og_platform_leaderboard_top(world.id, json(desc)),
  leaderboardAround: (world: W, desc?: unknown) => abi().og_platform_leaderboard_around(world.id, json(desc)),
  telemetry: (world: W, name: string, attrs?: unknown) => parse(abi().og_platform_telemetry(world.id, name, json(attrs))),
  point: (world: W, desc?: unknown) => parse(abi().og_platform_point(world.id, json(desc))),
  aiMark: (world: W, desc?: unknown) => parse(abi().og_platform_ai_mark(world.id, json(desc))),
  loadSettings: (world: W) => abi().og_platform_load_settings(world.id),
  saveSettings: (world: W, value: unknown) => parse(abi().og_platform_save_settings(world.id, JSON.stringify(value))),
  result: (world: W, request: number) => parse(abi().og_platform_result(world.id, request)),
  info: (world: W) => parse(abi().og_platform_info(world.id)),
  now: (world: W) => abi().og_platform_now(world.id),
});

export const i18n = Object.freeze({
  load: (world: W, path: string) => parse(abi().og_i18n_load(world.id, path)),
  setLocale: (world: W, locale: string) => parse(abi().og_i18n_set_locale(world.id, locale)),
  locale: (world: W) => parse(abi().og_i18n_locale(world.id)),
  t: (world: W, key: string, args?: unknown) => abi().og_i18n_t(world.id, key, json(args)),
  has: (world: W, key: string) => abi().og_i18n_has(world.id, key),
});
