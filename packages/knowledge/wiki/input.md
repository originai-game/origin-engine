<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `input` module

Input as the game sees it: the Input resource for polling, events for edges, og.Pointers for several pointers at once (multi-touch) and og.Gamepad for up to eight pads. All of it is world state, so it replays.

Gameplay actions (og.input.defineAction) name what the player does instead of the keys that do it, in action maps the player rebinds; prompts read each action's glyph (og.input.glyph).

Mouse-look (pointer lock, mouse delta, relative mouse movement): set og.PointerLock.wanted and turn the camera by og.Pointers.movement each tick.

See `input`.

- **TypeScript**: functions are the namespace `og.input`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module input {
  doc "Input as the game sees it: the Input resource for polling, events for edges, og.Pointers for several pointers at once (multi-touch) and og.Gamepad for up to eight pads. All of it is world state, so it replays."
  doc "Gameplay actions (og.input.defineAction) name what the player does instead of the keys that do it, in action maps the player rebinds; prompts read each action's glyph (og.input.glyph)."
  doc "Mouse-look (pointer lock, mouse delta, relative mouse movement): set og.PointerLock.wanted and turn the camera by og.Pointers.movement each tick."
  # declarations follow, each on its own page
```

<sub>Module `input` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`MAX_POINTERS`](#const-MAX_POINTERS) | constant |  | The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. |
| [`MAX_GAMEPADS`](#const-MAX_GAMEPADS) | constant |  | The most gamepads og.Gamepad reads: pads 0 to 7. |
| [`og.input.intent`](input/fn-intent.md) | function | `<E extends AnyEvent>(world: World, event: E, payload: Value<E>): Result<void>` | Send a reflected gameplay intent for the next tick and record its name, packed payload and layout hash. |
| [`RecordedIntent`](input/type-RecordedIntent.md) | struct |  | One deterministic input, independent of presentation checkpoints. |
| [`og.input.takeIntents`](input/fn-takeIntents.md) | function | `(world: World): readonly RecordedIntent[]` | Drain accepted intents in delivery order for recording. |
| [`og.input.replayIntent`](input/fn-replayIntent.md) | function | `(world: World, intent: RecordedIntent): Result<void>` | Deliver recorded input between ticks without UI state. |
| [`og.PauseIntent`](input/record-PauseIntent.md) | event |  | An engine gameplay intent: apply scale at the next tick boundary. |
| [`InputDevice`](input/type-InputDevice.md) | enum |  | A kind of input device: the set a glyph is drawn from, the device used last, and what a pad is. |
| [`ActionKind`](input/type-ActionKind.md) | enum |  | What an action gives. |
| [`TouchZone`](input/type-TouchZone.md) | desc struct |  | Where on the play surface a touch binding's touches must start, in fractions of it from the top left; a zone of no size is all of it. |
| [`InputBinding`](input/type-InputBinding.md) | desc struct |  | One binding of an action: a control, or a composite of parts. |
| [`ActionDesc`](input/type-ActionDesc.md) | desc struct |  | A gameplay action: a name the game reads instead of keys (`jump`, `move`, `look`), its kind and default bindings over keys, the mouse, pads and touch. |
| [`ActionState`](input/type-ActionState.md) | struct |  | An action this tick. |
| [`ActionInfo`](input/type-ActionInfo.md) | struct |  | An action as a controls screen lists it. |
| [`BindingConflict`](input/type-BindingConflict.md) | struct |  | Another binding a rebinding would share a control with: an action of the same action map (or another slot of the same action). |
| [`ConflictPolicy`](input/type-ConflictPolicy.md) | enum |  | What a rebinding does with a control another action holds. |
| [`RebindDesc`](input/type-RebindDesc.md) | desc struct |  | A player's rebinding of one slot of an action. |
| [`RebindAnswer`](input/type-RebindAnswer.md) | struct |  | What a rebinding did. |
| [`CaptureAccept`](input/type-CaptureAccept.md) | enum |  | Which devices a capture takes. |
| [`CaptureDesc`](input/type-CaptureDesc.md) | desc struct |  | A capture of the player's next input for a slot of an action: a controls screen's "press a key". |
| [`CaptureState`](input/type-CaptureState.md) | struct |  | Where a capture is: waiting, done with the control it took, or cancelled. |
| [`InputGlyph`](input/type-InputGlyph.md) | struct |  | A control's glyph, for prompts and a controls screen. |
| [`SensitivityDesc`](input/type-SensitivityDesc.md) | desc struct |  | The player's sensitivity and inversion of an axis action on one device class: a look action's mouse and its stick each have their own. |
| [`KeyLabel`](input/type-KeyLabel.md) | desc struct |  | A key and what the player's keyboard layout prints on it: `KeyQ` and `a` on AZERTY. |
| [`MouseDesc`](input/type-MouseDesc.md) | desc struct |  | How the host reads the mouse while the pointer is locked (og.PointerLock): the player's setting, kept by the game with its settings (og.platform.saveSettings). |
| [`MouseInfo`](input/type-MouseInfo.md) | struct |  | The mouse as the host reads it (og.input.mouse). |
| [`DeadzoneDesc`](input/type-DeadzoneDesc.md) | desc struct |  | A pad's dead zones: what its sticks and triggers read as rest. |
| [`RumbleDesc`](input/type-RumbleDesc.md) | desc struct |  | A rumble of a pad's motors: output for the host, never world state (a replay does not repeat it). |
| [`og.input.defineAction`](input/fn-defineAction.md) | function | `(world: World, desc?: ActionDesc): Result<void>` | Define (or redefine) a gameplay action. |
| [`og.input.actions`](input/fn-actions.md) | function | `(world: World): readonly ActionInfo[]` | Every action, in definition order, with its bindings now and its defaults: what a controls screen lists. |
| [`og.input.action`](input/fn-action.md) | function | `(world: World, name: string): ActionState \| null` | An action this tick: down, the edges, held, its value and vector, and the device that drove it. |
| [`og.input.isActionPressed`](input/fn-isActionPressed.md) | function | `(world: World, name: string): boolean` | Whether an action is down this tick (false for an unknown one). |
| [`og.input.isActionJustPressed`](input/fn-isActionJustPressed.md) | function | `(world: World, name: string): boolean` | Whether an action went down this tick. |
| [`og.input.isActionJustReleased`](input/fn-isActionJustReleased.md) | function | `(world: World, name: string): boolean` | Whether an action went up this tick. |
| [`og.input.actionHeld`](input/fn-actionHeld.md) | function | `(world: World, name: string): boolean` | Whether an action has been down for its holdMs. |
| [`og.input.actionValue`](input/fn-actionValue.md) | function | `(world: World, name: string): number` | An action's value: a button's strength (0 to 1), an axis's value, a two-axis value's length. |
| [`og.input.actionVector`](input/fn-actionVector.md) | function | `(world: World, name: string): Vector2` | A two-axis action's value (x right, y up; each -1 to 1 but the mouse's movement); (value, 0) for the others. |
| [`og.input.setActionMaps`](input/fn-setActionMaps.md) | function | `(world: World, actionMaps: readonly string[]): void` | The action maps whose actions are live, first the one whose bindings win a control another action map also binds (`["vehicle", "gameplay"]`: E exits the car, not interacts). |
| [`og.input.actionMaps`](input/fn-actionMaps.md) | function | `(world: World): readonly string[]` | The live action maps as og.input.setActionMaps set them (empty: every action map). |
| [`og.input.rebind`](input/fn-rebind.md) | function | `(world: World, desc?: RebindDesc): Result<RebindAnswer>` | Rebind one slot of an action (or one part of a composite) for the player. |
| [`og.input.conflicts`](input/fn-conflicts.md) | function | `(world: World, desc?: RebindDesc): Result<readonly BindingConflict[]>` | What a rebinding would share a control with, changing nothing. |
| [`og.input.resetBindings`](input/fn-resetBindings.md) | function | `(world: World, action?: string): Result<void>` | Back to the defaults: one action's bindings and toggle, or (no action) every action's, and the pads' dead zones. |
| [`og.input.setToggle`](input/fn-setToggle.md) | function | `(world: World, action: string, on: boolean): Result<void>` | The player's choice for a button action: a press turns it on and the next off (true), or it is down while held (false). |
| [`og.input.saveBindings`](input/fn-saveBindings.md) | function | `(world: World): string` | The player's settings as text to keep (og.platform.saveSettings, a save): every rebound action's bindings, the toggles and the dead zones that differ from the defaults. |
| [`og.input.loadBindings`](input/fn-loadBindings.md) | function | `(world: World, text: string): Result<readonly string[]>` | Take what og.input.saveBindings wrote, in place of the player's settings now. |
| [`og.input.startCapture`](input/fn-startCapture.md) | function | `(world: World, desc?: CaptureDesc): Result<void>` | Start capturing the player's next input for a slot of an action (a controls screen's "press a key"): read og.input.capture each tick until it is done or cancelled, then og.input.applyCapture. |
| [`og.input.capture`](input/fn-capture.md) | function | `(world: World): CaptureState` | Where the capture is: active while it waits, done with the control it took, its binding and the conflicts applying it would meet, or cancelled. |
| [`og.input.cancelCapture`](input/fn-cancelCapture.md) | function | `(world: World): void` | Stop a capture without taking anything. |
| [`og.input.applyCapture`](input/fn-applyCapture.md) | function | `(world: World, policy?: ConflictPolicy): Result<RebindAnswer>` | Apply what the capture took: og.input.rebind with its slot and binding. |
| [`og.input.glyph`](input/fn-glyph.md) | function | `(world: World, action: string, device?: InputDevice): InputGlyph \| null` | An action's prompt glyph: its first binding on the device (the device used last when omitted); a composite's in one (`W A S D`, `key.wasd`). |
| [`og.input.glyphs`](input/fn-glyphs.md) | function | `(world: World, action: string, device?: InputDevice): readonly InputGlyph[]` | An action's first binding on the device, one glyph per control (a composite's parts in prompt order: up, left, down, right; negative, positive). |
| [`og.input.controlGlyph`](input/fn-controlGlyph.md) | function | `(control: string, device?: InputDevice): InputGlyph \| null` | A control's glyph (what a capture took, before it is applied). |
| [`og.input.device`](input/fn-device.md) | function | `(world: World): InputDevice` | The device the player used last: a key or the mouse, a pad (its family as the host told it, else xbox), or a touch. |
| [`og.input.padDevice`](input/fn-padDevice.md) | function | `(world: World, pad: number): InputDevice \| null` | A pad's family as the host told it from the pad's id (Xbox, PlayStation, Switch or a generic pad). |
| [`og.input.setDeadzones`](input/fn-setDeadzones.md) | function | `(world: World, desc?: DeadzoneDesc): Result<void>` | A pad's dead zones (or every pad's): the player's setting, kept by og.input.saveBindings. |
| [`og.input.setSensitivity`](input/fn-setSensitivity.md) | function | `(world: World, desc?: SensitivityDesc): Result<void>` | Set the player's sensitivity and inversion of an axis action on a device class (SensitivityDesc): applied from the next tick's actions, kept by og.input.saveBindings. |
| [`og.input.sensitivity`](input/fn-sensitivity.md) | function | `(world: World, action: string, device?: InputDevice): SensitivityDesc` | An action's sensitivity on a device class (the device used last when omitted): the player's, else scale 1 and nothing inverted. |
| [`og.input.setKeyboardLayout`](input/fn-setKeyboardLayout.md) | function | `(labels: readonly KeyLabel[]): void` | Set the keys the player's keyboard layout prints, for glyphs: bindings still name physical keys (KeyboardEvent.code), so only their glyphs change (Z Q S D on AZERTY). |
| [`og.input.keyboardLayout`](input/fn-keyboardLayout.md) | function | `(): readonly KeyLabel[]` | The keys the layout prints as og.input.setKeyboardLayout last set them, sorted by code (empty: the US layout). |
| [`og.input.setMouse`](input/fn-setMouse.md) | function | `(world: World, desc?: MouseDesc): Result<void>` | Set how the host reads the mouse (MouseDesc: raw input). |
| [`og.input.mouse`](input/fn-mouse.md) | function | `(world: World): MouseInfo` | How the host reads the mouse: raw input asked for, granted or refused by the browser. |
| [`og.input.rumble`](input/fn-rumble.md) | function | `(world: World, desc?: RumbleDesc): Result<void>` | Rumble a pad (or every connected pad): the web shell plays it on the pad's vibration actuator. |
| [`GamepadButton`](input/type-GamepadButton.md) | enum |  | A gamepad button in the W3C standard mapping, in its index order (a is button 0). |
| [`GamepadAxis`](input/type-GamepadAxis.md) | enum |  | An analog input of a gamepad. |
| [`KeyAction`](input/type-KeyAction.md) | enum |  | What an injected key event does. |
| [`PointerAction`](input/type-PointerAction.md) | enum |  | What an injected pointer event does. |
| [`PointerButton`](input/type-PointerButton.md) | enum |  | A physical pointer button; `none` is a hover move. |
| [`Modifiers`](input/type-Modifiers.md) | struct |  | Modifier keys held during a key event. |
| [`KeyInput`](input/type-KeyInput.md) | struct |  | A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA. |
| [`PointerInput`](input/type-PointerInput.md) | struct |  | A pointer event at a point in CSS pixels of the play surface. |
| [`PointerLockReport`](input/type-PointerLockReport.md) | struct |  | The host reports whether the pointer is locked to the play surface now (an engine extension of protocol 5): the web shell sends one from the page's pointerlockchange, and a playtest can send its own;… |
| [`GamepadInput`](input/type-GamepadInput.md) | struct |  | A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own. |
| [`WheelInput`](input/type-WheelInput.md) | struct |  | A wheel or trackpad scroll at a point. |
| [`TextEntry`](input/type-TextEntry.md) | struct |  | Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's compos… |
| [`A11yActionKind`](input/type-A11yActionKind.md) | enum |  | What an assistive technology asks of a UI control. |
| [`A11yRequest`](input/type-A11yRequest.md) | struct |  | A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. |
| [`InjectedInput`](input/type-InjectedInput.md) | union |  | One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it. |
| [`og.Input`](input/record-Input.md) | resource |  | The input state at the start of this tick (og-input's og.Input, version 2). |
| [`og.Pointers`](input/record-Pointers.md) | resource |  | Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. |
| [`og.PointerLock`](input/record-PointerLock.md) | resource |  | Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), t… |
| [`og.Gamepad`](input/record-Gamepad.md) | resource |  | Up to eight gamepads at the start of this tick (og-input's og.Gamepad, version 2), in the W3C standard layout. |
| [`og.KeyDown`](input/record-KeyDown.md) | event |  | A key went down (a down for a held key is a repeat and sets no edge). |
| [`og.TextInput`](input/record-TextInput.md) | event |  | Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. |
| [`og.KeyUp`](input/record-KeyUp.md) | event |  | A key went up. |
| [`og.PointerDown`](input/record-PointerDown.md) | event |  | A pointer button went down (or a touch landed). |
| [`og.PointerUp`](input/record-PointerUp.md) | event |  | A pointer button went up (or a touch lifted). |
| [`og.PointerMove`](input/record-PointerMove.md) | event |  | A pointer moved. |
| [`og.Wheel`](input/record-Wheel.md) | event |  | A wheel or trackpad scroll. |
| [`og.A11yAction`](input/record-A11yAction.md) | event |  | A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. |

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_POINTERS"></a>`MAX_POINTERS` | `number` | `5` | The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. |
| <a id="const-MAX_GAMEPADS"></a>`MAX_GAMEPADS` | `number` | `8` | The most gamepads og.Gamepad reads: pads 0 to 7. |

```ogt
const MAX_POINTERS u32 5 "The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4."
```

<sub>Module `input` line 7</sub>

```ogt
const MAX_GAMEPADS u32 8 "The most gamepads og.Gamepad reads: pads 0 to 7."
```

<sub>Module `input` line 8</sub>
