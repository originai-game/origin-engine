// @origingame/engine: the runtime implementation of the generated
// declarations. Every export is typed by types/index.d.ts (the contract
// generated from crates/og-api/api/<module>.ogt), so a runtime member that drifts
// from the IDL fails `bun run typecheck`.

import type * as Api from "@origingame/engine";
import { BUILTINS, calls, clip, environmentSunRotation, top, ui } from "./runtime.ts";
import * as systems from "./systems/index.ts";
import { inventoryModule } from "./systems/inventory.ts";
import { abilitiesModule } from "./systems/abilities.ts";
import { assertModule } from "./assert.ts";
import { Path2, Shape } from "./geom.ts";
import { formatNotation } from "./ui/data/format.ts";

// The math layer (IDL `math`): three.js's methods as pure functions over the plain structs.
export { vec3, vec2, quat, mat4, box3, sphere, plane, frustum, ray, color, math } from "./math.ts";

// Capacity-taking factories share the same implementation with pure schemas.
export { str, list } from "./schema.ts";
export * from "./generated/exports.ts";
export * from "./generated/vocabulary.ts";

// The package's own functions and og's namespaces are generated from the IDL
// (src/generated/calls.ts); what is added here is TypeScript the IDL declares
// `host false` (og.anim.clip, og.load.environmentSunRotation, og.assert's sites, og.Shape, og.inventory, og.abilities) and og.ui's
// TypeScript-side scope, memoized frame queries, and formatNumber.
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
  load: Object.freeze({ ...calls.load, environmentSunRotation }),
  assert: Object.freeze({ ...calls.assert, ...assertModule }),
  ui: Object.freeze({ ...ui, formatNumber: formatNotation }),
  inventory: inventoryModule,
  abilities: abilitiesModule,
  systems: Object.freeze({ ...systems }),
}) as unknown as typeof Api.og;
