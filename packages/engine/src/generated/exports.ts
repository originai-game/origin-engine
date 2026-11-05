// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// The column factories and constants the IDL declares, re-exported by src/index.ts.
// A factory's default is its kind's zero value: an enum's first variant, a null handle.

import type * as Api from "@origingame/engine";
import { fieldSpec as spec } from "../runtime.ts";

// reflect
export const f32 = ((value = 0, options) => spec("f32", value, options)) as typeof Api.f32;
export const f64 = ((value = 0, options) => spec("f64", value, options)) as typeof Api.f64;
export const u32 = ((value = 0, options) => spec("u32", value, options)) as typeof Api.u32;
export const i32 = ((value = 0, options) => spec("i32", value, options)) as typeof Api.i32;
export const bool = ((value = false, options) => spec("bool", value, options)) as typeof Api.bool;
export const Vector2 = ((value = { x: 0, y: 0 }, options) => spec("Vector2", value, options)) as typeof Api.Vector2;
export const Vector3 = ((value = { x: 0, y: 0, z: 0 }, options) => spec("Vector3", value, options)) as typeof Api.Vector3;
export const Quaternion = ((value = { x: 0, y: 0, z: 0, w: 1 }, options) => spec("Quaternion", value, options)) as typeof Api.Quaternion;
export const Matrix4 = ((value = { elements: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] }, options) => spec("Matrix4", value, options)) as typeof Api.Matrix4;
export const Color = ((value = { r: 0, g: 0, b: 0 }, options) => spec("Color", value, options)) as typeof Api.Color;

// world
export const Entity = ((value = 0 as Api.Entity, options) => spec("Entity", value, options)) as typeof Api.Entity;

// transform
export const WorldOriginPlane = ((value = "xz" as Api.WorldOriginPlane, options) => spec("WorldOriginPlane", value, options)) as typeof Api.WorldOriginPlane;

// input
export const MAX_POINTERS: typeof Api.MAX_POINTERS = 5;
export const MAX_GAMEPADS: typeof Api.MAX_GAMEPADS = 8;

// render
export const Geometry = ((value = 0 as Api.Geometry, options) => spec("Geometry", value, options)) as typeof Api.Geometry;
export const Material = ((value = 0 as Api.Material, options) => spec("Material", value, options)) as typeof Api.Material;
export const Upscaler = ((value = "taau" as Api.Upscaler, options) => spec("Upscaler", value, options)) as typeof Api.Upscaler;
export const ToneMapping = ((value = "aces" as Api.ToneMapping, options) => spec("ToneMapping", value, options)) as typeof Api.ToneMapping;
export const AoResolutionSetting = ((value = "postEffects" as Api.AoResolutionSetting, options) => spec("AoResolutionSetting", value, options)) as typeof Api.AoResolutionSetting;
export const QualityPreset = ((value = "none" as Api.QualityPreset, options) => spec("QualityPreset", value, options)) as typeof Api.QualityPreset;
export const Texture = ((value = 0 as Api.Texture, options) => spec("Texture", value, options)) as typeof Api.Texture;
export const Lut = ((value = 0 as Api.Lut, options) => spec("Lut", value, options)) as typeof Api.Lut;
export const EnvironmentMap = ((value = 0 as Api.EnvironmentMap, options) => spec("EnvironmentMap", value, options)) as typeof Api.EnvironmentMap;
export const ParticleEmitter = ((value = 0 as Api.ParticleEmitter, options) => spec("ParticleEmitter", value, options)) as typeof Api.ParticleEmitter;
export const AoMethod = ((value = "auto" as Api.AoMethod, options) => spec("AoMethod", value, options)) as typeof Api.AoMethod;
export const AoResolution = ((value = "full" as Api.AoResolution, options) => spec("AoResolution", value, options)) as typeof Api.AoResolution;
export const DofQuality = ((value = "low" as Api.DofQuality, options) => spec("DofQuality", value, options)) as typeof Api.DofQuality;
export const ScreenBlur = ((value = "none" as Api.ScreenBlur, options) => spec("ScreenBlur", value, options)) as typeof Api.ScreenBlur;
export const SpriteBlend = ((value = "alpha" as Api.SpriteBlend, options) => spec("SpriteBlend", value, options)) as typeof Api.SpriteBlend;
export const SpriteFacing = ((value = "none" as Api.SpriteFacing, options) => spec("SpriteFacing", value, options)) as typeof Api.SpriteFacing;
export const StripBlend = ((value = "additive" as Api.StripBlend, options) => spec("StripBlend", value, options)) as typeof Api.StripBlend;
export const GiMethod = ((value = "screen" as Api.GiMethod, options) => spec("GiMethod", value, options)) as typeof Api.GiMethod;
export const Palette = ((value = 0 as Api.Palette, options) => spec("Palette", value, options)) as typeof Api.Palette;
export const Look = ((value = 0 as Api.Look, options) => spec("Look", value, options)) as typeof Api.Look;
export const DitherPattern = ((value = "none" as Api.DitherPattern, options) => spec("DitherPattern", value, options)) as typeof Api.DitherPattern;
export const MaterialSurface = ((value = "none" as Api.MaterialSurface, options) => spec("MaterialSurface", value, options)) as typeof Api.MaterialSurface;
export const OutlinePlace = ((value = "inside" as Api.OutlinePlace, options) => spec("OutlinePlace", value, options)) as typeof Api.OutlinePlace;
export const StyliseSpace = ((value = "screen" as Api.StyliseSpace, options) => spec("StyliseSpace", value, options)) as typeof Api.StyliseSpace;
export const HalftoneMode = ((value = "ink" as Api.HalftoneMode, options) => spec("HalftoneMode", value, options)) as typeof Api.HalftoneMode;
export const HalftoneRegion = ((value = "all" as Api.HalftoneRegion, options) => spec("HalftoneRegion", value, options)) as typeof Api.HalftoneRegion;
export const Light2DKind = ((value = "point" as Api.Light2DKind, options) => spec("Light2DKind", value, options)) as typeof Api.Light2DKind;
export const ReflectionProbeMode = ((value = "baked" as Api.ReflectionProbeMode, options) => spec("ReflectionProbeMode", value, options)) as typeof Api.ReflectionProbeMode;
export const Shader = ((value = 0 as Api.Shader, options) => spec("Shader", value, options)) as typeof Api.Shader;
export const LineMode = ((value = "strip" as Api.LineMode, options) => spec("LineMode", value, options)) as typeof Api.LineMode;
export const RenderTargetFormat = ((value = "ldr" as Api.RenderTargetFormat, options) => spec("RenderTargetFormat", value, options)) as typeof Api.RenderTargetFormat;
export const RenderTarget = ((value = 0 as Api.RenderTarget, options) => spec("RenderTarget", value, options)) as typeof Api.RenderTarget;
export const Post = ((value = 0 as Api.Post, options) => spec("Post", value, options)) as typeof Api.Post;
export const PARTICLES_ARMED: typeof Api.PARTICLES_ARMED = 4294967295;
export const MAX_LIGHTS: typeof Api.MAX_LIGHTS = 256;
export const MAX_CLUSTER_LIGHTS: typeof Api.MAX_CLUSTER_LIGHTS = 64;
export const UI_LAYER: typeof Api.UI_LAYER = 1000;
export const MAX_REFLECTION_PROBES: typeof Api.MAX_REFLECTION_PROBES = 8;

// load
export const TextureArray = ((value = 0 as Api.TextureArray, options) => spec("TextureArray", value, options)) as typeof Api.TextureArray;
export const Asset = ((value = 0 as Api.Asset, options) => spec("Asset", value, options)) as typeof Api.Asset;
export const Sound = ((value = 0 as Api.Sound, options) => spec("Sound", value, options)) as typeof Api.Sound;
export const Font = ((value = 0 as Api.Font, options) => spec("Font", value, options)) as typeof Api.Font;
export const Style = ((value = 0 as Api.Style, options) => spec("Style", value, options)) as typeof Api.Style;

// physics
export const MAX_RAYCAST_HITS: typeof Api.MAX_RAYCAST_HITS = 256;
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
export const SHAPE_DIAMOND: typeof Api.SHAPE_DIAMOND = 7;
export const SHAPE_HEXAGON: typeof Api.SHAPE_HEXAGON = 8;
export const PROFILE_2D: typeof Api.PROFILE_2D = 2;
export const PROFILE_3D: typeof Api.PROFILE_3D = 3;
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
export const VEHICLE_ARCADE: typeof Api.VEHICLE_ARCADE = 0;
export const VEHICLE_DRIVETRAIN: typeof Api.VEHICLE_DRIVETRAIN = 1;
export const SIMULATION_GAMEPLAY: typeof Api.SIMULATION_GAMEPLAY = 0;
export const SIMULATION_VISUAL: typeof Api.SIMULATION_VISUAL = 1;
export const PIN_TOP: typeof Api.PIN_TOP = 1;
export const PIN_TOP_CORNERS: typeof Api.PIN_TOP_CORNERS = 2;
export const PIN_LEFT: typeof Api.PIN_LEFT = 4;
export const PIN_TOP_RINGS: typeof Api.PIN_TOP_RINGS = 8;
export const SOFT_SPHERE: typeof Api.SOFT_SPHERE = 0;
export const SOFT_BOX: typeof Api.SOFT_BOX = 1;
export const SOFT_MESH: typeof Api.SOFT_MESH = 2;

// atmosphere
export const WindMotorKind = ((value = "omni" as Api.WindMotorKind, options) => spec("WindMotorKind", value, options)) as typeof Api.WindMotorKind;
export const WEATHER_NONE: typeof Api.WEATHER_NONE = 0;
export const WEATHER_RAIN: typeof Api.WEATHER_RAIN = 1;
export const WEATHER_SNOW: typeof Api.WEATHER_SNOW = 2;
export const WEATHER_DUST: typeof Api.WEATHER_DUST = 3;
export const WEATHER_MIST: typeof Api.WEATHER_MIST = 4;

// anim
export const SkeletonAsset = ((value = 0 as Api.SkeletonAsset, options) => spec("SkeletonAsset", value, options)) as typeof Api.SkeletonAsset;
export const ClipAsset = ((value = 0 as Api.ClipAsset, options) => spec("ClipAsset", value, options)) as typeof Api.ClipAsset;
export const GraphAsset = ((value = 0 as Api.GraphAsset, options) => spec("GraphAsset", value, options)) as typeof Api.GraphAsset;
export const RootMotion = ((value = "off" as Api.RootMotion, options) => spec("RootMotion", value, options)) as typeof Api.RootMotion;
export const BlendshapeSource = ((value = "component" as Api.BlendshapeSource, options) => spec("BlendshapeSource", value, options)) as typeof Api.BlendshapeSource;
export const IkSolver = ((value = "twoBone" as Api.IkSolver, options) => spec("IkSolver", value, options)) as typeof Api.IkSolver;
export const IkSpace = ((value = "world" as Api.IkSpace, options) => spec("IkSpace", value, options)) as typeof Api.IkSpace;
export const SpringSpace = ((value = "world" as Api.SpringSpace, options) => spec("SpringSpace", value, options)) as typeof Api.SpringSpace;
export const SpringColliderShape = ((value = "sphere" as Api.SpringColliderShape, options) => spec("SpringColliderShape", value, options)) as typeof Api.SpringColliderShape;
export const Rig = ((value = 0 as Api.Rig, options) => spec("Rig", value, options)) as typeof Api.Rig;
export const FlipbookSheet = ((value = 0 as Api.FlipbookSheet, options) => spec("FlipbookSheet", value, options)) as typeof Api.FlipbookSheet;
export const Rig2D = ((value = 0 as Api.Rig2D, options) => spec("Rig2D", value, options)) as typeof Api.Rig2D;

// tween
export const Tween = ((value = 0 as Api.Tween, options) => spec("Tween", value, options)) as typeof Api.Tween;
export const Ease = ((value = "quadOut" as Api.Ease, options) => spec("Ease", value, options)) as typeof Api.Ease;

// tilemap
export const TilemapAsset = ((value = 0 as Api.TilemapAsset, options) => spec("TilemapAsset", value, options)) as typeof Api.TilemapAsset;

// audio
export const AudioBus = ((value = "master" as Api.AudioBus, options) => spec("AudioBus", value, options)) as typeof Api.AudioBus;
export const AudioResampler = ((value = "linear" as Api.AudioResampler, options) => spec("AudioResampler", value, options)) as typeof Api.AudioResampler;
export const PanningModel = ((value = "equalpower" as Api.PanningModel, options) => spec("PanningModel", value, options)) as typeof Api.PanningModel;
export const AudioEventOrder = ((value = "random" as Api.AudioEventOrder, options) => spec("AudioEventOrder", value, options)) as typeof Api.AudioEventOrder;
export const SOUND_FINISHED: typeof Api.SOUND_FINISHED = 0;
export const SOUND_STOPPED: typeof Api.SOUND_STOPPED = 1;
export const SOUND_REMOVED: typeof Api.SOUND_REMOVED = 2;
export const SOUND_REPLACED: typeof Api.SOUND_REPLACED = 3;
export const SOUND_CULLED: typeof Api.SOUND_CULLED = 4;
export const SOUND_RETRIGGERED: typeof Api.SOUND_RETRIGGERED = 5;

// nav
export const Navigation = ((value = 0 as Api.Navigation, options) => spec("Navigation", value, options)) as typeof Api.Navigation;
export const NAV_IDLE: typeof Api.NAV_IDLE = 0;
export const NAV_MOVING: typeof Api.NAV_MOVING = 1;
export const NAV_ARRIVED: typeof Api.NAV_ARRIVED = 2;
export const NAV_UNREACHABLE: typeof Api.NAV_UNREACHABLE = 3;
export const NAV_PARTIAL: typeof Api.NAV_PARTIAL = 4;
export const NAV_ON_LINK: typeof Api.NAV_ON_LINK = 5;

// ai
export const Behaviour = ((value = 0 as Api.Behaviour, options) => spec("Behaviour", value, options)) as typeof Api.Behaviour;
export const AI_RUNNING: typeof Api.AI_RUNNING = 1;
export const AI_SUCCESS: typeof Api.AI_SUCCESS = 2;
export const AI_FAILURE: typeof Api.AI_FAILURE = 3;
export const AI_INTERRUPTED: typeof Api.AI_INTERRUPTED = 4;

// ui
export const VectorArt = ((value = 0 as Api.VectorArt, options) => spec("VectorArt", value, options)) as typeof Api.VectorArt;
export const UiMaterial = ((value = 0 as Api.UiMaterial, options) => spec("UiMaterial", value, options)) as typeof Api.UiMaterial;

// probe
export const PROBE_GLOBAL: typeof Api.PROBE_GLOBAL = "__runtime_probe__";
export const PROBE_VERSION: typeof Api.PROBE_VERSION = 5;
export const PROBE_WIRE_VERSION: typeof Api.PROBE_WIRE_VERSION = 1;
export const REPLAY_SEED_PARAM: typeof Api.REPLAY_SEED_PARAM = "replay-seed";
export const PLAYTEST_MAX_TICKS: typeof Api.PLAYTEST_MAX_TICKS = 3600;
export const PROBE_PROGRAM_MAX_OPERATIONS: typeof Api.PROBE_PROGRAM_MAX_OPERATIONS = 256;

// terrain
export const TerrainData = ((value = 0 as Api.TerrainData, options) => spec("TerrainData", value, options)) as typeof Api.TerrainData;

// stream
export const STREAM_UNLOADED: typeof Api.STREAM_UNLOADED = 0;
export const STREAM_QUEUED: typeof Api.STREAM_QUEUED = 1;
export const STREAM_LOADING: typeof Api.STREAM_LOADING = 2;
export const STREAM_RESIDENT: typeof Api.STREAM_RESIDENT = 3;
export const STREAM_FAILED: typeof Api.STREAM_FAILED = 4;

// video
export const Video = ((value = 0 as Api.Video, options) => spec("Video", value, options)) as typeof Api.Video;

// path
export const Path = ((value = 0 as Api.Path, options) => spec("Path", value, options)) as typeof Api.Path;
export const PathKind = ((value = "catmullRom" as Api.PathKind, options) => spec("PathKind", value, options)) as typeof Api.PathKind;

// timeline
export const TimelineAsset = ((value = 0 as Api.TimelineAsset, options) => spec("TimelineAsset", value, options)) as typeof Api.TimelineAsset;

// net
export const NetMode = ((value = "lockstep" as Api.NetMode, options) => spec("NetMode", value, options)) as typeof Api.NetMode;

// art
export const ArtRotation = ((value = "free" as Api.ArtRotation, options) => spec("ArtRotation", value, options)) as typeof Api.ArtRotation;
export const ArtFilter = ((value = "any" as Api.ArtFilter, options) => spec("ArtFilter", value, options)) as typeof Api.ArtFilter;
export const ArtMips = ((value = "any" as Api.ArtMips, options) => spec("ArtMips", value, options)) as typeof Api.ArtMips;
export const ArtNormals = ((value = "any" as Api.ArtNormals, options) => spec("ArtNormals", value, options)) as typeof Api.ArtNormals;
export const ArtLinePlace = ((value = "outside" as Api.ArtLinePlace, options) => spec("ArtLinePlace", value, options)) as typeof Api.ArtLinePlace;
export const ArtBrush = ((value = "diamond" as Api.ArtBrush, options) => spec("ArtBrush", value, options)) as typeof Api.ArtBrush;
export const ArtProjection = ((value = "any" as Api.ArtProjection, options) => spec("ArtProjection", value, options)) as typeof Api.ArtProjection;

// motion
export const MotionGoalKind = ((value = "none" as Api.MotionGoalKind, options) => spec("MotionGoalKind", value, options)) as typeof Api.MotionGoalKind;

// destruction
export const CAUSE_IMPACT: typeof Api.CAUSE_IMPACT = 0;
export const CAUSE_DAMAGE: typeof Api.CAUSE_DAMAGE = 1;
export const CAUSE_EXPLOSION: typeof Api.CAUSE_EXPLOSION = 2;
export const CAUSE_STRESS: typeof Api.CAUSE_STRESS = 3;
export const CAUSE_SHATTER: typeof Api.CAUSE_SHATTER = 4;
export const MATERIAL_BRICK: typeof Api.MATERIAL_BRICK = 0;
export const MATERIAL_CONCRETE: typeof Api.MATERIAL_CONCRETE = 1;
export const MATERIAL_WOOD: typeof Api.MATERIAL_WOOD = 2;
export const MATERIAL_GLASS: typeof Api.MATERIAL_GLASS = 3;
export const MATERIAL_STONE: typeof Api.MATERIAL_STONE = 4;
export const CAUSE_FRACTURE: typeof Api.CAUSE_FRACTURE = 5;
