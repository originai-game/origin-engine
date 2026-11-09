// @origingame/engine: the runtime implementation of the generated
// declarations. Every export is typed by types/index.d.ts (the contract
// generated from crates/og-api/api/<module>.ogt), so a runtime member that drifts
// from the IDL fails `bun run typecheck`.

import type * as Api from "@origingame/engine";
import {
  anim,
  BUILTINS,
  commands,
  defineGame,
  defineRecord,
  defineSystem as defineSystemImpl,
  defineHook as defineHookImpl,
  load,
  physics,
  atmosphere,
  audio,
  render,
  transform,
  ui,
  time,
  camera,
  tween,
  tilemap,
  fieldSpec,
  type FieldSpec,
} from "./runtime.ts";
import { cameras } from "./systems/cameras.ts";
import { ai, i18n, nav, net, platform, story } from "./e6.ts";
import { input } from "./input.ts";
import { destruction, motion, path, procgen, stream, terrain, timeline, vfx, video } from "./p2.ts";
import { debug, depthRender } from "./depth.ts";
import { assertModule } from "./assert.ts";
import { geomRender, Path2, Shape } from "./geom.ts";
import { targetsLoad, targetsRender } from "./targets.ts";
import { lightRender } from "./light.ts";
import { uiKit } from "./ui-kit/index.ts";
import { feedback } from "./ui/feedback/index.ts";

// The math layer (IDL `math`): three.js's methods as pure functions over the plain structs.
export { vec3, vec2, quat, mat4, box3, sphere, plane, frustum, ray, color, math } from "./math.ts";

// The column factories that take a capacity: the rest, and every constant, are generated from the IDL.
// The capacity crosses with the spec; the engine refuses one outside 1 to 4096 at declaration.
export const str = ((capacity, value = "", options) => Object.freeze({ ...fieldSpec("str", value, options), capacity })) as typeof Api.str;
// A list of another factory's kind: the engine refuses a kind that is not a number or a handle, and a capacity outside 1 to 1024, at declaration.
export const list = ((of, capacity, options) =>
  Object.freeze({ kind: of.kind, default: Object.freeze([]), options: Object.freeze({ ...(options ?? {}) }), list: capacity })) as typeof Api.list;
export * from "./generated/exports.ts";

export const defineComponent: typeof Api.defineComponent = (name, fields) =>
  defineRecord("component", name, fields as unknown as Record<string, FieldSpec>) as never;

export const defineResource: typeof Api.defineResource = (name, fields) =>
  defineRecord("resource", name, fields as unknown as Record<string, FieldSpec>) as never;

export const defineEvent: typeof Api.defineEvent = (name, fields) =>
  defineRecord("event", name, fields as unknown as Record<string, FieldSpec>) as never;

export const defineSystem: typeof Api.defineSystem = (name, desc, run) =>
  defineSystemImpl(name, desc as never, run) as never;

export const defineHook: typeof Api.defineHook = (name, component, desc, run) =>
  defineHookImpl(name, component as never, desc as never, run) as never;

export const og: typeof Api.og = Object.freeze({
  ...BUILTINS,
  game: (desc?: Api.GameDesc) => defineGame(desc as never),
  transform,
  Shape,
  Path2,
  render: Object.freeze({ ...render, ...depthRender, ...geomRender, ...targetsRender, ...lightRender }),
  load: Object.freeze({ ...load, ...targetsLoad }),
  commands,
  physics,
  atmosphere,
  audio,
  anim,
  time,
  camera,
  tween,
  tilemap,
  nav,
  ai,
  story,
  platform,
  i18n,
  net,
  input,
  terrain,
  stream,
  debug,
  assert: assertModule,
  path,
  procgen,
  timeline,
  motion,
  destruction,
  vfx,
  video,
  ui: Object.freeze({ ...ui, ...uiKit, ...feedback }),
  systems: Object.freeze({ cameras }),
}) as unknown as typeof Api.og;
