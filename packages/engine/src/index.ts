// @origingame/engine: the runtime implementation of the generated
// declarations. Every export is typed by types/index.d.ts (the contract
// generated from crates/og-api/api.ogt), so a runtime member that drifts
// from the IDL fails `bun run typecheck`.

import type * as Api from "@origingame/engine";
import {
  anim,
  BUILTINS,
  commands,
  defineGame,
  defineRecord,
  defineSystem as defineSystemImpl,
  load,
  physics,
  render,
  transform,
  ui,
  type FieldSpec,
} from "./runtime.ts";
import { cameras } from "./systems/cameras.ts";
import { uiKit } from "./ui-kit/index.ts";

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
export const Asset = ((value = 0 as Api.Asset, options) => spec("Asset", value, options)) as typeof Api.Asset;
export const Sound = ((value = 0 as Api.Sound, options) => spec("Sound", value, options)) as typeof Api.Sound;
export const SkeletonAsset = ((value = 0 as Api.SkeletonAsset, options) => spec("SkeletonAsset", value, options)) as typeof Api.SkeletonAsset;
export const ClipAsset = ((value = 0 as Api.ClipAsset, options) => spec("ClipAsset", value, options)) as typeof Api.ClipAsset;
export const GraphAsset = ((value = 0 as Api.GraphAsset, options) => spec("GraphAsset", value, options)) as typeof Api.GraphAsset;
export const RootMotion = ((value = "off" as Api.RootMotion, options) => spec("RootMotion", value, options)) as typeof Api.RootMotion;
export const BlendshapeSource = ((value = "component" as Api.BlendshapeSource, options) => spec("BlendshapeSource", value, options)) as typeof Api.BlendshapeSource;
export const IkSolver = ((value = "twoBone" as Api.IkSolver, options) => spec("IkSolver", value, options)) as typeof Api.IkSolver;
export const IkSpace = ((value = "world" as Api.IkSpace, options) => spec("IkSpace", value, options)) as typeof Api.IkSpace;
export const AudioBus = ((value = "master" as Api.AudioBus, options) => spec("AudioBus", value, options)) as typeof Api.AudioBus;

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
export const MAX_LIGHTS: typeof Api.MAX_LIGHTS = 8;
export const BODY_DYNAMIC: typeof Api.BODY_DYNAMIC = 0;
export const BODY_STATIC: typeof Api.BODY_STATIC = 1;
export const BODY_KINEMATIC: typeof Api.BODY_KINEMATIC = 2;
export const LOCK_TRANSLATION_X: typeof Api.LOCK_TRANSLATION_X = 1;
export const LOCK_TRANSLATION_Y: typeof Api.LOCK_TRANSLATION_Y = 2;
export const LOCK_TRANSLATION_Z: typeof Api.LOCK_TRANSLATION_Z = 4;
export const LOCK_ROTATION_X: typeof Api.LOCK_ROTATION_X = 8;
export const LOCK_ROTATION_Y: typeof Api.LOCK_ROTATION_Y = 16;
export const LOCK_ROTATION_Z: typeof Api.LOCK_ROTATION_Z = 32;
export const SHAPE_BOX: typeof Api.SHAPE_BOX = 0;
export const SHAPE_SPHERE: typeof Api.SHAPE_SPHERE = 1;
export const SHAPE_CAPSULE: typeof Api.SHAPE_CAPSULE = 2;
export const SHAPE_CYLINDER: typeof Api.SHAPE_CYLINDER = 3;
export const SHAPE_CONVEX: typeof Api.SHAPE_CONVEX = 4;
export const SHAPE_TRIMESH: typeof Api.SHAPE_TRIMESH = 5;
export const PROFILE_2D: typeof Api.PROFILE_2D = 2;
export const PROFILE_3D: typeof Api.PROFILE_3D = 3;
export const SOUND_FINISHED: typeof Api.SOUND_FINISHED = 0;
export const SOUND_STOPPED: typeof Api.SOUND_STOPPED = 1;
export const SOUND_REMOVED: typeof Api.SOUND_REMOVED = 2;
export const SOUND_REPLACED: typeof Api.SOUND_REPLACED = 3;

export const og: typeof Api.og = Object.freeze({
  ...BUILTINS,
  game: (desc?: Api.GameDesc) => defineGame(desc as never),
  transform,
  render,
  load,
  commands,
  physics,
  anim,
  ui: Object.freeze({ ...ui, ...uiKit }),
  systems: Object.freeze({ cameras }),
}) as unknown as typeof Api.og;
