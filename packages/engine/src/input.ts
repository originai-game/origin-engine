// og.input (IDL `input`): gameplay actions, rebinding, capture, glyphs, dead
// zones, sensitivity, the keyboard layout's printed keys, the host's mouse and rumble, as thin calls into the host's binding symbols (og-input's
// actions.rs owns the behaviour), typed by the generated declarations through
// index.ts.

import type { HostAbi } from "./generated/abi.ts";
import type { RecordedIntent, Result } from "@origingame/engine";
import { boundHost } from "./runtime.ts";

type W = { readonly id: number };

function abi(): HostAbi<Uint32Array, unknown> {
  const h = boundHost();
  if (!h) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return h.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));

export const input = Object.freeze({
  intent: (world: W, event: { readonly name: string }, payload: unknown): Result<void> =>
    parse(abi().og_input_intent(world.id, event.name, JSON.stringify(payload ?? {}))),
  takeIntents: (world: W): readonly RecordedIntent[] => parse(abi().og_input_take_intents(world.id)),
  replayIntent: (world: W, intent: RecordedIntent): Result<void> =>
    parse(abi().og_input_replay_intent(world.id, JSON.stringify(intent))),
  defineAction: (world: W, desc?: unknown) => parse(abi().og_input_define_action(world.id, json(desc))),
  actions: (world: W) => parse(abi().og_input_actions(world.id)),
  action: (world: W, name: string) => parse(abi().og_input_action(world.id, name)),
  isActionPressed: (world: W, name: string) => abi().og_input_is_action_pressed(world.id, name),
  isActionJustPressed: (world: W, name: string) => abi().og_input_is_action_just_pressed(world.id, name),
  isActionJustReleased: (world: W, name: string) => abi().og_input_is_action_just_released(world.id, name),
  actionHeld: (world: W, name: string) => abi().og_input_action_held(world.id, name),
  actionValue: (world: W, name: string) => abi().og_input_action_value(world.id, name),
  actionVector: (world: W, name: string) => parse(abi().og_input_action_vector(world.id, name)),
  moveRelative: (world: W, name: string, camera: unknown) => parse(abi().og_input_move_relative(world.id, name, JSON.stringify(camera))),
  setActionMaps: (world: W, contexts: readonly string[]) => abi().og_input_set_action_maps(world.id, JSON.stringify(contexts)),
  actionMaps: (world: W) => parse(abi().og_input_action_maps(world.id)),
  rebind: (world: W, desc?: unknown) => parse(abi().og_input_rebind(world.id, json(desc))),
  conflicts: (world: W, desc?: unknown) => parse(abi().og_input_conflicts(world.id, json(desc))),
  resetBindings: (world: W, action?: string) => parse(abi().og_input_reset_bindings(world.id, action)),
  setToggle: (world: W, action: string, on: boolean) => parse(abi().og_input_set_toggle(world.id, action, on)),
  saveBindings: (world: W) => abi().og_input_save_bindings(world.id),
  loadBindings: (world: W, text: string) => parse(abi().og_input_load_bindings(world.id, text)),
  startCapture: (world: W, desc?: unknown) => parse(abi().og_input_start_capture(world.id, json(desc))),
  capture: (world: W) => parse(abi().og_input_capture(world.id)),
  cancelCapture: (world: W) => abi().og_input_cancel_capture(world.id),
  applyCapture: (world: W, policy?: string) => parse(abi().og_input_apply_capture(world.id, policy)),
  glyph: (world: W, action: string, device?: string) => parse(abi().og_input_glyph(world.id, action, device)),
  glyphs: (world: W, action: string, device?: string) => parse(abi().og_input_glyphs(world.id, action, device)),
  controlGlyph: (control: string, device?: string) => parse(abi().og_input_control_glyph(control, device)),
  device: (world: W) => abi().og_input_device(world.id),
  padDevice: (world: W, pad: number) => parse(abi().og_input_pad_device(world.id, pad)),
  setDeadzones: (world: W, desc?: unknown) => parse(abi().og_input_set_deadzones(world.id, json(desc))),
  setSensitivity: (world: W, desc?: unknown) => parse(abi().og_input_set_sensitivity(world.id, json(desc))),
  sensitivity: (world: W, action: string, device?: string) => parse(abi().og_input_sensitivity(world.id, action, device)),
  setKeyboardLayout: (labels: readonly unknown[]) => abi().og_input_set_keyboard_layout(JSON.stringify(labels ?? [])),
  keyboardLayout: () => parse(abi().og_input_keyboard_layout()),
  setMouse: (world: W, desc?: unknown) => parse(abi().og_input_set_mouse(world.id, json(desc))),
  mouse: (world: W) => parse(abi().og_input_mouse(world.id)),
  rumble: (world: W, desc?: unknown) => parse(abi().og_input_rumble(world.id, json(desc))),
});
