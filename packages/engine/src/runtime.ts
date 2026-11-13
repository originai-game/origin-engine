// The TypeScript runtime behind the generated declarations (types/index.d.ts):
// tokens, declarations, the dispatcher the engine calls, and the world, query
// and view objects game code is handed. Host-agnostic: a loader binds it to
// the web host (src/web/) or the native host (src/node/) with `bindHost`.
//
// Every engine call goes through the binding symbols of the generated ABI
// (src/generated/abi.ts). Structured values cross as JSON text; column views
// cross as whatever the host aliases engine memory with, turned into typed
// arrays by the binding's `columns`.
//
// The runtime is the modules of src/runtime/; this file is what the loaders
// and the package's `./runtime` export import, and it exports what they may use.

export { COLUMNS, ARRAYS, bindHost, boundHost } from "./runtime/host.ts";
export type { ColumnArray, HostBinding, ColumnSpec } from "./runtime/host.ts";
export type { CursorImage, Result } from "./runtime/result.ts";
export { fieldSpec, defineRecord, BUILTINS, defineHook, defineSystem, declaredGameExplore, declaredGameLook, engineGame, defineGame, registerCommand } from "./runtime/declare.ts";
export type { FieldSpec, Token, SystemToken, SystemDescIn, HookToken, HookDescIn, GameToken, GameDescIn, SaveMigrationIn } from "./runtime/declare.ts";
export { currentCall, isPublishBuild } from "./runtime/views.ts";
export { WorldImpl, SpatialIndexImpl, SaveEditImpl } from "./runtime/world.ts";
export { beginBundle, systemClock, fixedClock, Engine, createEngine } from "./runtime/engine.ts";
export type { CreateOptions, CommandOutput, LogLine, HostClock } from "./runtime/engine.ts";
export { transform, render, load, anim, time, camera, tween, tilemap, commands, physics, audio, atmosphere, ui } from "./runtime/modules.ts";
