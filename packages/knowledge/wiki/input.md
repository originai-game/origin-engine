<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `input` module

Input as the game sees it: the Input resource for polling, events for edges.

The shell collects pointer, keyboard and wheel input on the main thread; the probe and the command channel inject through the same path.

Several pointers at once (multi-touch) are og.Pointers; up to eight gamepads, mapped to the W3C standard layout, are og.Gamepad. Both are world state, as og.Input is, so they replay.

Gameplay actions (og.input.defineAction) name what the player does instead of the keys that do it: buttons and one- or two-axis values bound to keys, the mouse, pad buttons, sticks, triggers and touch, in contexts, with dead zones and sensitivity. The player rebinds them (og.input.rebind, og.input.startCapture for a controls screen, conflicts reported), the bindings are kept as text (og.input.saveBindings), and prompts read each action's glyph on the device used last (og.input.glyph). Actions are read once per tick from og.Input, og.Pointers and og.Gamepad, so they replay; in lockstep they are this peer's own, handed on with og.net.submit as og.Input is.

Mouse-look (pointer lock, mouse delta, relative mouse movement): set og.PointerLock.wanted and turn the camera by og.Pointers.movement each tick.

- **TypeScript**: functions are the namespace `og.input`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module input {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1597</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`MAX_POINTERS`](#const-MAX_POINTERS) | constant | The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. |
| [`MAX_GAMEPADS`](#const-MAX_GAMEPADS) | constant | The most gamepads og.Gamepad reads: pads 0 to 7. |
| [`InputDevice`](#type-InputDevice) | enum | A kind of input device: the set a glyph is drawn from, the device used last, and what a pad is. og.ui's UiDevice has the same values. |
| [`ActionKind`](#type-ActionKind) | enum | What an action gives. |
| [`TouchZone`](#type-TouchZone) | desc struct | Where on the play surface a touch binding's touches must start, in fractions of it from the top left; a zone of no size is all of it. |
| [`InputBinding`](#type-InputBinding) | desc struct | One binding of an action: a control, or a composite of parts (A and D as one axis, WASD as two). Controls are named by strings: a key code (`KeyW`, `Space`), `mouse.left`, `mouse.middle`, `mouse.right`, `mouse.x`, `mouse.y`, `mouse.xy` (the mouse's movement this tick, CSS px), `wheel.x`, `wheel.y` (notches of 100 CSS px), `pad.a` to `pad.home` (GamepadButton's names, any pad), `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY`, `pad.leftTrigger`, `pad.rightTrigger`, `pad.leftXY`, `pad.rightXY` (a stick), `pad.dpad`, `touch` (a touch held) and `touch.xy` (a touch stick). A one-axis control takes `+` or `-` for its half (`pad.leftX+`: 0 to 1, a direction as a button). |
| [`ActionDesc`](#type-ActionDesc) | desc struct | A gameplay action: a name the game reads instead of keys (`jump`, `move`, `look`), its kind and default bindings over keys, the mouse, pads and touch. Actions are read once per tick, before the game's systems, from og.Input, og.Pointers and og.Gamepad, so they replay with them; the player rebinds them (og.input.rebind, og.input.startCapture) and a settings screen lists them (og.input.actions). |
| [`ActionState`](#type-ActionState) | struct | An action this tick. |
| [`ActionInfo`](#type-ActionInfo) | struct | An action as a controls screen lists it. |
| [`BindingConflict`](#type-BindingConflict) | struct | Another binding a rebinding would share a control with: an action of the same context (or another slot of the same action). |
| [`ConflictPolicy`](#type-ConflictPolicy) | enum | What a rebinding does with a control another action holds. |
| [`RebindDesc`](#type-RebindDesc) | desc struct | A player's rebinding of one slot of an action. |
| [`RebindAnswer`](#type-RebindAnswer) | struct | What a rebinding did. |
| [`CaptureAccept`](#type-CaptureAccept) | enum | Which devices a capture takes. |
| [`CaptureDesc`](#type-CaptureDesc) | desc struct | A capture of the player's next input for a slot of an action: a controls screen's "press a key". From the next tick every action reads nothing until it ends; what was pressed for it moves no action until it is let go. |
| [`CaptureState`](#type-CaptureState) | struct | Where a capture is: waiting, done with the control it took, or cancelled. |
| [`InputGlyph`](#type-InputGlyph) | struct | A control's glyph, for prompts and a controls screen. |
| [`SensitivityDesc`](#type-SensitivityDesc) | desc struct | The player's sensitivity and inversion of an axis action on one device class: a look action's mouse and its stick each have their own (a settings screen's Mouse sensitivity and Stick sensitivity sliders, Invert Y). Kept by og.input.saveBindings, forgotten by og.input.resetBindings; a world whose player set none saves and hashes as before. |
| [`KeyLabel`](#type-KeyLabel) | desc struct | A key and what the player's keyboard layout prints on it: `KeyQ` and `a` on AZERTY. |
| [`MouseDesc`](#type-MouseDesc) | desc struct | How the host reads the mouse while the pointer is locked (og.PointerLock): the player's setting, kept by the game with its settings (og.platform.saveSettings). Host only: none of it is world state. |
| [`MouseInfo`](#type-MouseInfo) | struct | The mouse as the host reads it (og.input.mouse). |
| [`DeadzoneDesc`](#type-DeadzoneDesc) | desc struct | A pad's dead zones: what its sticks and triggers read as rest. The player's to set (a settings slider); og.input.saveBindings keeps them. |
| [`RumbleDesc`](#type-RumbleDesc) | desc struct | A rumble of a pad's motors: output for the host, never world state (a replay does not repeat it). |
| [`og.input.defineAction`](#fn-defineAction) | function | Define (or redefine) a gameplay action. A redefinition keeps the player's bindings while they fit it, and its state while its kind stays. Callable anywhere (GameDesc.start, a system); the map is world state, so snapshots carry it. |
| [`og.input.actions`](#fn-actions) | function | Every action, in definition order, with its bindings now and its defaults: what a controls screen lists. |
| [`og.input.action`](#fn-action) | function | An action this tick: down, the edges, held, its value and vector, and the device that drove it. Null for an unknown action. |
| [`og.input.actionDown`](#fn-actionDown) | function | Whether an action is down this tick (false for an unknown one). |
| [`og.input.actionPressed`](#fn-actionPressed) | function | Whether an action went down this tick. |
| [`og.input.actionReleased`](#fn-actionReleased) | function | Whether an action went up this tick. |
| [`og.input.actionHeld`](#fn-actionHeld) | function | Whether an action has been down for its holdMs. |
| [`og.input.actionValue`](#fn-actionValue) | function | An action's value: a button's strength (0 to 1), an axis's value, a two-axis value's length. 0 for an unknown action. |
| [`og.input.actionVector`](#fn-actionVector) | function | A two-axis action's value (x right, y up; each -1 to 1 but the mouse's movement); (value, 0) for the others. |
| [`og.input.setContexts`](#fn-setContexts) | function | The contexts whose actions are live, first the one whose bindings win a control another context also binds (`["vehicle", "gameplay"]`: E exits the car, not interacts). Empty (the default) is every context, none winning. Takes effect from the next tick's actions. |
| [`og.input.contexts`](#fn-contexts) | function | The live contexts as og.input.setContexts set them (empty: every context). |
| [`og.input.rebind`](#fn-rebind) | function | Rebind one slot of an action (or one part of a composite) for the player. With policy refuse (the default) a binding that shares a control with another action of its context changes nothing and answers the conflicts; swap, replace and allow resolve them. The player's bindings stay until reset and are what og.input.saveBindings writes. |
| [`og.input.conflicts`](#fn-conflicts) | function | What a rebinding would share a control with, changing nothing. |
| [`og.input.resetBindings`](#fn-resetBindings) | function | Back to the defaults: one action's bindings and toggle, or (no action) every action's, and the pads' dead zones. |
| [`og.input.setToggle`](#fn-setToggle) | function | The player's choice for a button action: a press turns it on and the next off (true), or it is down while held (false). |
| [`og.input.saveBindings`](#fn-saveBindings) | function | The player's settings as text to keep (og.platform.saveSettings, a save): every rebound action's bindings, the toggles and the dead zones that differ from the defaults. |
| [`og.input.loadBindings`](#fn-loadBindings) | function | Take what og.input.saveBindings wrote, in place of the player's settings now. Bindings for an action not defined yet are kept until it is (load at start, before or after defining). |
| [`og.input.startCapture`](#fn-startCapture) | function | Start capturing the player's next input for a slot of an action (a controls screen's "press a key"): read og.input.capture each tick until it is done or cancelled, then og.input.applyCapture. |
| [`og.input.capture`](#fn-capture) | function | Where the capture is: active while it waits, done with the control it took, its binding and the conflicts applying it would meet, or cancelled. |
| [`og.input.cancelCapture`](#fn-cancelCapture) | function | Stop a capture without taking anything. |
| [`og.input.applyCapture`](#fn-applyCapture) | function | Apply what the capture took: og.input.rebind with its slot and binding. |
| [`og.input.glyph`](#fn-glyph) | function | An action's prompt glyph: its first binding on the device (the device used last when omitted); a composite's in one (`W A S D`, `key.wasd`). Null for an unknown action or one with no binding on that device. |
| [`og.input.glyphs`](#fn-glyphs) | function | An action's first binding on the device, one glyph per control (a composite's parts in prompt order: up, left, down, right; negative, positive). Empty for an unknown action or one with no binding there. |
| [`og.input.controlGlyph`](#fn-controlGlyph) | function | A control's glyph (what a capture took, before it is applied). Null for a name that names no control. |
| [`og.input.device`](#fn-device) | function | The device the player used last: a key or the mouse, a pad (its family as the host told it, else xbox), or a touch. Prompts follow it live. |
| [`og.input.padDevice`](#fn-padDevice) | function | A pad's family as the host told it from the pad's id (Xbox, PlayStation, Switch or a generic pad). Null for a pad that is not connected or that the host did not name. |
| [`og.input.setDeadzones`](#fn-setDeadzones) | function | A pad's dead zones (or every pad's): the player's setting, kept by og.input.saveBindings. |
| [`og.input.setSensitivity`](#fn-setSensitivity) | function | Set the player's sensitivity and inversion of an axis action on a device class (SensitivityDesc): applied from the next tick's actions, kept by og.input.saveBindings. |
| [`og.input.sensitivity`](#fn-sensitivity) | function | An action's sensitivity on a device class (the device used last when omitted): the player's, else scale 1 and nothing inverted. |
| [`og.input.setKeyboardLayout`](#fn-setKeyboardLayout) | function | The keys the player's keyboard layout prints, for glyphs: the web shell sets it from navigator.keyboard.getLayoutMap() (Chrome, Edge), and where the browser has none from the characters the player's unmodified keys type as they are pressed (Firefox, Safari: a key reads as on a US layout until it is pressed once). Bindings still name physical keys (KeyboardEvent.code), so WASD is the same four keys on every layout; only their glyphs change (Z Q S D on AZERTY). Empty is the US layout (native runs). The host's, one for the page: no world state. |
| [`og.input.keyboardLayout`](#fn-keyboardLayout) | function | The keys the layout prints as og.input.setKeyboardLayout last set them, sorted by code (empty: the US layout). |
| [`og.input.setMouse`](#fn-setMouse) | function | Set how the host reads the mouse (MouseDesc: raw input). Host only, never world state; fields left out keep their values. |
| [`og.input.mouse`](#fn-mouse) | function | How the host reads the mouse: raw input asked for, granted or refused by the browser. |
| [`og.input.rumble`](#fn-rumble) | function | Rumble a pad (or every connected pad): the web shell plays it on the pad's vibration actuator. Output only: it never feeds back into the simulation, and a native run records it in the frame answer's `rumble`. |
| [`GamepadButton`](#type-GamepadButton) | enum | A gamepad button in the W3C standard mapping, in its index order (a is button 0). |
| [`GamepadAxis`](#type-GamepadAxis) | enum | An analog input of a gamepad. |
| [`KeyAction`](#type-KeyAction) | enum | What an injected key event does. |
| [`PointerAction`](#type-PointerAction) | enum | What an injected pointer event does. |
| [`PointerButton`](#type-PointerButton) | enum | A physical pointer button; `none` is a hover move. |
| [`Modifiers`](#type-Modifiers) | struct | Modifier keys held during a key event. |
| [`KeyInput`](#type-KeyInput) | struct | A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA. |
| [`PointerInput`](#type-PointerInput) | struct | A pointer event at a point in CSS pixels of the play surface. |
| [`PointerLockReport`](#type-PointerLockReport) | struct | The host reports whether the pointer is locked to the play surface now (an engine extension of protocol 5): the web shell sends one from the page's pointerlockchange, and a playtest can send its own; og.PointerLock.locked follows it. |
| [`GamepadInput`](#type-GamepadInput) | struct | A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own. |
| [`WheelInput`](#type-WheelInput) | struct | A wheel or trackpad scroll at a point. |
| [`TextEntry`](#type-TextEntry) | struct | Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's composition so far. Raises TextInput events; the UI's focused text field takes them. |
| [`A11yActionKind`](#type-A11yActionKind) | enum | What an assistive technology asks of a UI control. |
| [`A11yRequest`](#type-A11yRequest) | struct | A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. Raises an A11yAction event; the UI acts on it at the start of the next frame, against the last frame's controls (a path naming no focusable control of the topmost dialog is ignored; increment and decrement also take a stepper's spinbutton, whose buttons are the focusable controls). |
| [`InjectedInput`](#type-InjectedInput) | union | One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it. |
| [`og.Input`](#record-Input) | resource | The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods. |
| [`og.Pointers`](#record-Pointers) | resource | Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id). |
| [`og.PointerLock`](#record-PointerLock) | resource | Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default. |
| [`og.Gamepad`](#record-Gamepad) | resource | Up to eight gamepads at the start of this tick (og-input's og.Gamepad, version 2), in the W3C standard layout: on the web the shell polls the pads each display frame, maps a pad the browser does not map (Firefox, Linux, many third-party pads) through its table, and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys. |
| [`og.KeyDown`](#record-KeyDown) | event | A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems. |
| [`og.TextInput`](#record-TextInput) | event | Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one. |
| [`og.KeyUp`](#record-KeyUp) | event | A key went up. |
| [`og.PointerDown`](#record-PointerDown) | event | A pointer button went down (or a touch landed). |
| [`og.PointerUp`](#record-PointerUp) | event | A pointer button went up (or a touch lifted). |
| [`og.PointerMove`](#record-PointerMove) | event | A pointer moved. |
| [`og.Wheel`](#record-Wheel) | event | A wheel or trackpad scroll. |
| [`og.A11yAction`](#record-A11yAction) | event | A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused. |

## Functions

<a id="fn-defineAction"></a>

### `og.input.defineAction`

Define (or redefine) a gameplay action. A redefinition keeps the player's bindings while they fit it, and its state while its kind stays. Callable anywhere (GameDesc.start, a system); the map is world state, so snapshots carry it.

```ts
og.input.defineAction(world: World, desc?: ActionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ActionDesc` | yes | The action. |

**Returns** `Result<void>`: Refused, naming the fix, for a binding whose control names none or that does not fit the kind (a stick on a button, a key alone on a two-axis action).

Binding symbol `og_input_define_action` · publish builds: kept

```ogt
fn defineAction {
  doc "Define (or redefine) a gameplay action. A redefinition keeps the player's bindings while they fit it, and its state while its kind stays. Callable anywhere (GameDesc.start, a system); the map is world state, so snapshots carry it."
  param world World "The world."
  param desc ActionDesc "The action."
  returns Result.Void "Refused, naming the fix, for a binding whose control names none or that does not fit the kind (a stick on a button, a key alone on a two-axis action)."
}
```

<sub>`api.ogt` line 1883</sub>

<a id="fn-actions"></a>

### `og.input.actions`

Every action, in definition order, with its bindings now and its defaults: what a controls screen lists.

```ts
og.input.actions(world: World): readonly ActionInfo[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly ActionInfo[]`.

Binding symbol `og_input_actions` · publish builds: kept

```ogt
fn actions {
  doc "Every action, in definition order, with its bindings now and its defaults: what a controls screen lists."
  param world World "The world."
  returns List.ActionInfo
}
```

<sub>`api.ogt` line 1890</sub>

<a id="fn-action"></a>

### `og.input.action`

An action this tick: down, the edges, held, its value and vector, and the device that drove it. Null for an unknown action.

```ts
og.input.action(world: World, name: string): ActionState | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `ActionState | null`.

Binding symbol `og_input_action` · publish builds: kept

```ogt
fn action {
  doc "An action this tick: down, the edges, held, its value and vector, and the device that drove it. Null for an unknown action."
  param world World "The world."
  param name String "The action."
  returns Option.ActionState
}
```

<sub>`api.ogt` line 1896</sub>

<a id="fn-actionDown"></a>

### `og.input.actionDown`

Whether an action is down this tick (false for an unknown one).

```ts
og.input.actionDown(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_down` · publish builds: kept

```ogt
fn actionDown {
  doc "Whether an action is down this tick (false for an unknown one)."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 1903</sub>

<a id="fn-actionPressed"></a>

### `og.input.actionPressed`

Whether an action went down this tick.

```ts
og.input.actionPressed(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_pressed` · publish builds: kept

```ogt
fn actionPressed {
  doc "Whether an action went down this tick."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 1910</sub>

<a id="fn-actionReleased"></a>

### `og.input.actionReleased`

Whether an action went up this tick.

```ts
og.input.actionReleased(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_released` · publish builds: kept

```ogt
fn actionReleased {
  doc "Whether an action went up this tick."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 1917</sub>

<a id="fn-actionHeld"></a>

### `og.input.actionHeld`

Whether an action has been down for its holdMs.

```ts
og.input.actionHeld(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_held` · publish builds: kept

```ogt
fn actionHeld {
  doc "Whether an action has been down for its holdMs."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 1924</sub>

<a id="fn-actionValue"></a>

### `og.input.actionValue`

An action's value: a button's strength (0 to 1), an axis's value, a two-axis value's length. 0 for an unknown action.

```ts
og.input.actionValue(world: World, name: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `number`.

Binding symbol `og_input_action_value` · publish builds: kept

```ogt
fn actionValue {
  doc "An action's value: a button's strength (0 to 1), an axis's value, a two-axis value's length. 0 for an unknown action."
  param world World "The world."
  param name String "The action."
  returns f32
}
```

<sub>`api.ogt` line 1931</sub>

<a id="fn-actionVector"></a>

### `og.input.actionVector`

A two-axis action's value (x right, y up; each -1 to 1 but the mouse's movement); (value, 0) for the others.

```ts
og.input.actionVector(world: World, name: string): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `Vector2`.

Binding symbol `og_input_action_vector` · publish builds: kept

```ogt
fn actionVector {
  doc "A two-axis action's value (x right, y up; each -1 to 1 but the mouse's movement); (value, 0) for the others."
  param world World "The world."
  param name String "The action."
  returns Vector2
}
```

<sub>`api.ogt` line 1938</sub>

<a id="fn-setContexts"></a>

### `og.input.setContexts`

The contexts whose actions are live, first the one whose bindings win a control another context also binds (`["vehicle", "gameplay"]`: E exits the car, not interacts). Empty (the default) is every context, none winning. Takes effect from the next tick's actions.

```ts
og.input.setContexts(world: World, contexts: readonly string[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `contexts` | `readonly string[]` | no | The live contexts, by priority. |

**Returns** `void`.

Binding symbol `og_input_set_contexts` · publish builds: kept

```ogt
fn setContexts {
  doc "The contexts whose actions are live, first the one whose bindings win a control another context also binds (`[\"vehicle\", \"gameplay\"]`: E exits the car, not interacts). Empty (the default) is every context, none winning. Takes effect from the next tick's actions."
  param world World "The world."
  param contexts List.String "The live contexts, by priority."
  returns Void
}
```

<sub>`api.ogt` line 1945</sub>

<a id="fn-contexts"></a>

### `og.input.contexts`

The live contexts as og.input.setContexts set them (empty: every context).

```ts
og.input.contexts(world: World): readonly string[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly string[]`.

Binding symbol `og_input_contexts` · publish builds: kept

```ogt
fn contexts {
  doc "The live contexts as og.input.setContexts set them (empty: every context)."
  param world World "The world."
  returns List.String
}
```

<sub>`api.ogt` line 1952</sub>

<a id="fn-rebind"></a>

### `og.input.rebind`

Rebind one slot of an action (or one part of a composite) for the player. With policy refuse (the default) a binding that shares a control with another action of its context changes nothing and answers the conflicts; swap, replace and allow resolve them. The player's bindings stay until reset and are what og.input.saveBindings writes.

```ts
og.input.rebind(world: World, desc?: RebindDesc): Result<RebindAnswer>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RebindDesc` | yes | The action, slot, part, binding and policy. |

**Returns** `Result<RebindAnswer>`: Refused for an unknown or locked action, a slot past one after the last, a part the kind has not, or a binding that does not fit.

Binding symbol `og_input_rebind` · publish builds: kept

```ogt
fn rebind {
  doc "Rebind one slot of an action (or one part of a composite) for the player. With policy refuse (the default) a binding that shares a control with another action of its context changes nothing and answers the conflicts; swap, replace and allow resolve them. The player's bindings stay until reset and are what og.input.saveBindings writes."
  param world World "The world."
  param desc RebindDesc "The action, slot, part, binding and policy."
  returns Result.RebindAnswer "Refused for an unknown or locked action, a slot past one after the last, a part the kind has not, or a binding that does not fit."
}
```

<sub>`api.ogt` line 1958</sub>

<a id="fn-conflicts"></a>

### `og.input.conflicts`

What a rebinding would share a control with, changing nothing.

```ts
og.input.conflicts(world: World, desc?: RebindDesc): Result<readonly BindingConflict[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RebindDesc` | yes | The rebinding. |

**Returns** `Result<readonly BindingConflict[]>`.

Binding symbol `og_input_conflicts` · publish builds: kept

```ogt
fn conflicts {
  doc "What a rebinding would share a control with, changing nothing."
  param world World "The world."
  param desc RebindDesc "The rebinding."
  returns Result.List.BindingConflict
}
```

<sub>`api.ogt` line 1965</sub>

<a id="fn-resetBindings"></a>

### `og.input.resetBindings`

Back to the defaults: one action's bindings and toggle, or (no action) every action's, and the pads' dead zones.

```ts
og.input.resetBindings(world: World, action?: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | yes | The action; omitted is every action. |

**Returns** `Result<void>`.

Binding symbol `og_input_reset_bindings` · publish builds: kept

```ogt
fn resetBindings {
  doc "Back to the defaults: one action's bindings and toggle, or (no action) every action's, and the pads' dead zones."
  param world World "The world."
  param action String "The action; omitted is every action." {
    optional true
  }
  returns Result.Void
}
```

<sub>`api.ogt` line 1972</sub>

<a id="fn-setToggle"></a>

### `og.input.setToggle`

The player's choice for a button action: a press turns it on and the next off (true), or it is down while held (false).

```ts
og.input.setToggle(world: World, action: string, on: boolean): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `on` | `boolean` | no | Toggle. |

**Returns** `Result<void>`.

Binding symbol `og_input_set_toggle` · publish builds: kept

```ogt
fn setToggle {
  doc "The player's choice for a button action: a press turns it on and the next off (true), or it is down while held (false)."
  param world World "The world."
  param action String "The action."
  param on bool "Toggle."
  returns Result.Void
}
```

<sub>`api.ogt` line 1981</sub>

<a id="fn-saveBindings"></a>

### `og.input.saveBindings`

The player's settings as text to keep (og.platform.saveSettings, a save): every rebound action's bindings, the toggles and the dead zones that differ from the defaults.

```ts
og.input.saveBindings(world: World): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string`.

Binding symbol `og_input_save_bindings` · publish builds: kept

```ogt
fn saveBindings {
  doc "The player's settings as text to keep (og.platform.saveSettings, a save): every rebound action's bindings, the toggles and the dead zones that differ from the defaults."
  param world World "The world."
  returns String
}
```

<sub>`api.ogt` line 1989</sub>

<a id="fn-loadBindings"></a>

### `og.input.loadBindings`

Take what og.input.saveBindings wrote, in place of the player's settings now. Bindings for an action not defined yet are kept until it is (load at start, before or after defining).

```ts
og.input.loadBindings(world: World, text: string): Result<readonly string[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `text` | `string` | no | The saved text. |

**Returns** `Result<readonly string[]>`: A warning for each entry it could not take (that action keeps its defaults); refused for text that is not saved bindings.

Binding symbol `og_input_load_bindings` · publish builds: kept

```ogt
fn loadBindings {
  doc "Take what og.input.saveBindings wrote, in place of the player's settings now. Bindings for an action not defined yet are kept until it is (load at start, before or after defining)."
  param world World "The world."
  param text String "The saved text."
  returns Result.List.String "A warning for each entry it could not take (that action keeps its defaults); refused for text that is not saved bindings."
}
```

<sub>`api.ogt` line 1995</sub>

<a id="fn-startCapture"></a>

### `og.input.startCapture`

Start capturing the player's next input for a slot of an action (a controls screen's "press a key"): read og.input.capture each tick until it is done or cancelled, then og.input.applyCapture.

```ts
og.input.startCapture(world: World, desc?: CaptureDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `CaptureDesc` | yes | The action, slot, part and the devices it takes. |

**Returns** `Result<void>`: Refused for an unknown or locked action, a slot or part a rebinding would refuse, or a cancel control that names none.

Binding symbol `og_input_start_capture` · publish builds: kept

```ogt
fn startCapture {
  doc "Start capturing the player's next input for a slot of an action (a controls screen's \"press a key\"): read og.input.capture each tick until it is done or cancelled, then og.input.applyCapture."
  param world World "The world."
  param desc CaptureDesc "The action, slot, part and the devices it takes."
  returns Result.Void "Refused for an unknown or locked action, a slot or part a rebinding would refuse, or a cancel control that names none."
}
```

<sub>`api.ogt` line 2002</sub>

<a id="fn-capture"></a>

### `og.input.capture`

Where the capture is: active while it waits, done with the control it took, its binding and the conflicts applying it would meet, or cancelled.

```ts
og.input.capture(world: World): CaptureState
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `CaptureState`.

Binding symbol `og_input_capture` · publish builds: kept

```ogt
fn capture {
  doc "Where the capture is: active while it waits, done with the control it took, its binding and the conflicts applying it would meet, or cancelled."
  param world World "The world."
  returns CaptureState
}
```

<sub>`api.ogt` line 2009</sub>

<a id="fn-cancelCapture"></a>

### `og.input.cancelCapture`

Stop a capture without taking anything.

```ts
og.input.cancelCapture(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `void`.

Binding symbol `og_input_cancel_capture` · publish builds: kept

```ogt
fn cancelCapture {
  doc "Stop a capture without taking anything."
  param world World "The world."
  returns Void
}
```

<sub>`api.ogt` line 2015</sub>

<a id="fn-applyCapture"></a>

### `og.input.applyCapture`

Apply what the capture took: og.input.rebind with its slot and binding.

```ts
og.input.applyCapture(world: World, policy?: ConflictPolicy): Result<RebindAnswer>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `policy` | `ConflictPolicy` | yes | What to do with a conflict; omitted is refuse. |

**Returns** `Result<RebindAnswer>`: Refused when no capture is done.

Binding symbol `og_input_apply_capture` · publish builds: kept

```ogt
fn applyCapture {
  doc "Apply what the capture took: og.input.rebind with its slot and binding."
  param world World "The world."
  param policy ConflictPolicy "What to do with a conflict; omitted is refuse." {
    optional true
  }
  returns Result.RebindAnswer "Refused when no capture is done."
}
```

<sub>`api.ogt` line 2021</sub>

<a id="fn-glyph"></a>

### `og.input.glyph`

An action's prompt glyph: its first binding on the device (the device used last when omitted); a composite's in one (`W A S D`, `key.wasd`). Null for an unknown action or one with no binding on that device.

```ts
og.input.glyph(world: World, action: string, device?: InputDevice): InputGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `device` | `InputDevice` | yes | The device; omitted is the one used last. |

**Returns** `InputGlyph | null`.

Binding symbol `og_input_glyph` · publish builds: kept

```ogt
fn glyph {
  doc "An action's prompt glyph: its first binding on the device (the device used last when omitted); a composite's in one (`W A S D`, `key.wasd`). Null for an unknown action or one with no binding on that device."
  param world World "The world."
  param action String "The action."
  param device InputDevice "The device; omitted is the one used last." {
    optional true
  }
  returns Option.InputGlyph
}
```

<sub>`api.ogt` line 2030</sub>

<a id="fn-glyphs"></a>

### `og.input.glyphs`

An action's first binding on the device, one glyph per control (a composite's parts in prompt order: up, left, down, right; negative, positive). Empty for an unknown action or one with no binding there.

```ts
og.input.glyphs(world: World, action: string, device?: InputDevice): readonly InputGlyph[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `device` | `InputDevice` | yes | The device; omitted is the one used last. |

**Returns** `readonly InputGlyph[]`.

Binding symbol `og_input_glyphs` · publish builds: kept

```ogt
fn glyphs {
  doc "An action's first binding on the device, one glyph per control (a composite's parts in prompt order: up, left, down, right; negative, positive). Empty for an unknown action or one with no binding there."
  param world World "The world."
  param action String "The action."
  param device InputDevice "The device; omitted is the one used last." {
    optional true
  }
  returns List.InputGlyph
}
```

<sub>`api.ogt` line 2040</sub>

<a id="fn-controlGlyph"></a>

### `og.input.controlGlyph`

A control's glyph (what a capture took, before it is applied). Null for a name that names no control.

```ts
og.input.controlGlyph(control: string, device?: InputDevice): InputGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `control` | `string` | no | The control. |
| `device` | `InputDevice` | yes | The device it is drawn for (a pad's family); omitted is xbox for a pad control. |

**Returns** `InputGlyph | null`.

Binding symbol `og_input_control_glyph` · publish builds: kept

```ogt
fn controlGlyph {
  doc "A control's glyph (what a capture took, before it is applied). Null for a name that names no control."
  param control String "The control."
  param device InputDevice "The device it is drawn for (a pad's family); omitted is xbox for a pad control." {
    optional true
  }
  returns Option.InputGlyph
}
```

<sub>`api.ogt` line 2050</sub>

<a id="fn-device"></a>

### `og.input.device`

The device the player used last: a key or the mouse, a pad (its family as the host told it, else xbox), or a touch. Prompts follow it live.

```ts
og.input.device(world: World): InputDevice
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `InputDevice`.

Binding symbol `og_input_device` · publish builds: kept

```ogt
fn device {
  doc "The device the player used last: a key or the mouse, a pad (its family as the host told it, else xbox), or a touch. Prompts follow it live."
  param world World "The world."
  returns InputDevice
}
```

<sub>`api.ogt` line 2059</sub>

<a id="fn-padDevice"></a>

### `og.input.padDevice`

A pad's family as the host told it from the pad's id (Xbox, PlayStation, Switch or a generic pad). Null for a pad that is not connected or that the host did not name.

```ts
og.input.padDevice(world: World, pad: number): InputDevice | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `pad` | `number` | no | The pad, 0 to 7. |

**Returns** `InputDevice | null`.

Binding symbol `og_input_pad_device` · publish builds: kept

```ogt
fn padDevice {
  doc "A pad's family as the host told it from the pad's id (Xbox, PlayStation, Switch or a generic pad). Null for a pad that is not connected or that the host did not name."
  param world World "The world."
  param pad u32 "The pad, 0 to 7."
  returns Option.InputDevice
}
```

<sub>`api.ogt` line 2065</sub>

<a id="fn-setDeadzones"></a>

### `og.input.setDeadzones`

A pad's dead zones (or every pad's): the player's setting, kept by og.input.saveBindings.

```ts
og.input.setDeadzones(world: World, desc?: DeadzoneDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `DeadzoneDesc` | yes | The pad and its dead zones. |

**Returns** `Result<void>`.

Binding symbol `og_input_set_deadzones` · publish builds: kept

```ogt
fn setDeadzones {
  doc "A pad's dead zones (or every pad's): the player's setting, kept by og.input.saveBindings."
  param world World "The world."
  param desc DeadzoneDesc "The pad and its dead zones."
  returns Result.Void
}
```

<sub>`api.ogt` line 2072</sub>

<a id="fn-setSensitivity"></a>

### `og.input.setSensitivity`

Set the player's sensitivity and inversion of an axis action on a device class (SensitivityDesc): applied from the next tick's actions, kept by og.input.saveBindings.

```ts
og.input.setSensitivity(world: World, desc?: SensitivityDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SensitivityDesc` | yes | The action, the device class, the scale and the inversions. |

**Returns** `Result<void>`: Refused for an unknown action, a button action, or a scale that is not above 0.

Binding symbol `og_input_set_sensitivity` · publish builds: kept

```ogt
fn setSensitivity {
  doc "Set the player's sensitivity and inversion of an axis action on a device class (SensitivityDesc): applied from the next tick's actions, kept by og.input.saveBindings."
  param world World "The world."
  param desc SensitivityDesc "The action, the device class, the scale and the inversions."
  returns Result.Void "Refused for an unknown action, a button action, or a scale that is not above 0."
}
```

<sub>`api.ogt` line 2079</sub>

<a id="fn-sensitivity"></a>

### `og.input.sensitivity`

An action's sensitivity on a device class (the device used last when omitted): the player's, else scale 1 and nothing inverted.

```ts
og.input.sensitivity(world: World, action: string, device?: InputDevice): SensitivityDesc
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `device` | `InputDevice` | yes | The device class; omitted is the one used last. |

**Returns** `SensitivityDesc`.

Binding symbol `og_input_sensitivity` · publish builds: kept

```ogt
fn sensitivity {
  doc "An action's sensitivity on a device class (the device used last when omitted): the player's, else scale 1 and nothing inverted."
  param world World "The world."
  param action String "The action."
  param device InputDevice "The device class; omitted is the one used last." {
    optional true
  }
  returns SensitivityDesc
}
```

<sub>`api.ogt` line 2086</sub>

<a id="fn-setKeyboardLayout"></a>

### `og.input.setKeyboardLayout`

The keys the player's keyboard layout prints, for glyphs: the web shell sets it from navigator.keyboard.getLayoutMap() (Chrome, Edge), and where the browser has none from the characters the player's unmodified keys type as they are pressed (Firefox, Safari: a key reads as on a US layout until it is pressed once). Bindings still name physical keys (KeyboardEvent.code), so WASD is the same four keys on every layout; only their glyphs change (Z Q S D on AZERTY). Empty is the US layout (native runs). The host's, one for the page: no world state.

```ts
og.input.setKeyboardLayout(labels: readonly KeyLabel[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `labels` | `readonly KeyLabel[]` | no | Each key's printed label, in place of the last list. |

**Returns** `void`.

Binding symbol `og_input_set_keyboard_layout` · publish builds: kept

```ogt
fn setKeyboardLayout {
  doc "The keys the player's keyboard layout prints, for glyphs: the web shell sets it from navigator.keyboard.getLayoutMap() (Chrome, Edge), and where the browser has none from the characters the player's unmodified keys type as they are pressed (Firefox, Safari: a key reads as on a US layout until it is pressed once). Bindings still name physical keys (KeyboardEvent.code), so WASD is the same four keys on every layout; only their glyphs change (Z Q S D on AZERTY). Empty is the US layout (native runs). The host's, one for the page: no world state."
  param labels List.KeyLabel "Each key's printed label, in place of the last list."
  returns Void
}
```

<sub>`api.ogt` line 2096</sub>

<a id="fn-keyboardLayout"></a>

### `og.input.keyboardLayout`

The keys the layout prints as og.input.setKeyboardLayout last set them, sorted by code (empty: the US layout).

```ts
og.input.keyboardLayout(): readonly KeyLabel[]
```

**Returns** `readonly KeyLabel[]`.

Binding symbol `og_input_keyboard_layout` · publish builds: kept

```ogt
fn keyboardLayout {
  doc "The keys the layout prints as og.input.setKeyboardLayout last set them, sorted by code (empty: the US layout)."
  returns List.KeyLabel
}
```

<sub>`api.ogt` line 2102</sub>

<a id="fn-setMouse"></a>

### `og.input.setMouse`

Set how the host reads the mouse (MouseDesc: raw input). Host only, never world state; fields left out keep their values.

```ts
og.input.setMouse(world: World, desc?: MouseDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `MouseDesc` | yes | The settings. |

**Returns** `Result<void>`: Refused for a world that is not alive.

Binding symbol `og_input_set_mouse` · publish builds: kept

```ogt
fn setMouse {
  doc "Set how the host reads the mouse (MouseDesc: raw input). Host only, never world state; fields left out keep their values."
  param world World "The world."
  param desc MouseDesc "The settings."
  returns Result.Void "Refused for a world that is not alive."
}
```

<sub>`api.ogt` line 2107</sub>

<a id="fn-mouse"></a>

### `og.input.mouse`

How the host reads the mouse: raw input asked for, granted or refused by the browser.

```ts
og.input.mouse(world: World): MouseInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `MouseInfo`.

Binding symbol `og_input_mouse` · publish builds: kept

```ogt
fn mouse {
  doc "How the host reads the mouse: raw input asked for, granted or refused by the browser."
  param world World "The world."
  returns MouseInfo
}
```

<sub>`api.ogt` line 2114</sub>

<a id="fn-rumble"></a>

### `og.input.rumble`

Rumble a pad (or every connected pad): the web shell plays it on the pad's vibration actuator. Output only: it never feeds back into the simulation, and a native run records it in the frame answer's `rumble`.

```ts
og.input.rumble(world: World, desc?: RumbleDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RumbleDesc` | yes | The pad, the motors and how long. |

**Returns** `Result<void>`: Refused for a pad outside -1 to 7 or a value that is not finite.

Binding symbol `og_input_rumble` · publish builds: kept

```ogt
fn rumble {
  doc "Rumble a pad (or every connected pad): the web shell plays it on the pad's vibration actuator. Output only: it never feeds back into the simulation, and a native run records it in the frame answer's `rumble`."
  param world World "The world."
  param desc RumbleDesc "The pad, the motors and how long."
  returns Result.Void "Refused for a pad outside -1 to 7 or a value that is not finite."
}
```

<sub>`api.ogt` line 2120</sub>

## Components, resources and events

<a id="record-Input"></a>

### `og.Input` (resource)

The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods.

Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.

Token `og.Input`, registered under the schema name `og.Input`; its value type is `Input`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pointer` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | The pointer position. |
| `pointerInside` | `boolean` | `false` | — | — | read only | Whether a pointer event has placed the pointer over the play surface. |
| `wheel` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Scroll accumulated during this tick. |
| `buttons` | `number` | `0` | — | — | read only | Held pointer buttons, as a mask. |
| `buttonsPressed` | `number` | `0` | — | — | read only | Buttons that went down this tick. |
| `buttonsReleased` | `number` | `0` | — | — | read only | Buttons that went up this tick. |
| `buttonsReleaseNext` | `number` | `0` | — | — | read only | Buttons a click releases at the end of the next tick. |
| `keys0` | `number` | `0` | — | — | read only | Held keys, word 0: codes 0 to 31 of KEY_CODES. |
| `keys1` | `number` | `0` | — | — | read only | Held keys, word 1: codes 32 to 63 of KEY_CODES. |
| `keys2` | `number` | `0` | — | — | read only | Held keys, word 2: codes 64 to 95 of KEY_CODES. |
| `keys3` | `number` | `0` | — | — | read only | Held keys, word 3: codes 96 to 127 of KEY_CODES. |
| `pressed0` | `number` | `0` | — | — | read only | Keys that went down this tick, word 0: codes 0 to 31 of KEY_CODES. |
| `pressed1` | `number` | `0` | — | — | read only | Keys that went down this tick, word 1: codes 32 to 63 of KEY_CODES. |
| `pressed2` | `number` | `0` | — | — | read only | Keys that went down this tick, word 2: codes 64 to 95 of KEY_CODES. |
| `pressed3` | `number` | `0` | — | — | read only | Keys that went down this tick, word 3: codes 96 to 127 of KEY_CODES. |
| `released0` | `number` | `0` | — | — | read only | Keys that went up this tick, word 0: codes 0 to 31 of KEY_CODES. |
| `released1` | `number` | `0` | — | — | read only | Keys that went up this tick, word 1: codes 32 to 63 of KEY_CODES. |
| `released2` | `number` | `0` | — | — | read only | Keys that went up this tick, word 2: codes 64 to 95 of KEY_CODES. |
| `released3` | `number` | `0` | — | — | read only | Keys that went up this tick, word 3: codes 96 to 127 of KEY_CODES. |
| `releaseNext0` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 0: codes 0 to 31 of KEY_CODES. |
| `releaseNext1` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 1: codes 32 to 63 of KEY_CODES. |
| `releaseNext2` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 2: codes 64 to 95 of KEY_CODES. |
| `releaseNext3` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 3: codes 96 to 127 of KEY_CODES. |

```ogt
resource Input {
  doc "The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods."
  doc "Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4."
  field pointer Vector2 "The pointer position." {
    readonly true
    unit cssPx
  }
  field pointerInside bool "Whether a pointer event has placed the pointer over the play surface." {
    readonly true
  }
  field wheel Vector2 "Scroll accumulated during this tick." {
    readonly true
    unit cssPx
  }
  field buttons u32 "Held pointer buttons, as a mask." {
    readonly true
  }
  field buttonsPressed u32 "Buttons that went down this tick." {
    readonly true
  }
  field buttonsReleased u32 "Buttons that went up this tick." {
    readonly true
  }
  field buttonsReleaseNext u32 "Buttons a click releases at the end of the next tick." {
    readonly true
  }
  field keys0 u32 "Held keys, word 0: codes 0 to 31 of KEY_CODES." {
    readonly true
  }
  field keys1 u32 "Held keys, word 1: codes 32 to 63 of KEY_CODES." {
    readonly true
  }
  field keys2 u32 "Held keys, word 2: codes 64 to 95 of KEY_CODES." {
    readonly true
  }
  field keys3 u32 "Held keys, word 3: codes 96 to 127 of KEY_CODES." {
    readonly true
  }
  field pressed0 u32 "Keys that went down this tick, word 0: codes 0 to 31 of KEY_CODES." {
    readonly true
  }
  field pressed1 u32 "Keys that went down this tick, word 1: codes 32 to 63 of KEY_CODES." {
    readonly true
  }
  field pressed2 u32 "Keys that went down this tick, word 2: codes 64 to 95 of KEY_CODES." {
    readonly true
  }
  field pressed3 u32 "Keys that went down this tick, word 3: codes 96 to 127 of KEY_CODES." {
    readonly true
  }
  field released0 u32 "Keys that went up this tick, word 0: codes 0 to 31 of KEY_CODES." {
    readonly true
  }
  field released1 u32 "Keys that went up this tick, word 1: codes 32 to 63 of KEY_CODES." {
    readonly true
  }
  field released2 u32 "Keys that went up this tick, word 2: codes 64 to 95 of KEY_CODES." {
    readonly true
  }
  field released3 u32 "Keys that went up this tick, word 3: codes 96 to 127 of KEY_CODES." {
    readonly true
  }
  field releaseNext0 u32 "Keys a press releases at the end of the next tick, word 0: codes 0 to 31 of KEY_CODES." {
    readonly true
  }
  field releaseNext1 u32 "Keys a press releases at the end of the next tick, word 1: codes 32 to 63 of KEY_CODES." {
    readonly true
  }
  field releaseNext2 u32 "Keys a press releases at the end of the next tick, word 2: codes 64 to 95 of KEY_CODES." {
    readonly true
  }
  field releaseNext3 u32 "Keys a press releases at the end of the next tick, word 3: codes 96 to 127 of KEY_CODES." {
    readonly true
  }
  fn down {
    doc "Whether a key is held."
    param key String "A KeyboardEvent.code name, such as KeyW or Space."
    returns bool
  }
  fn pressed {
    doc "Whether a key went down this tick."
    param key String "A KeyboardEvent.code name."
    returns bool
  }
  fn released {
    doc "Whether a key went up this tick."
    param key String "A KeyboardEvent.code name."
    returns bool
  }
  fn axis {
    doc "-1, 0 or 1 from two keys, such as `axis(\"KeyA\", \"KeyD\")`."
    param negative String "The key for -1."
    param positive String "The key for +1."
    returns f32
  }
  fn axis2 {
    doc "A 2D axis from four keys, such as `axis2(\"KeyA\", \"KeyD\", \"KeyS\", \"KeyW\")`: each component -1, 0 or 1."
    param left String "The key for x = -1."
    param right String "The key for x = +1."
    param down String "The key for y = -1."
    param up String "The key for y = +1."
    returns Vector2
  }
  fn buttonDown {
    doc "Whether a pointer button is held."
    param button PointerButton "The button."
    returns bool
  }
  fn buttonPressed {
    doc "Whether a pointer button went down this tick."
    param button PointerButton "The button."
    returns bool
  }
  fn buttonReleased {
    doc "Whether a pointer button went up this tick."
    param button PointerButton "The button."
    returns bool
  }
}
```

<sub>`api.ogt` line 2311</sub>

<a id="member-Input-down"></a>

#### `Input.down`

Whether a key is held.

```ts
Input.down(key: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | A KeyboardEvent.code name, such as KeyW or Space. |

**Returns** `boolean`.

Binding symbol `og_input_down` · publish builds: kept

```ogt
fn down {
  doc "Whether a key is held."
  param key String "A KeyboardEvent.code name, such as KeyW or Space."
  returns bool
}
```

<sub>`api.ogt` line 2385</sub>

<a id="member-Input-pressed"></a>

#### `Input.pressed`

Whether a key went down this tick.

```ts
Input.pressed(key: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | A KeyboardEvent.code name. |

**Returns** `boolean`.

Binding symbol `og_input_pressed` · publish builds: kept

```ogt
fn pressed {
  doc "Whether a key went down this tick."
  param key String "A KeyboardEvent.code name."
  returns bool
}
```

<sub>`api.ogt` line 2390</sub>

<a id="member-Input-released"></a>

#### `Input.released`

Whether a key went up this tick.

```ts
Input.released(key: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | A KeyboardEvent.code name. |

**Returns** `boolean`.

Binding symbol `og_input_released` · publish builds: kept

```ogt
fn released {
  doc "Whether a key went up this tick."
  param key String "A KeyboardEvent.code name."
  returns bool
}
```

<sub>`api.ogt` line 2395</sub>

<a id="member-Input-axis"></a>

#### `Input.axis`

-1, 0 or 1 from two keys, such as `axis("KeyA", "KeyD")`.

```ts
Input.axis(negative: string, positive: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `negative` | `string` | no | The key for -1. |
| `positive` | `string` | no | The key for +1. |

**Returns** `number`.

Binding symbol `og_input_axis` · publish builds: kept

```ogt
fn axis {
  doc "-1, 0 or 1 from two keys, such as `axis(\"KeyA\", \"KeyD\")`."
  param negative String "The key for -1."
  param positive String "The key for +1."
  returns f32
}
```

<sub>`api.ogt` line 2400</sub>

<a id="member-Input-axis2"></a>

#### `Input.axis2`

A 2D axis from four keys, such as `axis2("KeyA", "KeyD", "KeyS", "KeyW")`: each component -1, 0 or 1.

```ts
Input.axis2(left: string, right: string, down: string, up: string): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `left` | `string` | no | The key for x = -1. |
| `right` | `string` | no | The key for x = +1. |
| `down` | `string` | no | The key for y = -1. |
| `up` | `string` | no | The key for y = +1. |

**Returns** `Vector2`.

Binding symbol `og_input_axis2` · publish builds: kept

```ogt
fn axis2 {
  doc "A 2D axis from four keys, such as `axis2(\"KeyA\", \"KeyD\", \"KeyS\", \"KeyW\")`: each component -1, 0 or 1."
  param left String "The key for x = -1."
  param right String "The key for x = +1."
  param down String "The key for y = -1."
  param up String "The key for y = +1."
  returns Vector2
}
```

<sub>`api.ogt` line 2406</sub>

<a id="member-Input-buttonDown"></a>

#### `Input.buttonDown`

Whether a pointer button is held.

```ts
Input.buttonDown(button: PointerButton): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `PointerButton` | no | The button. |

**Returns** `boolean`.

Binding symbol `og_input_button_down` · publish builds: kept

```ogt
fn buttonDown {
  doc "Whether a pointer button is held."
  param button PointerButton "The button."
  returns bool
}
```

<sub>`api.ogt` line 2414</sub>

<a id="member-Input-buttonPressed"></a>

#### `Input.buttonPressed`

Whether a pointer button went down this tick.

```ts
Input.buttonPressed(button: PointerButton): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `PointerButton` | no | The button. |

**Returns** `boolean`.

Binding symbol `og_input_button_pressed` · publish builds: kept

```ogt
fn buttonPressed {
  doc "Whether a pointer button went down this tick."
  param button PointerButton "The button."
  returns bool
}
```

<sub>`api.ogt` line 2419</sub>

<a id="member-Input-buttonReleased"></a>

#### `Input.buttonReleased`

Whether a pointer button went up this tick.

```ts
Input.buttonReleased(button: PointerButton): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `PointerButton` | no | The button. |

**Returns** `boolean`.

Binding symbol `og_input_button_released` · publish builds: kept

```ogt
fn buttonReleased {
  doc "Whether a pointer button went up this tick."
  param button PointerButton "The button."
  returns bool
}
```

<sub>`api.ogt` line 2424</sub>

<a id="record-Pointers"></a>

### `og.Pointers` (resource)

Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id).

Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods.

Token `og.Pointers`, registered under the schema name `og.Pointers`; its value type is `Pointers`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `heldMask` | `number` | `0` | — | — | read only | Pointers held down, as a mask. |
| `pressedMask` | `number` | `0` | — | — | read only | Pointers that went down this tick. |
| `releasedMask` | `number` | `0` | — | — | read only | Pointers that went up this tick. |
| `releaseNextMask` | `number` | `0` | — | — | read only | Pointers a click releases at the end of the next tick. |
| `position0` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 0 is. |
| `position1` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 1 is. |
| `position2` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 2 is. |
| `position3` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 3 is. |
| `position4` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 4 is. |
| `start0` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 0 went down. |
| `start1` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 1 went down. |
| `start2` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 2 went down. |
| `start3` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 3 went down. |
| `start4` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Where pointer 4 went down. |
| `movement` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | The mouse's relative movement this tick: the sum of the movementX, movementY of pointer 0's events since the last tick (y down), zero again when the tick ends. Mouse-look reads it: it keeps coming while the pointer is locked (og.PointerLock), when the position stands still. The web shell fills it from the browser's movementX/Y; headless, only injected pointer events with movementX/Y move it. |

```ogt
resource Pointers {
  doc "Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id)."
  doc "Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods."
  field heldMask u32 "Pointers held down, as a mask." {
    readonly true
  }
  field pressedMask u32 "Pointers that went down this tick." {
    readonly true
  }
  field releasedMask u32 "Pointers that went up this tick." {
    readonly true
  }
  field releaseNextMask u32 "Pointers a click releases at the end of the next tick." {
    readonly true
  }
  field position0 Vector2 "Where pointer 0 is." {
    readonly true
    unit cssPx
  }
  field position1 Vector2 "Where pointer 1 is." {
    readonly true
    unit cssPx
  }
  field position2 Vector2 "Where pointer 2 is." {
    readonly true
    unit cssPx
  }
  field position3 Vector2 "Where pointer 3 is." {
    readonly true
    unit cssPx
  }
  field position4 Vector2 "Where pointer 4 is." {
    readonly true
    unit cssPx
  }
  field start0 Vector2 "Where pointer 0 went down." {
    readonly true
    unit cssPx
  }
  field start1 Vector2 "Where pointer 1 went down." {
    readonly true
    unit cssPx
  }
  field start2 Vector2 "Where pointer 2 went down." {
    readonly true
    unit cssPx
  }
  field start3 Vector2 "Where pointer 3 went down." {
    readonly true
    unit cssPx
  }
  field start4 Vector2 "Where pointer 4 went down." {
    readonly true
    unit cssPx
  }
  field movement Vector2 "The mouse's relative movement this tick: the sum of the movementX, movementY of pointer 0's events since the last tick (y down), zero again when the tick ends. Mouse-look reads it: it keeps coming while the pointer is locked (og.PointerLock), when the position stands still. The web shell fills it from the browser's movementX/Y; headless, only injected pointer events with movementX/Y move it." {
    readonly true
    unit cssPx
  }
  fn down {
    doc "Whether a pointer is held."
    param id u32 "The pointer, 0 to 4."
    returns bool
  }
  fn pressed {
    doc "Whether a pointer went down this tick."
    param id u32 "The pointer, 0 to 4."
    returns bool
  }
  fn released {
    doc "Whether a pointer went up this tick."
    param id u32 "The pointer, 0 to 4."
    returns bool
  }
  fn position {
    doc "Where a pointer is, in CSS pixels."
    param id u32 "The pointer, 0 to 4."
    returns Vector2
  }
  fn start {
    doc "Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it."
    param id u32 "The pointer, 0 to 4."
    returns Vector2
  }
  fn count {
    doc "How many pointers are held."
    returns u32
  }
}
```

<sub>`api.ogt` line 2431</sub>

<a id="member-Pointers-down"></a>

#### `Pointers.down`

Whether a pointer is held.

```ts
Pointers.down(id: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `boolean`.

Binding symbol `og_pointers_down` · publish builds: kept

```ogt
fn down {
  doc "Whether a pointer is held."
  param id u32 "The pointer, 0 to 4."
  returns bool
}
```

<sub>`api.ogt` line 2490</sub>

<a id="member-Pointers-pressed"></a>

#### `Pointers.pressed`

Whether a pointer went down this tick.

```ts
Pointers.pressed(id: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `boolean`.

Binding symbol `og_pointers_pressed` · publish builds: kept

```ogt
fn pressed {
  doc "Whether a pointer went down this tick."
  param id u32 "The pointer, 0 to 4."
  returns bool
}
```

<sub>`api.ogt` line 2495</sub>

<a id="member-Pointers-released"></a>

#### `Pointers.released`

Whether a pointer went up this tick.

```ts
Pointers.released(id: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `boolean`.

Binding symbol `og_pointers_released` · publish builds: kept

```ogt
fn released {
  doc "Whether a pointer went up this tick."
  param id u32 "The pointer, 0 to 4."
  returns bool
}
```

<sub>`api.ogt` line 2500</sub>

<a id="member-Pointers-position"></a>

#### `Pointers.position`

Where a pointer is, in CSS pixels.

```ts
Pointers.position(id: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `Vector2`.

Binding symbol `og_pointers_position` · publish builds: kept

```ogt
fn position {
  doc "Where a pointer is, in CSS pixels."
  param id u32 "The pointer, 0 to 4."
  returns Vector2
}
```

<sub>`api.ogt` line 2505</sub>

<a id="member-Pointers-start"></a>

#### `Pointers.start`

Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it.

```ts
Pointers.start(id: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `Vector2`.

Binding symbol `og_pointers_start` · publish builds: kept

```ogt
fn start {
  doc "Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it."
  param id u32 "The pointer, 0 to 4."
  returns Vector2
}
```

<sub>`api.ogt` line 2510</sub>

<a id="member-Pointers-count"></a>

#### `Pointers.count`

How many pointers are held.

```ts
Pointers.count(): number
```

**Returns** `number`.

Binding symbol `og_pointers_count` · publish builds: kept

```ogt
fn count {
  doc "How many pointers are held."
  returns u32
}
```

<sub>`api.ogt` line 2515</sub>

<a id="record-PointerLock"></a>

### `og.PointerLock` (resource)

Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default.

On the web the shell reads `wanted` from each frame and asks for the lock on the next press on the canvas (browsers grant it only to a gesture, so the click that starts play takes it), releases it when `wanted` goes false, and reports `locked` from pointerlockchange: the player's Escape releases it, and `locked` goes false while `wanted` stays true, so show a "click to look" prompt then. While locked, og.Input.pointer and og.Pointers' positions stand still (the UI's hover stays where it was) and the mouse's buttons still press. Native runs (og run, og dev) are headless: `locked` stays false unless a script injects `{ kind: "pointerLock", locked: true }`. `locked` is the host's (session state, as og.Input is: a restore keeps it, lockstep peers do not compare it).

Token `og.PointerLock`, registered under the schema name `og.PointerLock`; its value type is `PointerLock`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wanted` | `boolean` | `false` | — | — | — | The game wants the pointer locked and the cursor hidden. |
| `locked` | `boolean` | `false` | — | — | read only | The pointer is locked now, as the host last reported it. |

```ogt
resource PointerLock {
  doc "Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default."
  doc "On the web the shell reads `wanted` from each frame and asks for the lock on the next press on the canvas (browsers grant it only to a gesture, so the click that starts play takes it), releases it when `wanted` goes false, and reports `locked` from pointerlockchange: the player's Escape releases it, and `locked` goes false while `wanted` stays true, so show a \"click to look\" prompt then. While locked, og.Input.pointer and og.Pointers' positions stand still (the UI's hover stays where it was) and the mouse's buttons still press. Native runs (og run, og dev) are headless: `locked` stays false unless a script injects `{ kind: \"pointerLock\", locked: true }`. `locked` is the host's (session state, as og.Input is: a restore keeps it, lockstep peers do not compare it)."
  field wanted bool "The game wants the pointer locked and the cursor hidden."
  field locked bool "The pointer is locked now, as the host last reported it." {
    readonly true
  }
}
```

<sub>`api.ogt` line 2521</sub>

<a id="record-Gamepad"></a>

### `og.Gamepad` (resource)

Up to eight gamepads at the start of this tick (og-input's og.Gamepad, version 2), in the W3C standard layout: on the web the shell polls the pads each display frame, maps a pad the browser does not map (Firefox, Linux, many third-party pads) through its table, and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.

Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods. Sticks and triggers are raw: dead zones are the actions' (og.input.setDeadzones). `devices` holds each pad's family as the host told it (og.input.padDevice).

Token `og.Gamepad`, registered under the schema name `og.Gamepad`; its value type is `Gamepad`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `connectedMask` | `number` | `0` | — | — | read only | Connected pads: bit `pad`. |
| `devices` | `number` | `0` | — | — | read only | Each pad's family, four bits per pad (pad p at bit 4p): an InputDevice index (1 xbox, 2 playstation, 3 switch, 4 gamepad), 0 when the host did not say. |
| `buttons0` | `number` | `0` | — | — | read only | Held buttons of pad 0. |
| `buttons1` | `number` | `0` | — | — | read only | Held buttons of pad 1. |
| `buttons2` | `number` | `0` | — | — | read only | Held buttons of pad 2. |
| `buttons3` | `number` | `0` | — | — | read only | Held buttons of pad 3. |
| `buttons4` | `number` | `0` | — | — | read only | Held buttons of pad 4. |
| `buttons5` | `number` | `0` | — | — | read only | Held buttons of pad 5. |
| `buttons6` | `number` | `0` | — | — | read only | Held buttons of pad 6. |
| `buttons7` | `number` | `0` | — | — | read only | Held buttons of pad 7. |
| `pressed0` | `number` | `0` | — | — | read only | Buttons of pad 0 that went down this tick. |
| `pressed1` | `number` | `0` | — | — | read only | Buttons of pad 1 that went down this tick. |
| `pressed2` | `number` | `0` | — | — | read only | Buttons of pad 2 that went down this tick. |
| `pressed3` | `number` | `0` | — | — | read only | Buttons of pad 3 that went down this tick. |
| `pressed4` | `number` | `0` | — | — | read only | Buttons of pad 4 that went down this tick. |
| `pressed5` | `number` | `0` | — | — | read only | Buttons of pad 5 that went down this tick. |
| `pressed6` | `number` | `0` | — | — | read only | Buttons of pad 6 that went down this tick. |
| `pressed7` | `number` | `0` | — | — | read only | Buttons of pad 7 that went down this tick. |
| `released0` | `number` | `0` | — | — | read only | Buttons of pad 0 that went up this tick. |
| `released1` | `number` | `0` | — | — | read only | Buttons of pad 1 that went up this tick. |
| `released2` | `number` | `0` | — | — | read only | Buttons of pad 2 that went up this tick. |
| `released3` | `number` | `0` | — | — | read only | Buttons of pad 3 that went up this tick. |
| `released4` | `number` | `0` | — | — | read only | Buttons of pad 4 that went up this tick. |
| `released5` | `number` | `0` | — | — | read only | Buttons of pad 5 that went up this tick. |
| `released6` | `number` | `0` | — | — | read only | Buttons of pad 6 that went up this tick. |
| `released7` | `number` | `0` | — | — | read only | Buttons of pad 7 that went up this tick. |
| `leftStick0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 0, -1 to 1, y down. |
| `leftStick1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 1, -1 to 1, y down. |
| `leftStick2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 2, -1 to 1, y down. |
| `leftStick3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 3, -1 to 1, y down. |
| `leftStick4` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 4, -1 to 1, y down. |
| `leftStick5` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 5, -1 to 1, y down. |
| `leftStick6` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 6, -1 to 1, y down. |
| `leftStick7` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 7, -1 to 1, y down. |
| `rightStick0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 0, -1 to 1, y down. |
| `rightStick1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 1, -1 to 1, y down. |
| `rightStick2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 2, -1 to 1, y down. |
| `rightStick3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 3, -1 to 1, y down. |
| `rightStick4` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 4, -1 to 1, y down. |
| `rightStick5` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 5, -1 to 1, y down. |
| `rightStick6` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 6, -1 to 1, y down. |
| `rightStick7` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 7, -1 to 1, y down. |
| `triggers0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 0, 0 to 1. |
| `triggers1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 1, 0 to 1. |
| `triggers2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 2, 0 to 1. |
| `triggers3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 3, 0 to 1. |
| `triggers4` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 4, 0 to 1. |
| `triggers5` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 5, 0 to 1. |
| `triggers6` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 6, 0 to 1. |
| `triggers7` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 7, 0 to 1. |

```ogt
resource Gamepad {
  doc "Up to eight gamepads at the start of this tick (og-input's og.Gamepad, version 2), in the W3C standard layout: on the web the shell polls the pads each display frame, maps a pad the browser does not map (Firefox, Linux, many third-party pads) through its table, and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys."
  doc "Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods. Sticks and triggers are raw: dead zones are the actions' (og.input.setDeadzones). `devices` holds each pad's family as the host told it (og.input.padDevice)."
  field connectedMask u32 "Connected pads: bit `pad`." {
    readonly true
  }
  field devices u32 "Each pad's family, four bits per pad (pad p at bit 4p): an InputDevice index (1 xbox, 2 playstation, 3 switch, 4 gamepad), 0 when the host did not say." {
    readonly true
  }
  field buttons0 u32 "Held buttons of pad 0." {
    readonly true
  }
  field buttons1 u32 "Held buttons of pad 1." {
    readonly true
  }
  field buttons2 u32 "Held buttons of pad 2." {
    readonly true
  }
  field buttons3 u32 "Held buttons of pad 3." {
    readonly true
  }
  field buttons4 u32 "Held buttons of pad 4." {
    readonly true
  }
  field buttons5 u32 "Held buttons of pad 5." {
    readonly true
  }
  field buttons6 u32 "Held buttons of pad 6." {
    readonly true
  }
  field buttons7 u32 "Held buttons of pad 7." {
    readonly true
  }
  field pressed0 u32 "Buttons of pad 0 that went down this tick." {
    readonly true
  }
  field pressed1 u32 "Buttons of pad 1 that went down this tick." {
    readonly true
  }
  field pressed2 u32 "Buttons of pad 2 that went down this tick." {
    readonly true
  }
  field pressed3 u32 "Buttons of pad 3 that went down this tick." {
    readonly true
  }
  field pressed4 u32 "Buttons of pad 4 that went down this tick." {
    readonly true
  }
  field pressed5 u32 "Buttons of pad 5 that went down this tick." {
    readonly true
  }
  field pressed6 u32 "Buttons of pad 6 that went down this tick." {
    readonly true
  }
  field pressed7 u32 "Buttons of pad 7 that went down this tick." {
    readonly true
  }
  field released0 u32 "Buttons of pad 0 that went up this tick." {
    readonly true
  }
  field released1 u32 "Buttons of pad 1 that went up this tick." {
    readonly true
  }
  field released2 u32 "Buttons of pad 2 that went up this tick." {
    readonly true
  }
  field released3 u32 "Buttons of pad 3 that went up this tick." {
    readonly true
  }
  field released4 u32 "Buttons of pad 4 that went up this tick." {
    readonly true
  }
  field released5 u32 "Buttons of pad 5 that went up this tick." {
    readonly true
  }
  field released6 u32 "Buttons of pad 6 that went up this tick." {
    readonly true
  }
  field released7 u32 "Buttons of pad 7 that went up this tick." {
    readonly true
  }
  field leftStick0 Vector2 "The left stick of pad 0, -1 to 1, y down." {
    readonly true
  }
  field leftStick1 Vector2 "The left stick of pad 1, -1 to 1, y down." {
    readonly true
  }
  field leftStick2 Vector2 "The left stick of pad 2, -1 to 1, y down." {
    readonly true
  }
  field leftStick3 Vector2 "The left stick of pad 3, -1 to 1, y down." {
    readonly true
  }
  field leftStick4 Vector2 "The left stick of pad 4, -1 to 1, y down." {
    readonly true
  }
  field leftStick5 Vector2 "The left stick of pad 5, -1 to 1, y down." {
    readonly true
  }
  field leftStick6 Vector2 "The left stick of pad 6, -1 to 1, y down." {
    readonly true
  }
  field leftStick7 Vector2 "The left stick of pad 7, -1 to 1, y down." {
    readonly true
  }
  field rightStick0 Vector2 "The right stick of pad 0, -1 to 1, y down." {
    readonly true
  }
  field rightStick1 Vector2 "The right stick of pad 1, -1 to 1, y down." {
    readonly true
  }
  field rightStick2 Vector2 "The right stick of pad 2, -1 to 1, y down." {
    readonly true
  }
  field rightStick3 Vector2 "The right stick of pad 3, -1 to 1, y down." {
    readonly true
  }
  field rightStick4 Vector2 "The right stick of pad 4, -1 to 1, y down." {
    readonly true
  }
  field rightStick5 Vector2 "The right stick of pad 5, -1 to 1, y down." {
    readonly true
  }
  field rightStick6 Vector2 "The right stick of pad 6, -1 to 1, y down." {
    readonly true
  }
  field rightStick7 Vector2 "The right stick of pad 7, -1 to 1, y down." {
    readonly true
  }
  field triggers0 Vector2 "The left (x) and right (y) triggers of pad 0, 0 to 1." {
    readonly true
  }
  field triggers1 Vector2 "The left (x) and right (y) triggers of pad 1, 0 to 1." {
    readonly true
  }
  field triggers2 Vector2 "The left (x) and right (y) triggers of pad 2, 0 to 1." {
    readonly true
  }
  field triggers3 Vector2 "The left (x) and right (y) triggers of pad 3, 0 to 1." {
    readonly true
  }
  field triggers4 Vector2 "The left (x) and right (y) triggers of pad 4, 0 to 1." {
    readonly true
  }
  field triggers5 Vector2 "The left (x) and right (y) triggers of pad 5, 0 to 1." {
    readonly true
  }
  field triggers6 Vector2 "The left (x) and right (y) triggers of pad 6, 0 to 1." {
    readonly true
  }
  field triggers7 Vector2 "The left (x) and right (y) triggers of pad 7, 0 to 1." {
    readonly true
  }
  fn connected {
    doc "Whether a pad is connected."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn down {
    doc "Whether a button is held."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn pressed {
    doc "Whether a button went down this tick."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn released {
    doc "Whether a button went up this tick."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn axis {
    doc "An analog input, raw: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Actions read sticks past their dead zones and y up (og.input.defineAction)."
    param axis GamepadAxis "Which input."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns f32
  }
}
```

<sub>`api.ogt` line 2530</sub>

<a id="member-Gamepad-connected"></a>

#### `Gamepad.connected`

Whether a pad is connected.

```ts
Gamepad.connected(pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_connected` · publish builds: kept

```ogt
fn connected {
  doc "Whether a pad is connected."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 2683</sub>

<a id="member-Gamepad-down"></a>

#### `Gamepad.down`

Whether a button is held.

```ts
Gamepad.down(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_down` · publish builds: kept

```ogt
fn down {
  doc "Whether a button is held."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 2690</sub>

<a id="member-Gamepad-pressed"></a>

#### `Gamepad.pressed`

Whether a button went down this tick.

```ts
Gamepad.pressed(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_pressed` · publish builds: kept

```ogt
fn pressed {
  doc "Whether a button went down this tick."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 2698</sub>

<a id="member-Gamepad-released"></a>

#### `Gamepad.released`

Whether a button went up this tick.

```ts
Gamepad.released(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_released` · publish builds: kept

```ogt
fn released {
  doc "Whether a button went up this tick."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 2706</sub>

<a id="member-Gamepad-axis"></a>

#### `Gamepad.axis`

An analog input, raw: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Actions read sticks past their dead zones and y up (og.input.defineAction).

```ts
Gamepad.axis(axis: GamepadAxis, pad?: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `axis` | `GamepadAxis` | no | Which input. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `number`.

Binding symbol `og_gamepad_axis` · publish builds: kept

```ogt
fn axis {
  doc "An analog input, raw: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Actions read sticks past their dead zones and y up (og.input.defineAction)."
  param axis GamepadAxis "Which input."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns f32
}
```

<sub>`api.ogt` line 2714</sub>

<a id="record-KeyDown"></a>

### `og.KeyDown` (event)

A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.

Token `og.KeyDown`, registered under the schema name `og.KeyDown`; its value type is `KeyDown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `repeat` | `boolean` | `false` | — | — | — | Whether this is an auto-repeat. |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it. |

```ogt
event KeyDown {
  doc "A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems."
  field key String "The physical key code."
  field repeat bool "Whether this is an auto-repeat."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it." {
    min -1
    max 1
  }
}
```

<sub>`api.ogt` line 2724</sub>

<a id="record-TextInput"></a>

### `og.TextInput` (event)

Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one.

Token `og.TextInput`, registered under the schema name `og.TextInput`; its value type is `TextInput`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text (whole characters). |
| `composing` | `boolean` | `false` | — | — | — | An input method's composition so far, not committed: it replaces the composition before it (empty ends it). False: committed text to insert. |
| `more` | `boolean` | `false` | — | — | — | Continues the composition of the event before it (a composition longer than one event). |

```ogt
event TextInput {
  doc "Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one."
  field text String "The text (whole characters)."
  field composing bool "An input method's composition so far, not committed: it replaces the composition before it (empty ends it). False: committed text to insert."
  field more bool "Continues the composition of the event before it (a composition longer than one event)."
}
```

<sub>`api.ogt` line 2734</sub>

<a id="record-KeyUp"></a>

### `og.KeyUp` (event)

A key went up.

Token `og.KeyUp`, registered under the schema name `og.KeyUp`; its value type is `KeyUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it. |

```ogt
event KeyUp {
  doc "A key went up."
  field key String "The physical key code."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it." {
    min -1
    max 1
  }
}
```

<sub>`api.ogt` line 2741</sub>

<a id="record-PointerDown"></a>

### `og.PointerDown` (event)

A pointer button went down (or a touch landed).

Token `og.PointerDown`, registered under the schema name `og.PointerDown`; its value type is `PointerDown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button. |
| `id` | `number` | `0` | — | — | — | Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers). |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it. |

```ogt
event PointerDown {
  doc "A pointer button went down (or a touch landed)."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers)."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it." {
    min -1
    max 1
  }
}
```

<sub>`api.ogt` line 2750</sub>

<a id="record-PointerUp"></a>

### `og.PointerUp` (event)

A pointer button went up (or a touch lifted).

Token `og.PointerUp`, registered under the schema name `og.PointerUp`; its value type is `PointerUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button. |
| `id` | `number` | `0` | — | — | — | Which pointer: 0 the primary one, 1 to 4 further touches. |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it. |

```ogt
event PointerUp {
  doc "A pointer button went up (or a touch lifted)."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it." {
    min -1
    max 1
  }
}
```

<sub>`api.ogt` line 2762</sub>

<a id="record-PointerMove"></a>

### `og.PointerMove` (event)

A pointer moved.

Token `og.PointerMove`, registered under the schema name `og.PointerMove`; its value type is `PointerMove`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `id` | `number` | `0` | — | — | — | Which pointer: 0 the primary one, 1 to 4 further touches. |
| `movementX` | `number` | `0` | — | — | — | The relative movement this event carried (movementX), CSS pixels; 0 when the host gave none. While the pointer is locked x and y stand still and only this moves. |
| `movementY` | `number` | `0` | — | — | — | The relative vertical movement (movementY, down positive), CSS pixels. |

```ogt
event PointerMove {
  doc "A pointer moved."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches."
  field movementX f32 "The relative movement this event carried (movementX), CSS pixels; 0 when the host gave none. While the pointer is locked x and y stand still and only this moves."
  field movementY f32 "The relative vertical movement (movementY, down positive), CSS pixels."
}
```

<sub>`api.ogt` line 2774</sub>

<a id="record-Wheel"></a>

### `og.Wheel` (event)

A wheel or trackpad scroll.

Token `og.Wheel`, registered under the schema name `og.Wheel`; its value type is `Wheel`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `deltaX` | `number` | `0` | — | — | — | Horizontal scroll in CSS pixels. |
| `deltaY` | `number` | `0` | — | — | — | Vertical scroll in CSS pixels. |

```ogt
event Wheel {
  doc "A wheel or trackpad scroll."
  field deltaX f32 "Horizontal scroll in CSS pixels."
  field deltaY f32 "Vertical scroll in CSS pixels."
}
```

<sub>`api.ogt` line 2783</sub>

<a id="record-A11yAction"></a>

### `og.A11yAction` (event)

A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused.

Token `og.A11yAction`, registered under the schema name `og.A11yAction`; its value type is `A11yAction`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `A11yActionKind` | `"focus"` | — | — | — | What is asked. |
| `path` | `string` | `""` | — | — | — | The control's path, as og.ui.a11y() names it. |

```ogt
event A11yAction {
  doc "A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused."
  field action A11yActionKind "What is asked."
  field path String "The control's path, as og.ui.a11y() names it."
}
```

<sub>`api.ogt` line 2789</sub>

## Types

<a id="type-InputDevice"></a>

### `InputDevice` (enum)

A kind of input device: the set a glyph is drawn from, the device used last, and what a pad is. og.ui's UiDevice has the same values.

A string: one of the variants below. The zero value is `"keyboardMouse"`.

| Variant | Doc |
|---|---|
| `"keyboardMouse"` | Keyboard and mouse. |
| `"xbox"` | An Xbox pad (and the family a pad whose kind the host could not tell is drawn as). |
| `"playstation"` | A PlayStation pad. |
| `"switch"` | A Nintendo Switch pad (its south button is B). |
| `"gamepad"` | A generic pad: glyphs name positions (South, East). |
| `"touch"` | A touch screen. |

```ogt
enum InputDevice {
  doc "A kind of input device: the set a glyph is drawn from, the device used last, and what a pad is. og.ui's UiDevice has the same values."
  variant keyboardMouse "Keyboard and mouse."
  variant xbox "An Xbox pad (and the family a pad whose kind the host could not tell is drawn as)."
  variant playstation "A PlayStation pad."
  variant switch "A Nintendo Switch pad (its south button is B)."
  variant gamepad "A generic pad: glyphs name positions (South, East)."
  variant touch "A touch screen."
}
```

<sub>`api.ogt` line 1609</sub>

<a id="type-ActionKind"></a>

### `ActionKind` (enum)

What an action gives.

A string: one of the variants below. The zero value is `"button"`.

| Variant | Doc |
|---|---|
| `"button"` | On or off: down, pressed, released, held. An analog control (a trigger, a half axis) presses it at ActionDesc.threshold and lets go under three quarters of it. |
| `"axis"` | One value, -1 to 1: A and D, a stick's x, a trigger (0 to 1), the wheel, the mouse's x movement (CSS px). |
| `"axis2"` | Two values, x right and y up: WASD, a stick, the d-pad, a touch stick, the mouse's movement. |

```ogt
enum ActionKind {
  doc "What an action gives."
  variant button "On or off: down, pressed, released, held. An analog control (a trigger, a half axis) presses it at ActionDesc.threshold and lets go under three quarters of it."
  variant axis "One value, -1 to 1: A and D, a stick's x, a trigger (0 to 1), the wheel, the mouse's x movement (CSS px)."
  variant axis2 "Two values, x right and y up: WASD, a stick, the d-pad, a touch stick, the mouse's movement."
}
```

<sub>`api.ogt` line 1619</sub>

<a id="type-TouchZone"></a>

### `TouchZone` (desc struct)

Where on the play surface a touch binding's touches must start, in fractions of it from the top left; a zone of no size is all of it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | 0 … 1 | `ratio` | — | Its left edge. |
| `y` | `number` | `0` | 0 … 1 | `ratio` | — | Its top edge. |
| `width` | `number` | `0` | 0 … 1 | `ratio` | — | Its width (0: the whole surface). |
| `height` | `number` | `0` | 0 … 1 | `ratio` | — | Its height (0: the whole surface). |

```ogt
struct TouchZone {
  doc "Where on the play surface a touch binding's touches must start, in fractions of it from the top left; a zone of no size is all of it."
  desc true
  field x f32 "Its left edge." {
    min 0
    max 1
    unit ratio
  }
  field y f32 "Its top edge." {
    min 0
    max 1
    unit ratio
  }
  field width f32 "Its width (0: the whole surface)." {
    min 0
    max 1
    unit ratio
  }
  field height f32 "Its height (0: the whole surface)." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>`api.ogt` line 1626</sub>

<a id="type-InputBinding"></a>

### `InputBinding` (desc struct)

One binding of an action: a control, or a composite of parts (A and D as one axis, WASD as two). Controls are named by strings: a key code (`KeyW`, `Space`), `mouse.left`, `mouse.middle`, `mouse.right`, `mouse.x`, `mouse.y`, `mouse.xy` (the mouse's movement this tick, CSS px), `wheel.x`, `wheel.y` (notches of 100 CSS px), `pad.a` to `pad.home` (GamepadButton's names, any pad), `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY`, `pad.leftTrigger`, `pad.rightTrigger`, `pad.leftXY`, `pad.rightXY` (a stick), `pad.dpad`, `touch` (a touch held) and `touch.xy` (a touch stick). A one-axis control takes `+` or `-` for its half (`pad.leftX+`: 0 to 1, a direction as a button).

Every y is up-positive in actions (W, the stick pushed up, the mouse moved up, the wheel scrolled up, a touch dragged up: +1), whatever the raw og.Gamepad says. A binding with neither a control nor parts is an unbound slot.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `control` | `string` | `""` | — | — | — | The control; empty for a composite or an unbound slot. |
| `negative` | `string` | `""` | — | — | — | An axis composite's control for -1 (`KeyA`). |
| `positive` | `string` | `""` | — | — | — | An axis composite's control for +1 (`KeyD`). |
| `up` | `string` | `""` | — | — | — | A two-axis composite's control for y = 1 (`KeyW`). |
| `down` | `string` | `""` | — | — | — | A two-axis composite's control for y = -1. |
| `left` | `string` | `""` | — | — | — | A two-axis composite's control for x = -1. |
| `right` | `string` | `""` | — | — | — | A two-axis composite's control for x = 1. |
| `scale` | `number` | `1` | — | — | — | Multiplies the value: the binding's sensitivity (a mouse look's 0.1, a stick's 2); negative inverts it. |
| `invertY` | `boolean` | `false` | — | — | — | A two-axis value's y is turned over (inverted look). |
| `deadzone` | `number` | `0` | 0 … 0.95 | — | — | A pad axis's dead zone for this binding, over the pad's (og.input.setDeadzones) when above 0: values inside it read 0 and the rest is rescaled to reach 1. |
| `zone` | `TouchZone` | `{}` | — | — | — | `touch`, `touch.xy`: where a touch must start to count (empty: anywhere). |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | `touch.xy`: how far a touch drags for a full deflection (0: 64). |

```ogt
struct InputBinding {
  doc "One binding of an action: a control, or a composite of parts (A and D as one axis, WASD as two). Controls are named by strings: a key code (`KeyW`, `Space`), `mouse.left`, `mouse.middle`, `mouse.right`, `mouse.x`, `mouse.y`, `mouse.xy` (the mouse's movement this tick, CSS px), `wheel.x`, `wheel.y` (notches of 100 CSS px), `pad.a` to `pad.home` (GamepadButton's names, any pad), `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY`, `pad.leftTrigger`, `pad.rightTrigger`, `pad.leftXY`, `pad.rightXY` (a stick), `pad.dpad`, `touch` (a touch held) and `touch.xy` (a touch stick). A one-axis control takes `+` or `-` for its half (`pad.leftX+`: 0 to 1, a direction as a button)."
  doc "Every y is up-positive in actions (W, the stick pushed up, the mouse moved up, the wheel scrolled up, a touch dragged up: +1), whatever the raw og.Gamepad says. A binding with neither a control nor parts is an unbound slot."
  desc true
  field control String "The control; empty for a composite or an unbound slot."
  field negative String "An axis composite's control for -1 (`KeyA`)."
  field positive String "An axis composite's control for +1 (`KeyD`)."
  field up String "A two-axis composite's control for y = 1 (`KeyW`)."
  field down String "A two-axis composite's control for y = -1."
  field left String "A two-axis composite's control for x = -1."
  field right String "A two-axis composite's control for x = 1."
  field scale f32 "Multiplies the value: the binding's sensitivity (a mouse look's 0.1, a stick's 2); negative inverts it." {
    default 1
  }
  field invertY bool "A two-axis value's y is turned over (inverted look)."
  field deadzone f32 "A pad axis's dead zone for this binding, over the pad's (og.input.setDeadzones) when above 0: values inside it read 0 and the rest is rescaled to reach 1." {
    min 0
    max 0.95
  }
  field zone TouchZone "`touch`, `touch.xy`: where a touch must start to count (empty: anywhere)."
  field radius f32 "`touch.xy`: how far a touch drags for a full deflection (0: 64)." {
    min 0
    unit cssPx
  }
}
```

<sub>`api.ogt` line 1651</sub>

<a id="type-ActionDesc"></a>

### `ActionDesc` (desc struct)

A gameplay action: a name the game reads instead of keys (`jump`, `move`, `look`), its kind and default bindings over keys, the mouse, pads and touch. Actions are read once per tick, before the game's systems, from og.Input, og.Pointers and og.Gamepad, so they replay with them; the player rebinds them (og.input.rebind, og.input.startCapture) and a settings screen lists them (og.input.actions).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `ActionKind` | `"button"` | — | — | — | Button, one axis or two. |
| `context` | `string` | `"gameplay"` | — | — | — | Its context (og.input.setContexts): `gameplay`, `menu`, `vehicle`, or the game's own. |
| `label` | `string` | `""` | — | — | — | What a settings screen and prompts call it (empty: its name). |
| `bindings` | `readonly InputBinding[]` | `[]` | — | — | — | The default bindings, in slot order: the player's rebinding replaces a slot, reset brings these back. |
| `threshold` | `number` | `0.5` | 0.01 … 1 | — | — | Where an analog value presses it (a button; and an axis's down). |
| `deadzone` | `number` | `0` | 0 … 0.95 | — | — | A dead zone on its combined value (radial for two axes), after each binding's own. |
| `holdMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it is down before `held` (0: never), counted in ticks. |
| `locked` | `boolean` | `false` | — | — | — | The player cannot rebind it (pause, a debug key). |
| `toggle` | `boolean` | `false` | — | — | — | A button whose press turns it on and the next press off (a hold's alternative, XAG 107); the player may change it (og.input.setToggle). |

```ogt
struct ActionDesc {
  doc "A gameplay action: a name the game reads instead of keys (`jump`, `move`, `look`), its kind and default bindings over keys, the mouse, pads and touch. Actions are read once per tick, before the game's systems, from og.Input, og.Pointers and og.Gamepad, so they replay with them; the player rebinds them (og.input.rebind, og.input.startCapture) and a settings screen lists them (og.input.actions)."
  desc true
  field name String "Its name."
  field kind ActionKind "Button, one axis or two."
  field context String "Its context (og.input.setContexts): `gameplay`, `menu`, `vehicle`, or the game's own." {
    default "gameplay"
  }
  field label String "What a settings screen and prompts call it (empty: its name)."
  field bindings List.InputBinding "The default bindings, in slot order: the player's rebinding replaces a slot, reset brings these back."
  field threshold f32 "Where an analog value presses it (a button; and an axis's down)." {
    default 0.5
    min 0.01
    max 1
  }
  field deadzone f32 "A dead zone on its combined value (radial for two axes), after each binding's own." {
    min 0
    max 0.95
  }
  field holdMs f32 "How long it is down before `held` (0: never), counted in ticks." {
    min 0
    unit ms
  }
  field locked bool "The player cannot rebind it (pause, a debug key)."
  field toggle bool "A button whose press turns it on and the next press off (a hold's alternative, XAG 107); the player may change it (og.input.setToggle)."
}
```

<sub>`api.ogt` line 1677</sub>

<a id="type-ActionState"></a>

### `ActionState` (struct)

An action this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `down` | `boolean` | `false` | — | — | — | Its binding is past the threshold (or its toggle is on). |
| `pressed` | `boolean` | `false` | — | — | — | It went down this tick (a press and a release between two ticks set both edges). |
| `released` | `boolean` | `false` | — | — | — | It went up this tick. |
| `held` | `boolean` | `false` | — | — | — | Down for its holdMs. |
| `value` | `number` | `0` | — | — | — | A button's strength (0 to 1: a trigger's travel), an axis's value, a two-axis value's length. |
| `vector` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | A two-axis action's value (x right, y up); value and 0 for the others. |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | — | The device whose binding drove it last. |

```ogt
struct ActionState {
  doc "An action this tick."
  field down bool "Its binding is past the threshold (or its toggle is on)."
  field pressed bool "It went down this tick (a press and a release between two ticks set both edges)."
  field released bool "It went up this tick."
  field held bool "Down for its holdMs."
  field value f32 "A button's strength (0 to 1: a trigger's travel), an axis's value, a two-axis value's length."
  field vector Vector2 "A two-axis action's value (x right, y up); value and 0 for the others."
  field device InputDevice "The device whose binding drove it last."
}
```

<sub>`api.ogt` line 1704</sub>

<a id="type-ActionInfo"></a>

### `ActionInfo` (struct)

An action as a controls screen lists it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `ActionKind` | `"button"` | — | — | — | Its kind. |
| `context` | `string` | `""` | — | — | — | Its context. |
| `label` | `string` | `""` | — | — | — | Its label (its name when it has none). |
| `bindings` | `readonly InputBinding[]` | `[]` | — | — | — | Its bindings now: the player's, else the defaults. |
| `defaults` | `readonly InputBinding[]` | `[]` | — | — | — | Its default bindings. |
| `locked` | `boolean` | `false` | — | — | — | The player cannot rebind it. |
| `toggle` | `boolean` | `false` | — | — | — | It toggles (the player's choice, else its desc's). |
| `rebound` | `boolean` | `false` | — | — | — | The player changed its bindings. |

```ogt
struct ActionInfo {
  doc "An action as a controls screen lists it."
  field name String "Its name."
  field kind ActionKind "Its kind."
  field context String "Its context."
  field label String "Its label (its name when it has none)."
  field bindings List.InputBinding "Its bindings now: the player's, else the defaults."
  field defaults List.InputBinding "Its default bindings."
  field locked bool "The player cannot rebind it."
  field toggle bool "It toggles (the player's choice, else its desc's)."
  field rebound bool "The player changed its bindings."
}
```

<sub>`api.ogt` line 1715</sub>

<a id="type-BindingConflict"></a>

### `BindingConflict` (struct)

Another binding a rebinding would share a control with: an action of the same context (or another slot of the same action).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The other action. |
| `slot` | `number` | `0` | — | — | — | Its binding's slot. |
| `control` | `string` | `""` | — | — | — | The shared control, as the new binding names it. |
| `context` | `string` | `""` | — | — | — | The context they share. |

```ogt
struct BindingConflict {
  doc "Another binding a rebinding would share a control with: an action of the same context (or another slot of the same action)."
  field action String "The other action."
  field slot u32 "Its binding's slot."
  field control String "The shared control, as the new binding names it."
  field context String "The context they share."
}
```

<sub>`api.ogt` line 1728</sub>

<a id="type-ConflictPolicy"></a>

### `ConflictPolicy` (enum)

What a rebinding does with a control another action holds.

A string: one of the variants below. The zero value is `"refuse"`.

| Variant | Doc |
|---|---|
| `"refuse"` | Change nothing and answer the conflicts (a controls screen asks the player). |
| `"swap"` | The other action takes what this slot had. |
| `"replace"` | The other action loses the control (its slot is left unbound). |
| `"allow"` | Bind it anyway: both actions read it. |

```ogt
enum ConflictPolicy {
  doc "What a rebinding does with a control another action holds."
  variant refuse "Change nothing and answer the conflicts (a controls screen asks the player)."
  variant swap "The other action takes what this slot had."
  variant replace "The other action loses the control (its slot is left unbound)."
  variant allow "Bind it anyway: both actions read it."
}
```

<sub>`api.ogt` line 1736</sub>

<a id="type-RebindDesc"></a>

### `RebindDesc` (desc struct)

A player's rebinding of one slot of an action.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action. |
| `slot` | `number` | `0` | — | — | — | Which binding: 0 the first; one past the last adds one. |
| `part` | `string` | `""` | — | — | — | A composite's part to change (`up`, `down`, `left`, `right`, `negative`, `positive`); empty sets the whole binding. |
| `binding` | `InputBinding` | `{}` | — | — | — | The new binding; for a part, its control is the part's. |
| `policy` | `ConflictPolicy` | `"refuse"` | — | — | — | What to do with a control another action holds. |

```ogt
struct RebindDesc {
  doc "A player's rebinding of one slot of an action."
  desc true
  field action String "The action."
  field slot u32 "Which binding: 0 the first; one past the last adds one."
  field part String "A composite's part to change (`up`, `down`, `left`, `right`, `negative`, `positive`); empty sets the whole binding."
  field binding InputBinding "The new binding; for a part, its control is the part's."
  field policy ConflictPolicy "What to do with a control another action holds."
}
```

<sub>`api.ogt` line 1744</sub>

<a id="type-RebindAnswer"></a>

### `RebindAnswer` (struct)

What a rebinding did.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `applied` | `boolean` | `false` | — | — | — | The binding changed (false: refused over conflicts). |
| `conflicts` | `readonly BindingConflict[]` | `[]` | — | — | — | The bindings it shared a control with. |

```ogt
struct RebindAnswer {
  doc "What a rebinding did."
  field applied bool "The binding changed (false: refused over conflicts)."
  field conflicts List.BindingConflict "The bindings it shared a control with."
}
```

<sub>`api.ogt` line 1754</sub>

<a id="type-CaptureAccept"></a>

### `CaptureAccept` (enum)

Which devices a capture takes.

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | Keyboard, mouse and pads. |
| `"keyboardMouse"` | Keyboard and mouse. |
| `"gamepad"` | Pads. |

```ogt
enum CaptureAccept {
  doc "Which devices a capture takes."
  variant any "Keyboard, mouse and pads."
  variant keyboardMouse "Keyboard and mouse."
  variant gamepad "Pads."
}
```

<sub>`api.ogt` line 1760</sub>

<a id="type-CaptureDesc"></a>

### `CaptureDesc` (desc struct)

A capture of the player's next input for a slot of an action: a controls screen's "press a key". From the next tick every action reads nothing until it ends; what was pressed for it moves no action until it is let go.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action. |
| `slot` | `number` | `0` | — | — | — | Which binding: 0 the first; one past the last adds one. |
| `part` | `string` | `""` | — | — | — | A composite's part (`up` of WASD); empty: the whole binding (a two-axis action then takes a stick or the d-pad). |
| `accept` | `CaptureAccept` | `"any"` | — | — | — | The devices it takes. |
| `cancel` | `readonly string[]` | `[]` | — | — | — | Controls that cancel it (empty: `Escape` and `pad.back`). |
| `timeoutMs` | `number` | `0` | ≥ 0 | `ms` | — | It cancels itself after this long (0: never), counted in ticks. |

```ogt
struct CaptureDesc {
  doc "A capture of the player's next input for a slot of an action: a controls screen's \"press a key\". From the next tick every action reads nothing until it ends; what was pressed for it moves no action until it is let go."
  desc true
  field action String "The action."
  field slot u32 "Which binding: 0 the first; one past the last adds one."
  field part String "A composite's part (`up` of WASD); empty: the whole binding (a two-axis action then takes a stick or the d-pad)."
  field accept CaptureAccept "The devices it takes."
  field cancel List.String "Controls that cancel it (empty: `Escape` and `pad.back`)."
  field timeoutMs f32 "It cancels itself after this long (0: never), counted in ticks." {
    min 0
    unit ms
  }
}
```

<sub>`api.ogt` line 1767</sub>

<a id="type-CaptureState"></a>

### `CaptureState` (struct)

Where a capture is: waiting, done with the control it took, or cancelled.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `active` | `boolean` | `false` | — | — | — | It waits for input. |
| `done` | `boolean` | `false` | — | — | — | It took a control: apply it with og.input.applyCapture (or show the conflicts first). |
| `cancelled` | `boolean` | `false` | — | — | — | The player cancelled it, or it timed out. |
| `action` | `string` | `""` | — | — | — | The action. |
| `slot` | `number` | `0` | — | — | — | The slot. |
| `part` | `string` | `""` | — | — | — | The part. |
| `control` | `string` | `""` | — | — | — | The control it took. |
| `binding` | `InputBinding` | `{}` | — | — | — | The slot's binding with that control in it. |
| `conflicts` | `readonly BindingConflict[]` | `[]` | — | — | — | What applying it would share a control with. |

```ogt
struct CaptureState {
  doc "Where a capture is: waiting, done with the control it took, or cancelled."
  field active bool "It waits for input."
  field done bool "It took a control: apply it with og.input.applyCapture (or show the conflicts first)."
  field cancelled bool "The player cancelled it, or it timed out."
  field action String "The action."
  field slot u32 "The slot."
  field part String "The part."
  field control String "The control it took."
  field binding InputBinding "The slot's binding with that control in it."
  field conflicts List.BindingConflict "What applying it would share a control with."
}
```

<sub>`api.ogt` line 1781</sub>

<a id="type-InputGlyph"></a>

### `InputGlyph` (struct)

A control's glyph, for prompts and a controls screen.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `icon` | `string` | `""` | — | — | — | The icon a style's set draws: `key.space`, `mouse.left`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `xbox.stickL`, `key.wasd`, `touch.stick`. |
| `text` | `string` | `""` | — | — | — | The text that names it: `Space`, `LMB`, `A`, `×`, `L Stick`, `W A S D`. |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | — | The device it is drawn for. |
| `control` | `string` | `""` | — | — | — | The control it names (a composite's: its parts', space-separated). |

```ogt
struct InputGlyph {
  doc "A control's glyph, for prompts and a controls screen."
  field icon String "The icon a style's set draws: `key.space`, `mouse.left`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `xbox.stickL`, `key.wasd`, `touch.stick`."
  field text String "The text that names it: `Space`, `LMB`, `A`, `×`, `L Stick`, `W A S D`."
  field device InputDevice "The device it is drawn for."
  field control String "The control it names (a composite's: its parts', space-separated)."
}
```

<sub>`api.ogt` line 1794</sub>

<a id="type-SensitivityDesc"></a>

### `SensitivityDesc` (desc struct)

The player's sensitivity and inversion of an axis action on one device class: a look action's mouse and its stick each have their own (a settings screen's Mouse sensitivity and Stick sensitivity sliders, Invert Y). Kept by og.input.saveBindings, forgotten by og.input.resetBindings; a world whose player set none saves and hashes as before.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action (axis or axis2). |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | — | The device class it is for: keyboardMouse (the mouse's movement and the wheel; inversion turns the keys over too), any pad family (every pad), or touch (a touch stick). |
| `scale` | `number` | `1` | 0.01 … 100 | — | — | Multiplies the value of the action's analog bindings on that class (the mouse's movement, the wheel, a stick, a trigger, a touch stick), after each binding's own scale: 2 turns twice as far for the same mouse movement. Keys, buttons, the d-pad and composites read whole. |
| `invertX` | `boolean` | `false` | — | — | — | The value's x (an axis action's only value) is turned over. |
| `invertY` | `boolean` | `false` | — | — | — | A two-axis value's y is turned over: inverted look (an action's y is up-positive, so the mouse moved up then reads down). |

```ogt
struct SensitivityDesc {
  doc "The player's sensitivity and inversion of an axis action on one device class: a look action's mouse and its stick each have their own (a settings screen's Mouse sensitivity and Stick sensitivity sliders, Invert Y). Kept by og.input.saveBindings, forgotten by og.input.resetBindings; a world whose player set none saves and hashes as before."
  desc true
  field action String "The action (axis or axis2)."
  field device InputDevice "The device class it is for: keyboardMouse (the mouse's movement and the wheel; inversion turns the keys over too), any pad family (every pad), or touch (a touch stick)."
  field scale f32 "Multiplies the value of the action's analog bindings on that class (the mouse's movement, the wheel, a stick, a trigger, a touch stick), after each binding's own scale: 2 turns twice as far for the same mouse movement. Keys, buttons, the d-pad and composites read whole." {
    default 1
    min 0.01
    max 100
  }
  field invertX bool "The value's x (an axis action's only value) is turned over."
  field invertY bool "A two-axis value's y is turned over: inverted look (an action's y is up-positive, so the mouse moved up then reads down)."
}
```

<sub>`api.ogt` line 1802</sub>

<a id="type-KeyLabel"></a>

### `KeyLabel` (desc struct)

A key and what the player's keyboard layout prints on it: `KeyQ` and `a` on AZERTY.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `code` | `string` | `""` | — | — | — | The physical key (KeyboardEvent.code). |
| `label` | `string` | `""` | — | — | — | The character it types without modifiers, as the layout map gives it (`a`, `,`, `ö`). |

```ogt
struct KeyLabel {
  doc "A key and what the player's keyboard layout prints on it: `KeyQ` and `a` on AZERTY."
  desc true
  field code String "The physical key (KeyboardEvent.code)."
  field label String "The character it types without modifiers, as the layout map gives it (`a`, `,`, `ö`)."
}
```

<sub>`api.ogt` line 1816</sub>

<a id="type-MouseDesc"></a>

### `MouseDesc` (desc struct)

How the host reads the mouse while the pointer is locked (og.PointerLock): the player's setting, kept by the game with its settings (og.platform.saveSettings). Host only: none of it is world state.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `raw` | `boolean` | `true` | — | — | — | Raw input: ask the browser for the mouse's movement without the OS's pointer acceleration (requestPointerLock with unadjustedMovement), where it supports that (Chrome and Edge on Windows, macOS and ChromeOS); elsewhere the lock is the plain one and the movement keeps the OS's acceleration (MouseInfo.rawUnsupported). Takes effect at the next lock. |

```ogt
struct MouseDesc {
  doc "How the host reads the mouse while the pointer is locked (og.PointerLock): the player's setting, kept by the game with its settings (og.platform.saveSettings). Host only: none of it is world state."
  desc true
  field raw bool "Raw input: ask the browser for the mouse's movement without the OS's pointer acceleration (requestPointerLock with unadjustedMovement), where it supports that (Chrome and Edge on Windows, macOS and ChromeOS); elsewhere the lock is the plain one and the movement keeps the OS's acceleration (MouseInfo.rawUnsupported). Takes effect at the next lock." {
    default true
  }
}
```

<sub>`api.ogt` line 1823</sub>

<a id="type-MouseInfo"></a>

### `MouseInfo` (struct)

The mouse as the host reads it (og.input.mouse).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `raw` | `boolean` | `false` | — | — | — | Raw input is asked for (MouseDesc.raw). |
| `rawActive` | `boolean` | `false` | — | — | — | The lock now delivers raw movement: the browser granted unadjustedMovement. False while unlocked. |
| `rawUnsupported` | `boolean` | `false` | — | — | — | The browser refused unadjustedMovement (Firefox, Safari, Linux): its lock moves with the OS's acceleration. A settings screen shows the Raw input switch as unavailable then. |

```ogt
struct MouseInfo {
  doc "The mouse as the host reads it (og.input.mouse)."
  field raw bool "Raw input is asked for (MouseDesc.raw)."
  field rawActive bool "The lock now delivers raw movement: the browser granted unadjustedMovement. False while unlocked."
  field rawUnsupported bool "The browser refused unadjustedMovement (Firefox, Safari, Linux): its lock moves with the OS's acceleration. A settings screen shows the Raw input switch as unavailable then."
}
```

<sub>`api.ogt` line 1831</sub>

<a id="type-DeadzoneDesc"></a>

### `DeadzoneDesc` (desc struct)

A pad's dead zones: what its sticks and triggers read as rest. The player's to set (a settings slider); og.input.saveBindings keeps them.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `-1` | -1 … 7 | — | — | The pad, 0 to 7; -1 every pad. |
| `stick` | `number` | `0.15` | 0 … 0.95 | — | — | The sticks' (radial for a stick's two axes). |
| `trigger` | `number` | `0.05` | 0 … 0.95 | — | — | The triggers'. |

```ogt
struct DeadzoneDesc {
  doc "A pad's dead zones: what its sticks and triggers read as rest. The player's to set (a settings slider); og.input.saveBindings keeps them."
  desc true
  field pad i32 "The pad, 0 to 7; -1 every pad." {
    default -1
    min -1
    max 7
  }
  field stick f32 "The sticks' (radial for a stick's two axes)." {
    default 0.15
    min 0
    max 0.95
  }
  field trigger f32 "The triggers'." {
    default 0.05
    min 0
    max 0.95
  }
}
```

<sub>`api.ogt` line 1838</sub>

<a id="type-RumbleDesc"></a>

### `RumbleDesc` (desc struct)

A rumble of a pad's motors: output for the host, never world state (a replay does not repeat it).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `-1` | -1 … 7 | — | — | The pad, 0 to 7; -1 every connected pad. |
| `low` | `number` | `0` | 0 … 1 | `ratio` | — | The heavy (low-frequency) motor. |
| `high` | `number` | `0` | 0 … 1 | `ratio` | — | The light (high-frequency) motor. |
| `ms` | `number` | `200` | ≥ 0 | `ms` | — | How long; 0 stops the pad's rumble. |

```ogt
struct RumbleDesc {
  doc "A rumble of a pad's motors: output for the host, never world state (a replay does not repeat it)."
  desc true
  field pad i32 "The pad, 0 to 7; -1 every connected pad." {
    default -1
    min -1
    max 7
  }
  field low f32 "The heavy (low-frequency) motor." {
    min 0
    max 1
    unit ratio
  }
  field high f32 "The light (high-frequency) motor." {
    min 0
    max 1
    unit ratio
  }
  field ms f32 "How long; 0 stops the pad's rumble." {
    default 200
    min 0
    unit ms
  }
}
```

<sub>`api.ogt` line 1858</sub>

<a id="type-GamepadButton"></a>

### `GamepadButton` (enum)

A gamepad button in the W3C standard mapping, in its index order (a is button 0).

A string: one of the variants below. The zero value is `"a"`.

| Variant | Doc |
|---|---|
| `"a"` | The bottom face button (A, Cross). |
| `"b"` | The right face button (B, Circle). |
| `"x"` | The left face button (X, Square). |
| `"y"` | The top face button (Y, Triangle). |
| `"leftBumper"` | The left shoulder button. |
| `"rightBumper"` | The right shoulder button. |
| `"leftTrigger"` | The left trigger, as a button (its analog value is the leftTrigger axis). |
| `"rightTrigger"` | The right trigger, as a button. |
| `"back"` | Back, Select or View. |
| `"start"` | Start, Options or Menu. |
| `"leftStick"` | The left stick pressed in. |
| `"rightStick"` | The right stick pressed in. |
| `"up"` | The d-pad's up. |
| `"down"` | The d-pad's down. |
| `"left"` | The d-pad's left. |
| `"right"` | The d-pad's right. |
| `"home"` | The home or guide button. |

```ogt
enum GamepadButton {
  doc "A gamepad button in the W3C standard mapping, in its index order (a is button 0)."
  variant a "The bottom face button (A, Cross)."
  variant b "The right face button (B, Circle)."
  variant x "The left face button (X, Square)."
  variant y "The top face button (Y, Triangle)."
  variant leftBumper "The left shoulder button."
  variant rightBumper "The right shoulder button."
  variant leftTrigger "The left trigger, as a button (its analog value is the leftTrigger axis)."
  variant rightTrigger "The right trigger, as a button."
  variant back "Back, Select or View."
  variant start "Start, Options or Menu."
  variant leftStick "The left stick pressed in."
  variant rightStick "The right stick pressed in."
  variant up "The d-pad's up."
  variant down "The d-pad's down."
  variant left "The d-pad's left."
  variant right "The d-pad's right."
  variant home "The home or guide button."
}
```

<sub>`api.ogt` line 2128</sub>

<a id="type-GamepadAxis"></a>

### `GamepadAxis` (enum)

An analog input of a gamepad.

A string: one of the variants below. The zero value is `"leftX"`.

| Variant | Doc |
|---|---|
| `"leftX"` | The left stick's x, -1 (left) to 1 (right). |
| `"leftY"` | The left stick's y, -1 (up) to 1 (down), as the Gamepad API reports it. |
| `"rightX"` | The right stick's x. |
| `"rightY"` | The right stick's y, down positive. |
| `"leftTrigger"` | The left trigger, 0 to 1. |
| `"rightTrigger"` | The right trigger, 0 to 1. |

```ogt
enum GamepadAxis {
  doc "An analog input of a gamepad."
  variant leftX "The left stick's x, -1 (left) to 1 (right)."
  variant leftY "The left stick's y, -1 (up) to 1 (down), as the Gamepad API reports it."
  variant rightX "The right stick's x."
  variant rightY "The right stick's y, down positive."
  variant leftTrigger "The left trigger, 0 to 1."
  variant rightTrigger "The right trigger, 0 to 1."
}
```

<sub>`api.ogt` line 2149</sub>

<a id="type-KeyAction"></a>

### `KeyAction` (enum)

What an injected key event does.

A string: one of the variants below. The zero value is `"press"`.

| Variant | Doc |
|---|---|
| `"press"` | Down, then up on the next tick. |
| `"down"` | Held down until an up. |
| `"up"` | Released. |

```ogt
enum KeyAction {
  doc "What an injected key event does."
  variant press "Down, then up on the next tick."
  variant down "Held down until an up."
  variant up "Released."
}
```

<sub>`api.ogt` line 2159</sub>

<a id="type-PointerAction"></a>

### `PointerAction` (enum)

What an injected pointer event does.

A string: one of the variants below. The zero value is `"click"`.

| Variant | Doc |
|---|---|
| `"click"` | Down, then up on the next tick, at the point. |
| `"move"` | A move to the point. |
| `"down"` | A press at the point. |
| `"up"` | A release at the point. |

```ogt
enum PointerAction {
  doc "What an injected pointer event does."
  variant click "Down, then up on the next tick, at the point."
  variant move "A move to the point."
  variant down "A press at the point."
  variant up "A release at the point."
}
```

<sub>`api.ogt` line 2166</sub>

<a id="type-PointerButton"></a>

### `PointerButton` (enum)

A physical pointer button; `none` is a hover move.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No button. |
| `"left"` | The primary button, or a touch. |
| `"middle"` | The middle button. |
| `"right"` | The secondary button. |

```ogt
enum PointerButton {
  doc "A physical pointer button; `none` is a hover move."
  variant none "No button."
  variant left "The primary button, or a touch."
  variant middle "The middle button."
  variant right "The secondary button."
}
```

<sub>`api.ogt` line 2174</sub>

<a id="type-Modifiers"></a>

### `Modifiers` (struct)

Modifier keys held during a key event.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `alt` | `boolean` | `false` | — | — | — | Alt or Option. |
| `control` | `boolean` | `false` | — | — | — | Control. |
| `platform` | `boolean` | `false` | — | — | — | Command on macOS, the Windows key on Windows. |
| `shift` | `boolean` | `false` | — | — | — | Shift. |

```ogt
struct Modifiers {
  doc "Modifier keys held during a key event."
  field alt bool "Alt or Option."
  field control bool "Control."
  field platform bool "Command on macOS, the Windows key on Windows."
  field shift bool "Shift."
}
```

<sub>`api.ogt` line 2182</sub>

<a id="type-KeyInput"></a>

### `KeyInput` (struct)

A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `action` | `KeyAction` | `"press"` | — | — | — | Press, down or up. |
| `modifiers` | `Modifiers` | `{ alt: false, control: false, platform: false, shift: false }` | — | — | optional | Held modifiers; omitted means none. |
| `at` | `number` | `0` | -1 … 1 | — | optional | When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5. |

```ogt
struct KeyInput {
  doc "A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA."
  field key String "The physical key code."
  field action KeyAction "Press, down or up."
  field modifiers Modifiers "Held modifiers; omitted means none." {
    optional true
  }
  field at f32 "When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5." {
    min -1
    max 1
    optional true
  }
}
```

<sub>`api.ogt` line 2190</sub>

<a id="type-PointerInput"></a>

### `PointerInput` (struct)

A pointer event at a point in CSS pixels of the play surface.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `PointerAction` | `"click"` | — | — | — | Click, move, down or up. |
| `x` | `number` | `0` | — | `cssPx` | — | Horizontal position. |
| `y` | `number` | `0` | — | `cssPx` | — | Vertical position. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button; `none` for a hover move. |
| `id` | `number` | `0` | ≤ 4 | — | optional | Which pointer: 0 the primary one (the mouse, the first touch; what protocol 5 means), 1 to 4 further touches held at the same time (an engine extension). Omitted is 0. |
| `at` | `number` | `0` | -1 … 1 | — | optional | When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5. |
| `movementX` | `number` | `0` | — | `cssPx` | optional | The mouse's relative movement since its last event (the browser's movementX), which keeps coming while the pointer is locked and x, y stand still: og.Pointers.movement sums pointer 0's over the tick for mouse-look. Omitted is 0. An engine extension of protocol 5. |
| `movementY` | `number` | `0` | — | `cssPx` | optional | The relative vertical movement (movementY, down positive). Omitted is 0. |

```ogt
struct PointerInput {
  doc "A pointer event at a point in CSS pixels of the play surface."
  field action PointerAction "Click, move, down or up."
  field x f32 "Horizontal position." {
    unit cssPx
  }
  field y f32 "Vertical position." {
    unit cssPx
  }
  field button PointerButton "The button; `none` for a hover move."
  field id u32 "Which pointer: 0 the primary one (the mouse, the first touch; what protocol 5 means), 1 to 4 further touches held at the same time (an engine extension). Omitted is 0." {
    max 4
    optional true
  }
  field at f32 "When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5." {
    min -1
    max 1
    optional true
  }
  field movementX f32 "The mouse's relative movement since its last event (the browser's movementX), which keeps coming while the pointer is locked and x, y stand still: og.Pointers.movement sums pointer 0's over the tick for mouse-look. Omitted is 0. An engine extension of protocol 5." {
    unit cssPx
    optional true
  }
  field movementY f32 "The relative vertical movement (movementY, down positive). Omitted is 0." {
    unit cssPx
    optional true
  }
}
```

<sub>`api.ogt` line 2204</sub>

<a id="type-PointerLockReport"></a>

### `PointerLockReport` (struct)

The host reports whether the pointer is locked to the play surface now (an engine extension of protocol 5): the web shell sends one from the page's pointerlockchange, and a playtest can send its own; og.PointerLock.locked follows it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `locked` | `boolean` | `false` | — | — | — | Locked now (true) or released (false). |

```ogt
struct PointerLockReport {
  doc "The host reports whether the pointer is locked to the play surface now (an engine extension of protocol 5): the web shell sends one from the page's pointerlockchange, and a playtest can send its own; og.PointerLock.locked follows it."
  field locked bool "Locked now (true) or released (false)."
}
```

<sub>`api.ogt` line 2233</sub>

<a id="type-GamepadInput"></a>

### `GamepadInput` (struct)

A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `0` | ≤ 7 | — | optional | Which pad, 0 to 7; omitted is 0. |
| `connected` | `boolean` | `false` | — | — | optional | Whether the pad is connected (omitted is true); a disconnected pad holds nothing. |
| `buttons` | `readonly GamepadButton[]` | `[]` | — | — | optional | The buttons held (omitted is none). |
| `axes` | `readonly number[]` | `[]` | — | — | optional | Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0. |
| `triggers` | `readonly number[]` | `[]` | — | — | optional | The left and right analog triggers, 0 to 1. Missing values are 0. |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | optional | The pad's family as the host tells it from the pad's id (xbox, playstation, switch, or gamepad for a pad it cannot place); omitted keeps what was said. Glyphs follow it. |

```ogt
struct GamepadInput {
  doc "A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own."
  field pad u32 "Which pad, 0 to 7; omitted is 0." {
    max 7
    optional true
  }
  field connected bool "Whether the pad is connected (omitted is true); a disconnected pad holds nothing." {
    optional true
  }
  field buttons List.GamepadButton "The buttons held (omitted is none)." {
    optional true
  }
  field axes List.f32 "Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0." {
    optional true
  }
  field triggers List.f32 "The left and right analog triggers, 0 to 1. Missing values are 0." {
    optional true
  }
  field device InputDevice "The pad's family as the host tells it from the pad's id (xbox, playstation, switch, or gamepad for a pad it cannot place); omitted keeps what was said. Glyphs follow it." {
    optional true
  }
}
```

<sub>`api.ogt` line 2238</sub>

<a id="type-WheelInput"></a>

### `WheelInput` (struct)

A wheel or trackpad scroll at a point.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Horizontal position. |
| `y` | `number` | `0` | — | `cssPx` | — | Vertical position. |
| `deltaX` | `number` | `0` | — | `cssPx` | — | Horizontal scroll amount. |
| `deltaY` | `number` | `0` | — | `cssPx` | — | Vertical scroll amount. |

```ogt
struct WheelInput {
  doc "A wheel or trackpad scroll at a point."
  field x f32 "Horizontal position." {
    unit cssPx
  }
  field y f32 "Vertical position." {
    unit cssPx
  }
  field deltaX f32 "Horizontal scroll amount." {
    unit cssPx
  }
  field deltaY f32 "Vertical scroll amount." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2261</sub>

<a id="type-TextEntry"></a>

### `TextEntry` (struct)

Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's composition so far. Raises TextInput events; the UI's focused text field takes them.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text. |
| `composing` | `boolean` | `false` | — | — | optional | An input method's composition, not yet committed: it replaces the composition before it, and empty ends it. Omitted is false: committed text. |

```ogt
struct TextEntry {
  doc "Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's composition so far. Raises TextInput events; the UI's focused text field takes them."
  field text String "The text."
  field composing bool "An input method's composition, not yet committed: it replaces the composition before it, and empty ends it. Omitted is false: committed text." {
    optional true
  }
}
```

<sub>`api.ogt` line 2277</sub>

<a id="type-A11yActionKind"></a>

### `A11yActionKind` (enum)

What an assistive technology asks of a UI control.

A string: one of the variants below. The zero value is `"focus"`.

| Variant | Doc |
|---|---|
| `"focus"` | Move the UI's focus to it (with the focus ring). |
| `"activate"` | Press it, as Enter on it would (a text control takes the focus). |
| `"increment"` | Step an adjustable control (a slider, a stepper) up. |
| `"decrement"` | Step an adjustable control down. |

```ogt
enum A11yActionKind {
  doc "What an assistive technology asks of a UI control."
  variant focus "Move the UI's focus to it (with the focus ring)."
  variant activate "Press it, as Enter on it would (a text control takes the focus)."
  variant increment "Step an adjustable control (a slider, a stepper) up."
  variant decrement "Step an adjustable control down."
}
```

<sub>`api.ogt` line 2285</sub>

<a id="type-A11yRequest"></a>

### `A11yRequest` (struct)

A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. Raises an A11yAction event; the UI acts on it at the start of the next frame, against the last frame's controls (a path naming no focusable control of the topmost dialog is ignored; increment and decrement also take a stepper's spinbutton, whose buttons are the focusable controls).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `A11yActionKind` | `"focus"` | — | — | — | What is asked. |
| `path` | `string` | `""` | — | — | — | The control's path, as og.ui.a11y() names it. |

```ogt
struct A11yRequest {
  doc "A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. Raises an A11yAction event; the UI acts on it at the start of the next frame, against the last frame's controls (a path naming no focusable control of the topmost dialog is ignored; increment and decrement also take a stepper's spinbutton, whose buttons are the focusable controls)."
  field action A11yActionKind "What is asked."
  field path String "The control's path, as og.ui.a11y() names it."
}
```

<sub>`api.ogt` line 2293</sub>

<a id="type-InjectedInput"></a>

### `InjectedInput` (union)

One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "key"` | [`KeyInput`](#type-KeyInput) | A key event. |
| `kind: "pointer"` | [`PointerInput`](#type-PointerInput) | A pointer event. |
| `kind: "wheel"` | [`WheelInput`](#type-WheelInput) | A wheel event. |
| `kind: "gamepad"` | [`GamepadInput`](#type-GamepadInput) | A gamepad's state. |
| `kind: "text"` | [`TextEntry`](#type-TextEntry) | Typed or composed text. |
| `kind: "a11y"` | [`A11yRequest`](#type-A11yRequest) | A screen reader's request on a UI control. |
| `kind: "pointerLock"` | [`PointerLockReport`](#type-PointerLockReport) | The host's word on the pointer lock (og.PointerLock.locked). |

```ogt
union InjectedInput {
  doc "One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it."
  tag kind
  case key KeyInput "A key event."
  case pointer PointerInput "A pointer event."
  case wheel WheelInput "A wheel event."
  case gamepad GamepadInput "A gamepad's state."
  case text TextEntry "Typed or composed text."
  case a11y A11yRequest "A screen reader's request on a UI control."
  case pointerLock PointerLockReport "The host's word on the pointer lock (og.PointerLock.locked)."
}
```

<sub>`api.ogt` line 2299</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_POINTERS"></a>`MAX_POINTERS` | `number` | `5` | The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. |
| <a id="const-MAX_GAMEPADS"></a>`MAX_GAMEPADS` | `number` | `8` | The most gamepads og.Gamepad reads: pads 0 to 7. |

```ogt
const MAX_POINTERS u32 5 "The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4."
```

<sub>`api.ogt` line 1605</sub>

```ogt
const MAX_GAMEPADS u32 8 "The most gamepads og.Gamepad reads: pads 0 to 7."
```

<sub>`api.ogt` line 1606</sub>
