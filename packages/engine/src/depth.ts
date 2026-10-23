// The depth wave's additions (IDL `render` geometry, lines, points and
// raycast; `debug`): thin calls into the host's binding symbols, typed by
// the generated declarations through index.ts.

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

// A typed array crosses the wire as a plain array (JSON.stringify writes
// a Float32Array as an object keyed by index).
const ATTRIBUTES = ["positions", "normals", "uvs", "colors", "tangents", "indices"] as const;
function attributes(desc: unknown): string | undefined {
  if (desc === undefined || desc === null || typeof desc !== "object") return json(desc);
  const d: Record<string, unknown> = { ...(desc as Record<string, unknown>) };
  for (const k of ATTRIBUTES) {
    const v = d[k];
    if (ArrayBuffer.isView(v)) d[k] = Array.from(v as unknown as ArrayLike<number>);
  }
  return JSON.stringify(d);
}

export const depthRender = Object.freeze({
  bufferGeometry: (world: W, desc?: unknown) => parse(abi().og_render_buffer_geometry(world.id, attributes(desc))),
  updateGeometry: (world: W, geometry: number, desc?: unknown) =>
    parse(abi().og_render_update_geometry(world.id, geometry, attributes(desc))),
  geometryInfo: (world: W, geometry: number) => parse(abi().og_render_geometry_info(world.id, geometry)),
  cylinderGeometry: (world: W, desc?: unknown) => parse(abi().og_render_cylinder_geometry(world.id, json(desc))),
  coneGeometry: (world: W, desc?: unknown) => parse(abi().og_render_cone_geometry(world.id, json(desc))),
  capsuleGeometry: (world: W, desc?: unknown) => parse(abi().og_render_capsule_geometry(world.id, json(desc))),
  torusGeometry: (world: W, desc?: unknown) => parse(abi().og_render_torus_geometry(world.id, json(desc))),
  ringGeometry: (world: W, desc?: unknown) => parse(abi().og_render_ring_geometry(world.id, json(desc))),
  circleGeometry: (world: W, desc?: unknown) => parse(abi().og_render_circle_geometry(world.id, json(desc))),
  tubeGeometry: (world: W, desc?: unknown) => parse(abi().og_render_tube_geometry(world.id, json(desc))),
  lines: (world: W, desc?: unknown) => parse(abi().og_render_lines(world.id, json(desc))),
  points: (world: W, desc?: unknown) => parse(abi().og_render_points(world.id, json(desc))),
  raycast: (world: W, ray: unknown, desc?: unknown) => parse(abi().og_render_raycast(world.id, JSON.stringify(ray), json(desc))),
});

export const debug = Object.freeze({
  line: (world: W, from: unknown, to: unknown, desc?: unknown) =>
    abi().og_debug_line(world.id, JSON.stringify(from), JSON.stringify(to), json(desc)),
  box: (world: W, center: unknown, size: unknown, desc?: unknown) =>
    abi().og_debug_box(world.id, JSON.stringify(center), JSON.stringify(size), json(desc)),
  sphere: (world: W, center: unknown, radius: number, desc?: unknown) =>
    abi().og_debug_sphere(world.id, JSON.stringify(center), radius, json(desc)),
  arrow: (world: W, from: unknown, to: unknown, desc?: unknown) =>
    abi().og_debug_arrow(world.id, JSON.stringify(from), JSON.stringify(to), json(desc)),
  axes: (world: W, position: unknown, size: number, desc?: unknown) =>
    abi().og_debug_axes(world.id, JSON.stringify(position), size, json(desc)),
  text: (world: W, position: unknown, text: string, desc?: unknown) =>
    abi().og_debug_text(world.id, JSON.stringify(position), text, json(desc)),
  clear: (world: W) => abi().og_debug_clear(world.id),
  setView: (world: W, view: string) => abi().og_debug_set_view(world.id, JSON.stringify(view)),
  // A publish build answers every debug member with a refusal: the view is none there.
  view: (world: W) => {
    const v = parse<unknown>(abi().og_debug_view(world.id));
    return typeof v === "string" ? v : "none";
  },
  setEnabled: (world: W, enabled: boolean) => abi().og_debug_set_enabled(world.id, enabled),
  enabled: (world: W) => abi().og_debug_enabled(world.id),
});
