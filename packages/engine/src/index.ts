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
  atmosphere,
  audio,
  render,
  transform,
  ui,
  time,
  camera,
  tween,
  tilemap,
  type FieldSpec,
} from "./runtime.ts";
import { cameras } from "./systems/cameras.ts";
import { ai, i18n, nav, net, platform } from "./e6.ts";
import { path, stream, terrain, timeline } from "./p2.ts";
import { debug, depthRender } from "./depth.ts";
import { targetsLoad, targetsRender } from "./targets.ts";
import { uiKit } from "./ui-kit/index.ts";

// The math layer (IDL `math`): three.js's methods as pure functions over the plain structs.
export { vec3, vec2, quat, mat4, box3, sphere, plane, frustum, ray, color } from "./math.ts";

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
// The capacity crosses with the spec; the engine refuses one outside 1 to 4096 at declaration.
export const str = ((capacity, value = "", options) => Object.freeze({ ...spec("str", value, options), capacity })) as typeof Api.str;
export const Entity = ((value = 0 as Api.Entity, options) => spec("Entity", value, options)) as typeof Api.Entity;
export const Geometry = ((value = 0 as Api.Geometry, options) => spec("Geometry", value, options)) as typeof Api.Geometry;
export const Material = ((value = 0 as Api.Material, options) => spec("Material", value, options)) as typeof Api.Material;
export const ToneMapping = ((value = "aces" as Api.ToneMapping, options) => spec("ToneMapping", value, options)) as typeof Api.ToneMapping;
export const Texture = ((value = 0 as Api.Texture, options) => spec("Texture", value, options)) as typeof Api.Texture;
export const Lut = ((value = 0 as Api.Lut, options) => spec("Lut", value, options)) as typeof Api.Lut;
export const EnvironmentMap = ((value = 0 as Api.EnvironmentMap, options) => spec("EnvironmentMap", value, options)) as typeof Api.EnvironmentMap;
export const ParticleEmitter = ((value = 0 as Api.ParticleEmitter, options) => spec("ParticleEmitter", value, options)) as typeof Api.ParticleEmitter;
export const AoMethod = ((value = "auto" as Api.AoMethod, options) => spec("AoMethod", value, options)) as typeof Api.AoMethod;
export const LineMode = ((value = "strip" as Api.LineMode, options) => spec("LineMode", value, options)) as typeof Api.LineMode;
export const DofQuality = ((value = "low" as Api.DofQuality, options) => spec("DofQuality", value, options)) as typeof Api.DofQuality;
export const SpriteBlend = ((value = "alpha" as Api.SpriteBlend, options) => spec("SpriteBlend", value, options)) as typeof Api.SpriteBlend;
export const StripBlend = ((value = "additive" as Api.StripBlend, options) => spec("StripBlend", value, options)) as typeof Api.StripBlend;
export const Palette = ((value = 0 as Api.Palette, options) => spec("Palette", value, options)) as typeof Api.Palette;
export const DitherPattern = ((value = "none" as Api.DitherPattern, options) => spec("DitherPattern", value, options)) as typeof Api.DitherPattern;
export const StyliseSpace = ((value = "screen" as Api.StyliseSpace, options) => spec("StyliseSpace", value, options)) as typeof Api.StyliseSpace;
export const TextureArray = ((value = 0 as Api.TextureArray, options) => spec("TextureArray", value, options)) as typeof Api.TextureArray;
export const Asset = ((value = 0 as Api.Asset, options) => spec("Asset", value, options)) as typeof Api.Asset;
export const Sound = ((value = 0 as Api.Sound, options) => spec("Sound", value, options)) as typeof Api.Sound;
export const SkeletonAsset = ((value = 0 as Api.SkeletonAsset, options) => spec("SkeletonAsset", value, options)) as typeof Api.SkeletonAsset;
export const ClipAsset = ((value = 0 as Api.ClipAsset, options) => spec("ClipAsset", value, options)) as typeof Api.ClipAsset;
export const GraphAsset = ((value = 0 as Api.GraphAsset, options) => spec("GraphAsset", value, options)) as typeof Api.GraphAsset;
export const FlipbookSheet = ((value = 0 as Api.FlipbookSheet, options) => spec("FlipbookSheet", value, options)) as typeof Api.FlipbookSheet;
export const Tween = ((value = 0 as Api.Tween, options) => spec("Tween", value, options)) as typeof Api.Tween;
export const Video = ((value = 0 as Api.Video, options) => spec("Video", value, options)) as typeof Api.Video;
export const Path = ((value = 0 as Api.Path, options) => spec("Path", value, options)) as typeof Api.Path;
export const PathKind = ((value = "catmullRom" as Api.PathKind, options) => spec("PathKind", value, options)) as typeof Api.PathKind;
export const TimelineAsset = ((value = 0 as Api.TimelineAsset, options) => spec("TimelineAsset", value, options)) as typeof Api.TimelineAsset;
export const Rig2D = ((value = 0 as Api.Rig2D, options) => spec("Rig2D", value, options)) as typeof Api.Rig2D;
export const RenderTargetFormat = ((value = "ldr" as Api.RenderTargetFormat, options) => spec("RenderTargetFormat", value, options)) as typeof Api.RenderTargetFormat;
export const RenderTarget = ((value = 0 as Api.RenderTarget, options) => spec("RenderTarget", value, options)) as typeof Api.RenderTarget;
export const Post = ((value = 0 as Api.Post, options) => spec("Post", value, options)) as typeof Api.Post;
export const TilemapAsset = ((value = 0 as Api.TilemapAsset, options) => spec("TilemapAsset", value, options)) as typeof Api.TilemapAsset;
export const TerrainData = ((value = 0 as Api.TerrainData, options) => spec("TerrainData", value, options)) as typeof Api.TerrainData;
export const RootMotion = ((value = "off" as Api.RootMotion, options) => spec("RootMotion", value, options)) as typeof Api.RootMotion;
export const BlendshapeSource = ((value = "component" as Api.BlendshapeSource, options) => spec("BlendshapeSource", value, options)) as typeof Api.BlendshapeSource;
export const IkSolver = ((value = "twoBone" as Api.IkSolver, options) => spec("IkSolver", value, options)) as typeof Api.IkSolver;
export const IkSpace = ((value = "world" as Api.IkSpace, options) => spec("IkSpace", value, options)) as typeof Api.IkSpace;
export const AudioBus = ((value = "master" as Api.AudioBus, options) => spec("AudioBus", value, options)) as typeof Api.AudioBus;
export const Navigation = ((value = 0 as Api.Navigation, options) => spec("Navigation", value, options)) as typeof Api.Navigation;
export const Behaviour = ((value = 0 as Api.Behaviour, options) => spec("Behaviour", value, options)) as typeof Api.Behaviour;

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
export const MAX_POINTERS: typeof Api.MAX_POINTERS = 5;
export const MAX_GAMEPADS: typeof Api.MAX_GAMEPADS = 4;
export const MAX_LIGHTS: typeof Api.MAX_LIGHTS = 256;
export const MAX_CLUSTER_LIGHTS: typeof Api.MAX_CLUSTER_LIGHTS = 64;
export const PARTICLES_ARMED: typeof Api.PARTICLES_ARMED = 4294967295;
export const UI_LAYER: typeof Api.UI_LAYER = 1000;
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
export const SHAPE_HEIGHTFIELD: typeof Api.SHAPE_HEIGHTFIELD = 6;
export const PROFILE_2D: typeof Api.PROFILE_2D = 2;
export const PROFILE_3D: typeof Api.PROFILE_3D = 3;
export const SOUND_FINISHED: typeof Api.SOUND_FINISHED = 0;
export const SOUND_STOPPED: typeof Api.SOUND_STOPPED = 1;
export const SOUND_REMOVED: typeof Api.SOUND_REMOVED = 2;
export const SOUND_REPLACED: typeof Api.SOUND_REPLACED = 3;
export const SOUND_CULLED: typeof Api.SOUND_CULLED = 4;
export const NAV_IDLE: typeof Api.NAV_IDLE = 0;
export const NAV_MOVING: typeof Api.NAV_MOVING = 1;
export const NAV_ARRIVED: typeof Api.NAV_ARRIVED = 2;
export const NAV_UNREACHABLE: typeof Api.NAV_UNREACHABLE = 3;
export const NAV_PARTIAL: typeof Api.NAV_PARTIAL = 4;
export const AI_RUNNING: typeof Api.AI_RUNNING = 1;
export const AI_SUCCESS: typeof Api.AI_SUCCESS = 2;
export const AI_FAILURE: typeof Api.AI_FAILURE = 3;

// The E5 round: physics beyond rigid bodies and the atmosphere.
export const WEATHER_NONE: typeof Api.WEATHER_NONE = 0;
export const WEATHER_RAIN: typeof Api.WEATHER_RAIN = 1;
export const WEATHER_SNOW: typeof Api.WEATHER_SNOW = 2;
export const WEATHER_DUST: typeof Api.WEATHER_DUST = 3;
export const WEATHER_MIST: typeof Api.WEATHER_MIST = 4;
export const JOINT_FIXED: typeof Api.JOINT_FIXED = 0;
export const JOINT_REVOLUTE: typeof Api.JOINT_REVOLUTE = 1;
export const JOINT_PRISMATIC: typeof Api.JOINT_PRISMATIC = 2;
export const JOINT_SPRING: typeof Api.JOINT_SPRING = 3;
export const JOINT_ROPE: typeof Api.JOINT_ROPE = 4;
export const JOINT_BALL: typeof Api.JOINT_BALL = 5;
export const MOTOR_NONE: typeof Api.MOTOR_NONE = 0;
export const MOTOR_VELOCITY: typeof Api.MOTOR_VELOCITY = 1;
export const MOTOR_POSITION: typeof Api.MOTOR_POSITION = 2;
export const CHARACTER_JUMPED: typeof Api.CHARACTER_JUMPED = 1;
export const CHARACTER_LANDED: typeof Api.CHARACTER_LANDED = 2;
export const CHARACTER_WALL_JUMPED: typeof Api.CHARACTER_WALL_JUMPED = 3;
export const CHARACTER_AIR_JUMPED: typeof Api.CHARACTER_AIR_JUMPED = 4;
export const CHARACTER_LEFT_GROUND: typeof Api.CHARACTER_LEFT_GROUND = 5;
export const CHARACTER_BUMPED: typeof Api.CHARACTER_BUMPED = 6;
export const DRIVE_REAR: typeof Api.DRIVE_REAR = 0;
export const DRIVE_FRONT: typeof Api.DRIVE_FRONT = 1;
export const DRIVE_ALL: typeof Api.DRIVE_ALL = 2;
export const PIN_TOP: typeof Api.PIN_TOP = 1;
export const PIN_TOP_CORNERS: typeof Api.PIN_TOP_CORNERS = 2;
export const PIN_LEFT: typeof Api.PIN_LEFT = 4;
export const PIN_TOP_RINGS: typeof Api.PIN_TOP_RINGS = 8;
export const STREAM_UNLOADED: typeof Api.STREAM_UNLOADED = 0;
export const STREAM_QUEUED: typeof Api.STREAM_QUEUED = 1;
export const STREAM_LOADING: typeof Api.STREAM_LOADING = 2;
export const STREAM_RESIDENT: typeof Api.STREAM_RESIDENT = 3;
export const STREAM_FAILED: typeof Api.STREAM_FAILED = 4;

export const og: typeof Api.og = Object.freeze({
  ...BUILTINS,
  game: (desc?: Api.GameDesc) => defineGame(desc as never),
  transform,
  render: Object.freeze({ ...render, ...depthRender, ...targetsRender }),
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
  platform,
  i18n,
  net,
  terrain,
  stream,
  debug,
  path,
  timeline,
  ui: Object.freeze({ ...ui, ...uiKit }),
  systems: Object.freeze({ cameras }),
}) as unknown as typeof Api.og;
