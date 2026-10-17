// @origingame/engine: the runtime implementation of the generated
// declarations. Every export is typed by types/index.d.ts (the contract
// generated from crates/og-api/api.ogt), so a runtime member that drifts
// from the IDL fails `bun run typecheck`.

import type * as Api from "@origingame/engine";
import {
  BUILTINS,
  commands,
  defineGame,
  defineRecord,
  defineSystem as defineSystemImpl,
  load,
  render,
  transform,
  type FieldSpec,
} from "./runtime.ts";

const spec = (kind: string, value: unknown, options: unknown): FieldSpec =>
  Object.freeze({ kind, default: value, options: Object.freeze({ ...((options as object | undefined) ?? {}) }) });

const vector2 = () => ({ x: 0, y: 0 });
const vector3 = () => ({ x: 0, y: 0, z: 0 });
const identity = (): Api.Matrix4["elements"] => [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

export const f32 = ((value = 0, options) => spec("f32", value, options)) as typeof Api.f32;
export const f64 = ((value = 0, options) => spec("f64", value, options)) as typeof Api.f64;
export const u32 = ((value = 0, options) => spec("u32", value, options)) as typeof Api.u32;
export const i32 = ((value = 0, options) => spec("i32", value, options)) as typeof Api.i32;
export const bool = ((value = false, options) => spec("bool", value, options)) as typeof Api.bool;
export const Vector2 = ((value = vector2(), options) => spec("Vector2", value, options)) as typeof Api.Vector2;
export const Vector3 = ((value = vector3(), options) => spec("Vector3", value, options)) as typeof Api.Vector3;
export const Quaternion = ((value = { x: 0, y: 0, z: 0, w: 1 }, options) => spec("Quaternion", value, options)) as typeof Api.Quaternion;
export const Matrix4 = ((value = { elements: identity() }, options) => spec("Matrix4", value, options)) as typeof Api.Matrix4;
export const Color = ((value = { r: 0, g: 0, b: 0 }, options) => spec("Color", value, options)) as typeof Api.Color;
export const Entity = ((value = 0 as Api.Entity, options) => spec("Entity", value, options)) as typeof Api.Entity;
export const Geometry = ((value = 0 as Api.Geometry, options) => spec("Geometry", value, options)) as typeof Api.Geometry;
export const Material = ((value = 0 as Api.Material, options) => spec("Material", value, options)) as typeof Api.Material;

export const defineComponent: typeof Api.defineComponent = (name, fields) =>
  defineRecord("component", name, fields as unknown as Record<string, FieldSpec>) as never;

export const defineResource: typeof Api.defineResource = (name, fields) =>
  defineRecord("resource", name, fields as unknown as Record<string, FieldSpec>) as never;

export const defineSystem: typeof Api.defineSystem = (name, desc, run) =>
  defineSystemImpl(name, desc as never, run) as never;

export const PROBE_GLOBAL: typeof Api.PROBE_GLOBAL = "__runtime_probe__";
export const PROBE_VERSION: typeof Api.PROBE_VERSION = 5;
export const PROBE_WIRE_VERSION: typeof Api.PROBE_WIRE_VERSION = 1;
export const REPLAY_SEED_PARAM: typeof Api.REPLAY_SEED_PARAM = "replay-seed";
export const PLAYTEST_MAX_TICKS: typeof Api.PLAYTEST_MAX_TICKS = 3600;
export const PROBE_PROGRAM_MAX_OPERATIONS: typeof Api.PROBE_PROGRAM_MAX_OPERATIONS = 256;

export const og: typeof Api.og = Object.freeze({
  ...BUILTINS,
  game: (desc?: Api.GameDesc) => defineGame(desc as never),
  transform,
  render,
  load,
  commands,
}) as unknown as typeof Api.og;
