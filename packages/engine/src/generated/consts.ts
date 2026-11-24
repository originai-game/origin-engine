// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// The IDL's module constants, as runtime values.
// src/generated/exports.ts re-exports these beside the column factories.

import type * as Api from "@origingame/engine";

// input
export const MAX_POINTERS: typeof Api.MAX_POINTERS = 5;
export const MAX_GAMEPADS: typeof Api.MAX_GAMEPADS = 8;

// camera
export const MAX_CAMERA_COC_SOURCES: typeof Api.MAX_CAMERA_COC_SOURCES = 4;

// render
export const PARTICLES_ARMED: typeof Api.PARTICLES_ARMED = 4294967295;
export const MAX_LIGHTS: typeof Api.MAX_LIGHTS = 256;
export const MAX_CLUSTER_LIGHTS: typeof Api.MAX_CLUSTER_LIGHTS = 64;
export const UI_LAYER: typeof Api.UI_LAYER = 1000;
export const MAX_REFLECTION_PROBES: typeof Api.MAX_REFLECTION_PROBES = 8;

// physics
export const MAX_RAYCAST_HITS: typeof Api.MAX_RAYCAST_HITS = 256;
export const LOCK_TRANSLATION_X: typeof Api.LOCK_TRANSLATION_X = 1;
export const LOCK_TRANSLATION_Y: typeof Api.LOCK_TRANSLATION_Y = 2;
export const LOCK_TRANSLATION_Z: typeof Api.LOCK_TRANSLATION_Z = 4;
export const LOCK_ROTATION_X: typeof Api.LOCK_ROTATION_X = 8;
export const LOCK_ROTATION_Y: typeof Api.LOCK_ROTATION_Y = 16;
export const LOCK_ROTATION_Z: typeof Api.LOCK_ROTATION_Z = 32;
export const PROFILE_2D: typeof Api.PROFILE_2D = 2;
export const PROFILE_3D: typeof Api.PROFILE_3D = 3;
export const CHARACTER_JUMPED: typeof Api.CHARACTER_JUMPED = 1;
export const CHARACTER_LANDED: typeof Api.CHARACTER_LANDED = 2;
export const CHARACTER_WALL_JUMPED: typeof Api.CHARACTER_WALL_JUMPED = 3;
export const CHARACTER_AIR_JUMPED: typeof Api.CHARACTER_AIR_JUMPED = 4;
export const CHARACTER_LEFT_GROUND: typeof Api.CHARACTER_LEFT_GROUND = 5;
export const CHARACTER_BUMPED: typeof Api.CHARACTER_BUMPED = 6;
export const PIN_TOP: typeof Api.PIN_TOP = 1;
export const PIN_TOP_CORNERS: typeof Api.PIN_TOP_CORNERS = 2;
export const PIN_LEFT: typeof Api.PIN_LEFT = 4;
export const PIN_TOP_RINGS: typeof Api.PIN_TOP_RINGS = 8;

// atmosphere
export const WEATHER_NONE: typeof Api.WEATHER_NONE = 0;
export const WEATHER_RAIN: typeof Api.WEATHER_RAIN = 1;
export const WEATHER_SNOW: typeof Api.WEATHER_SNOW = 2;
export const WEATHER_DUST: typeof Api.WEATHER_DUST = 3;
export const WEATHER_MIST: typeof Api.WEATHER_MIST = 4;
export const SEASON_SPRING: typeof Api.SEASON_SPRING = 79;
export const SEASON_SUMMER: typeof Api.SEASON_SUMMER = 172;
export const SEASON_AUTUMN: typeof Api.SEASON_AUTUMN = 265;
export const SEASON_WINTER: typeof Api.SEASON_WINTER = 355;

// audio
export const SOUND_FINISHED: typeof Api.SOUND_FINISHED = 0;
export const SOUND_STOPPED: typeof Api.SOUND_STOPPED = 1;
export const SOUND_REMOVED: typeof Api.SOUND_REMOVED = 2;
export const SOUND_REPLACED: typeof Api.SOUND_REPLACED = 3;
export const SOUND_CULLED: typeof Api.SOUND_CULLED = 4;
export const SOUND_RETRIGGERED: typeof Api.SOUND_RETRIGGERED = 5;

// nav
export const NAV_IDLE: typeof Api.NAV_IDLE = 0;
export const NAV_MOVING: typeof Api.NAV_MOVING = 1;
export const NAV_ARRIVED: typeof Api.NAV_ARRIVED = 2;
export const NAV_UNREACHABLE: typeof Api.NAV_UNREACHABLE = 3;
export const NAV_PARTIAL: typeof Api.NAV_PARTIAL = 4;
export const NAV_ON_LINK: typeof Api.NAV_ON_LINK = 5;

// ai
export const AI_RUNNING: typeof Api.AI_RUNNING = 1;
export const AI_SUCCESS: typeof Api.AI_SUCCESS = 2;
export const AI_FAILURE: typeof Api.AI_FAILURE = 3;
export const AI_INTERRUPTED: typeof Api.AI_INTERRUPTED = 4;

// probe
export const PROBE_GLOBAL: typeof Api.PROBE_GLOBAL = "__runtime_probe__";
export const PROBE_VERSION: typeof Api.PROBE_VERSION = 5;
export const PROBE_WIRE_VERSION: typeof Api.PROBE_WIRE_VERSION = 1;
export const REPLAY_SEED_PARAM: typeof Api.REPLAY_SEED_PARAM = "replay-seed";
export const PLAYTEST_MAX_TICKS: typeof Api.PLAYTEST_MAX_TICKS = 3600;
export const PROBE_PROGRAM_MAX_OPERATIONS: typeof Api.PROBE_PROGRAM_MAX_OPERATIONS = 256;
export const CAPTURE_FIT: typeof Api.CAPTURE_FIT = 1280;

// stream
export const STREAM_UNLOADED: typeof Api.STREAM_UNLOADED = 0;
export const STREAM_QUEUED: typeof Api.STREAM_QUEUED = 1;
export const STREAM_LOADING: typeof Api.STREAM_LOADING = 2;
export const STREAM_RESIDENT: typeof Api.STREAM_RESIDENT = 3;
export const STREAM_FAILED: typeof Api.STREAM_FAILED = 4;

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
export const CAUSE_CARVE: typeof Api.CAUSE_CARVE = 6;
