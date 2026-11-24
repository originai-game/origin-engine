// The members the IDL marks `custom`, which src/generated/calls.ts calls
// instead of forwarding to their symbol: each module owns its own, and
// `Custom` (generated) makes a missing one a type error. Read through
// getters, since the modules import the generated classes they extend.

import type { Custom } from "../generated/calls.ts";
import { debugCustom } from "../depth.ts";
import { shapeCustom } from "../geom.ts";
import { platformCustom } from "../platform-rooms.ts";
import { defineGame, defineHook, defineRecord, defineSystem, type FieldSpec } from "./declare.ts";
import { engineProbeCustom, runtimeProbeCustom } from "./engine.ts";
import { audioCustom, commandsCustom, tweenTimelineCustom } from "./modules.ts";
import { rngCustom } from "./resources.ts";
import { saveEditCustom, worldCustom } from "./world.ts";

const reflect: Custom["reflect"] = {
  defineComponent: (name, fields) => defineRecord("component", name, fields as Record<string, FieldSpec>),
  defineResource: (name, fields) => defineRecord("resource", name, fields as Record<string, FieldSpec>),
  defineEvent: (name, fields) => defineRecord("event", name, fields as Record<string, FieldSpec>),
};

const system: Custom["system"] = {
  defineHook: (name, component, desc, run) => defineHook(name, component as never, desc as never, run),
  defineSystem: (name, desc, run) => defineSystem(name, desc as never, run),
};

export const custom: Custom = {
  get World() { return worldCustom; },
  get SaveEdit() { return saveEditCustom; },
  get Rng() { return rngCustom; },
  get RuntimeProbe() { return runtimeProbeCustom; },
  get EngineProbe() { return engineProbeCustom; },
  get TweenTimeline() { return tweenTimelineCustom; },
  get audio() { return audioCustom; },
  get commands() { return commandsCustom; },
  get debug() { return debugCustom; },
  get platform() { return platformCustom; },
  get render() { return shapeCustom; },
  get game() { return { game: (desc?: unknown) => defineGame(desc as never) }; },
  get reflect() { return reflect; },
  get system() { return system; },
};
