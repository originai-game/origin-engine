// @origingame/engine: the runtime implementation of the generated
// declarations. Every export is typed by types/index.d.ts (the contract
// generated from crates/og-api/api/<module>.ogt), so a runtime member that drifts
// from the IDL fails `bun run typecheck`.

import type * as Api from "@origingame/engine";
import { BUILTINS, calls, clip, fieldSpec, top, ui } from "./runtime.ts";
import * as systems from "./systems/index.ts";
import { inventoryModule } from "./systems/inventory.ts";
import { assertModule } from "./assert.ts";
import { Path2, Shape } from "./geom.ts";
import { uiKit } from "./ui-kit/index.ts";
import { formatNotation } from "./ui/data/format.ts";

// The math layer (IDL `math`): three.js's methods as pure functions over the plain structs.
export { vec3, vec2, quat, mat4, box3, sphere, plane, frustum, ray, color, math } from "./math.ts";

// The column factories that take a capacity: the rest, and every constant, are generated from the IDL.
// The capacity crosses with the spec; the engine refuses one outside 1 to 4096 at declaration.
export const str = ((capacity, value = "", options) => Object.freeze({ ...fieldSpec("str", value, options), capacity })) as typeof Api.str;
// A list of another factory's kind: the engine refuses a kind that is not a number or a handle, and a capacity outside 1 to 1024, at declaration.
export const list = ((of, capacity, options) =>
  Object.freeze({ kind: of.kind, default: Object.freeze([]), options: Object.freeze({ ...(options ?? {}) }), list: capacity })) as typeof Api.list;
export * from "./generated/exports.ts";
export * from "./generated/vocabulary.ts";

// The package's own functions and og's namespaces are generated from the IDL
// (src/generated/calls.ts); what is added here is TypeScript the IDL declares
// `host false` (og.anim.clip, og.assert's sites, og.Shape, og.inventory) and og.ui, whose
// hand-written members go once its IDL settles.
export const { defineComponent, defineResource, defineEvent, defineSystem, defineHook } = top as unknown as Pick<
  typeof Api,
  "defineComponent" | "defineResource" | "defineEvent" | "defineSystem" | "defineHook"
>;

export const og: typeof Api.og = Object.freeze({
  ...BUILTINS,
  ...calls,
  Shape,
  Path2,
  anim: Object.freeze({ ...calls.anim, clip }),
  assert: Object.freeze({ ...calls.assert, ...assertModule }),
  ui: Object.freeze({ ...ui, ...uiKit, formatNumber: formatNotation }),
  inventory: inventoryModule,
  systems: Object.freeze({ ...systems }),
}) as unknown as typeof Api.og;
