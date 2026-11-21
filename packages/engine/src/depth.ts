// The `custom` members of `render` geometry and `debug` (IDL): a geometry's
// typed arrays cross as plain arrays, and a publish build's debug view is none.

import type { Custom } from "./generated/calls.ts";
import { abi } from "./runtime/host.ts";
import { json, parse } from "./runtime/result.ts";

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

export const geometryCustom: Pick<Custom["render"], "bufferGeometry" | "updateGeometry"> = {
  bufferGeometry: (world, desc) => parse(abi().og_render_buffer_geometry(world.id, attributes(desc))),
  updateGeometry: (world, geometry, desc) => parse(abi().og_render_update_geometry(world.id, geometry, attributes(desc))),
};

export const debugCustom: Custom["debug"] = {
  // A publish build answers every debug member with a refusal: the view is none there.
  view: (world) => {
    const v = parse<unknown>(abi().og_debug_view(world.id));
    return typeof v === "string" ? v : "none";
  },
};
