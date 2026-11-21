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
export { MAX_POINTERS, MAX_GAMEPADS } from "./consts.ts";

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
export const EffectMaterial = ((value = 0 as Api.EffectMaterial, options) => spec("EffectMaterial", value, options)) as typeof Api.EffectMaterial;
export const AoMethod = ((value = "auto" as Api.AoMethod, options) => spec("AoMethod", value, options)) as typeof Api.AoMethod;
export const AoResolution = ((value = "full" as Api.AoResolution, options) => spec("AoResolution", value, options)) as typeof Api.AoResolution;
export const DofQuality = ((value = "low" as Api.DofQuality, options) => spec("DofQuality", value, options)) as typeof Api.DofQuality;
export const ScreenBlur = ((value = "none" as Api.ScreenBlur, options) => spec("ScreenBlur", value, options)) as typeof Api.ScreenBlur;
export const Blend = ((value = "alpha" as Api.Blend, options) => spec("Blend", value, options)) as typeof Api.Blend;
export const SpriteFacing = ((value = "none" as Api.SpriteFacing, options) => spec("SpriteFacing", value, options)) as typeof Api.SpriteFacing;
export const GiMethod = ((value = "screen" as Api.GiMethod, options) => spec("GiMethod", value, options)) as typeof Api.GiMethod;
export const Palette = ((value = 0 as Api.Palette, options) => spec("Palette", value, options)) as typeof Api.Palette;
export const Look = ((value = 0 as Api.Look, options) => spec("Look", value, options)) as typeof Api.Look;
export const DitherPattern = ((value = "none" as Api.DitherPattern, options) => spec("DitherPattern", value, options)) as typeof Api.DitherPattern;
export const MaterialSurface = ((value = "none" as Api.MaterialSurface, options) => spec("MaterialSurface", value, options)) as typeof Api.MaterialSurface;
export const OutlinePlace = ((value = "inside" as Api.OutlinePlace, options) => spec("OutlinePlace", value, options)) as typeof Api.OutlinePlace;
export const StyliseSpace = ((value = "screen" as Api.StyliseSpace, options) => spec("StyliseSpace", value, options)) as typeof Api.StyliseSpace;
export const PosterizeMode = ((value = "channel" as Api.PosterizeMode, options) => spec("PosterizeMode", value, options)) as typeof Api.PosterizeMode;
export const HalftoneMode = ((value = "ink" as Api.HalftoneMode, options) => spec("HalftoneMode", value, options)) as typeof Api.HalftoneMode;
export const HalftoneRegion = ((value = "all" as Api.HalftoneRegion, options) => spec("HalftoneRegion", value, options)) as typeof Api.HalftoneRegion;
export const Light2DKind = ((value = "point" as Api.Light2DKind, options) => spec("Light2DKind", value, options)) as typeof Api.Light2DKind;
export const ReflectionProbeMode = ((value = "baked" as Api.ReflectionProbeMode, options) => spec("ReflectionProbeMode", value, options)) as typeof Api.ReflectionProbeMode;
export const Shader = ((value = 0 as Api.Shader, options) => spec("Shader", value, options)) as typeof Api.Shader;
export const LineMode = ((value = "strip" as Api.LineMode, options) => spec("LineMode", value, options)) as typeof Api.LineMode;
export const RenderTargetFormat = ((value = "ldr" as Api.RenderTargetFormat, options) => spec("RenderTargetFormat", value, options)) as typeof Api.RenderTargetFormat;
export const RenderTarget = ((value = 0 as Api.RenderTarget, options) => spec("RenderTarget", value, options)) as typeof Api.RenderTarget;
export const Post = ((value = 0 as Api.Post, options) => spec("Post", value, options)) as typeof Api.Post;
export { PARTICLES_ARMED, MAX_LIGHTS, MAX_CLUSTER_LIGHTS, UI_LAYER, MAX_REFLECTION_PROBES } from "./consts.ts";

// load
export const TextureArray = ((value = 0 as Api.TextureArray, options) => spec("TextureArray", value, options)) as typeof Api.TextureArray;
export const Asset = ((value = 0 as Api.Asset, options) => spec("Asset", value, options)) as typeof Api.Asset;
export const Sound = ((value = 0 as Api.Sound, options) => spec("Sound", value, options)) as typeof Api.Sound;
export const Font = ((value = 0 as Api.Font, options) => spec("Font", value, options)) as typeof Api.Font;
export const Style = ((value = 0 as Api.Style, options) => spec("Style", value, options)) as typeof Api.Style;

// physics
export const RigidBodyType = ((value = "dynamic" as Api.RigidBodyType, options) => spec("RigidBodyType", value, options)) as typeof Api.RigidBodyType;
export const ColliderShape = ((value = "box" as Api.ColliderShape, options) => spec("ColliderShape", value, options)) as typeof Api.ColliderShape;
export const ForceFieldKind = ((value = "directional" as Api.ForceFieldKind, options) => spec("ForceFieldKind", value, options)) as typeof Api.ForceFieldKind;
export const JointType = ((value = "fixed" as Api.JointType, options) => spec("JointType", value, options)) as typeof Api.JointType;
export const JointMotor = ((value = "none" as Api.JointMotor, options) => spec("JointMotor", value, options)) as typeof Api.JointMotor;
export const VehicleDrive = ((value = "rear" as Api.VehicleDrive, options) => spec("VehicleDrive", value, options)) as typeof Api.VehicleDrive;
export const VehicleModel = ((value = "arcade" as Api.VehicleModel, options) => spec("VehicleModel", value, options)) as typeof Api.VehicleModel;
export const Simulation = ((value = "gameplay" as Api.Simulation, options) => spec("Simulation", value, options)) as typeof Api.Simulation;
export const SoftBodyShape = ((value = "sphere" as Api.SoftBodyShape, options) => spec("SoftBodyShape", value, options)) as typeof Api.SoftBodyShape;
export { MAX_RAYCAST_HITS, LOCK_TRANSLATION_X, LOCK_TRANSLATION_Y, LOCK_TRANSLATION_Z, LOCK_ROTATION_X, LOCK_ROTATION_Y, LOCK_ROTATION_Z, PROFILE_2D, PROFILE_3D, CHARACTER_JUMPED, CHARACTER_LANDED, CHARACTER_WALL_JUMPED, CHARACTER_AIR_JUMPED, CHARACTER_LEFT_GROUND, CHARACTER_BUMPED, PIN_TOP, PIN_TOP_CORNERS, PIN_LEFT, PIN_TOP_RINGS } from "./consts.ts";

// atmosphere
export const WindMotorKind = ((value = "omni" as Api.WindMotorKind, options) => spec("WindMotorKind", value, options)) as typeof Api.WindMotorKind;
export { WEATHER_NONE, WEATHER_RAIN, WEATHER_SNOW, WEATHER_DUST, WEATHER_MIST, SEASON_SPRING, SEASON_SUMMER, SEASON_AUTUMN, SEASON_WINTER } from "./consts.ts";

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
export const Ease = ((value = "power1.out" as Api.ColumnKinds["Ease"]["value"], options) => spec("Ease", value, options)) as typeof Api.Ease;

// tilemap
export const TilemapAsset = ((value = 0 as Api.TilemapAsset, options) => spec("TilemapAsset", value, options)) as typeof Api.TilemapAsset;

// audio
export const AudioBus = ((value = "master" as Api.AudioBus, options) => spec("AudioBus", value, options)) as typeof Api.AudioBus;
export const AudioResampler = ((value = "linear" as Api.AudioResampler, options) => spec("AudioResampler", value, options)) as typeof Api.AudioResampler;
export const PanningModel = ((value = "equalpower" as Api.PanningModel, options) => spec("PanningModel", value, options)) as typeof Api.PanningModel;
export const AudioEventOrder = ((value = "random" as Api.AudioEventOrder, options) => spec("AudioEventOrder", value, options)) as typeof Api.AudioEventOrder;
export { SOUND_FINISHED, SOUND_STOPPED, SOUND_REMOVED, SOUND_REPLACED, SOUND_CULLED, SOUND_RETRIGGERED } from "./consts.ts";

// nav
export const Navigation = ((value = 0 as Api.Navigation, options) => spec("Navigation", value, options)) as typeof Api.Navigation;
export { NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE, NAV_PARTIAL, NAV_ON_LINK } from "./consts.ts";

// ai
export const Behaviour = ((value = 0 as Api.Behaviour, options) => spec("Behaviour", value, options)) as typeof Api.Behaviour;
export { AI_RUNNING, AI_SUCCESS, AI_FAILURE, AI_INTERRUPTED } from "./consts.ts";

// ui
export const VectorArt = ((value = 0 as Api.VectorArt, options) => spec("VectorArt", value, options)) as typeof Api.VectorArt;
export const UiMaterial = ((value = 0 as Api.UiMaterial, options) => spec("UiMaterial", value, options)) as typeof Api.UiMaterial;

// probe
export { PROBE_GLOBAL, PROBE_VERSION, PROBE_WIRE_VERSION, REPLAY_SEED_PARAM, PLAYTEST_MAX_TICKS, PROBE_PROGRAM_MAX_OPERATIONS, CAPTURE_FIT } from "./consts.ts";

// terrain
export const TerrainData = ((value = 0 as Api.TerrainData, options) => spec("TerrainData", value, options)) as typeof Api.TerrainData;

// stream
export { STREAM_UNLOADED, STREAM_QUEUED, STREAM_LOADING, STREAM_RESIDENT, STREAM_FAILED } from "./consts.ts";

// video
export const Video = ((value = 0 as Api.Video, options) => spec("Video", value, options)) as typeof Api.Video;

// path
export const Path = ((value = 0 as Api.Path, options) => spec("Path", value, options)) as typeof Api.Path;
export const PathKind = ((value = "catmullRom" as Api.PathKind, options) => spec("PathKind", value, options)) as typeof Api.PathKind;

// timeline
export const TimelineAsset = ((value = 0 as Api.TimelineAsset, options) => spec("TimelineAsset", value, options)) as typeof Api.TimelineAsset;

// net
export const NetMode = ((value = "lockstep" as Api.NetMode, options) => spec("NetMode", value, options)) as typeof Api.NetMode;

// shared
export const SharedState = ((value = "off" as Api.SharedState, options) => spec("SharedState", value, options)) as typeof Api.SharedState;

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
export { CAUSE_IMPACT, CAUSE_DAMAGE, CAUSE_EXPLOSION, CAUSE_STRESS, CAUSE_SHATTER, MATERIAL_BRICK, MATERIAL_CONCRETE, MATERIAL_WOOD, MATERIAL_GLASS, MATERIAL_STONE, CAUSE_FRACTURE, CAUSE_CARVE } from "./consts.ts";

// vfx
export const VfxAsset = ((value = 0 as Api.VfxAsset, options) => spec("VfxAsset", value, options)) as typeof Api.VfxAsset;

// sight
export const SightPlane = ((value = "ground" as Api.SightPlane, options) => spec("SightPlane", value, options)) as typeof Api.SightPlane;
