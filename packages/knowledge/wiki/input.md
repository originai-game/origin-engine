<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `input` module

Input as the game sees it: the Input resource for polling, events for edges.

The shell collects pointer, keyboard and wheel input on the main thread; the probe and the command channel inject through the same path.

Several pointers at once (multi-touch) are og.Pointers; the first four gamepads, in the W3C standard mapping, are og.Gamepad. Both are world state, as og.Input is, so they replay.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module input {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 683</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`MAX_POINTERS`](#const-MAX_POINTERS) | constant | The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. |
| [`MAX_GAMEPADS`](#const-MAX_GAMEPADS) | constant | The most gamepads og.Gamepad reads: pads 0 to 3. |
| [`GamepadButton`](#type-GamepadButton) | enum | A gamepad button in the W3C standard mapping, in its index order (a is button 0). |
| [`GamepadAxis`](#type-GamepadAxis) | enum | An analog input of a gamepad. |
| [`KeyAction`](#type-KeyAction) | enum | What an injected key event does. |
| [`PointerAction`](#type-PointerAction) | enum | What an injected pointer event does. |
| [`PointerButton`](#type-PointerButton) | enum | A physical pointer button; `none` is a hover move. |
| [`Modifiers`](#type-Modifiers) | struct | Modifier keys held during a key event. |
| [`KeyInput`](#type-KeyInput) | struct | A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA. |
| [`PointerInput`](#type-PointerInput) | struct | A pointer event at a point in CSS pixels of the play surface. |
| [`GamepadInput`](#type-GamepadInput) | struct | A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own. |
| [`WheelInput`](#type-WheelInput) | struct | A wheel or trackpad scroll at a point. |
| [`InjectedInput`](#type-InjectedInput) | union | One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it. |
| [`og.Input`](#record-Input) | resource | The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods. |
| [`og.Pointers`](#record-Pointers) | resource | Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id). |
| [`og.Gamepad`](#record-Gamepad) | resource | The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys. |
| [`og.KeyDown`](#record-KeyDown) | event | A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems. |
| [`og.KeyUp`](#record-KeyUp) | event | A key went up. |
| [`og.PointerDown`](#record-PointerDown) | event | A pointer button went down (or a touch landed). |
| [`og.PointerUp`](#record-PointerUp) | event | A pointer button went up (or a touch lifted). |
| [`og.PointerMove`](#record-PointerMove) | event | A pointer moved. |
| [`og.Wheel`](#record-Wheel) | event | A wheel or trackpad scroll. |

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

<sub>`api.ogt` line 824</sub>

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

<sub>`api.ogt` line 898</sub>

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

<sub>`api.ogt` line 903</sub>

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

<sub>`api.ogt` line 908</sub>

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

<sub>`api.ogt` line 913</sub>

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

<sub>`api.ogt` line 919</sub>

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

<sub>`api.ogt` line 927</sub>

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

<sub>`api.ogt` line 932</sub>

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

<sub>`api.ogt` line 937</sub>

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

<sub>`api.ogt` line 944</sub>

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

<sub>`api.ogt` line 999</sub>

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

<sub>`api.ogt` line 1004</sub>

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

<sub>`api.ogt` line 1009</sub>

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

<sub>`api.ogt` line 1014</sub>

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

<sub>`api.ogt` line 1019</sub>

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

<sub>`api.ogt` line 1024</sub>

<a id="record-Gamepad"></a>

### `og.Gamepad` (resource)

The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.

Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods.

Token `og.Gamepad`, registered under the schema name `og.Gamepad`; its value type is `Gamepad`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `connectedMask` | `number` | `0` | — | — | read only | Connected pads: bit `pad`. |
| `buttons0` | `number` | `0` | — | — | read only | Held buttons of pad 0. |
| `buttons1` | `number` | `0` | — | — | read only | Held buttons of pad 1. |
| `buttons2` | `number` | `0` | — | — | read only | Held buttons of pad 2. |
| `buttons3` | `number` | `0` | — | — | read only | Held buttons of pad 3. |
| `pressed0` | `number` | `0` | — | — | read only | Buttons of pad 0 that went down this tick. |
| `pressed1` | `number` | `0` | — | — | read only | Buttons of pad 1 that went down this tick. |
| `pressed2` | `number` | `0` | — | — | read only | Buttons of pad 2 that went down this tick. |
| `pressed3` | `number` | `0` | — | — | read only | Buttons of pad 3 that went down this tick. |
| `released0` | `number` | `0` | — | — | read only | Buttons of pad 0 that went up this tick. |
| `released1` | `number` | `0` | — | — | read only | Buttons of pad 1 that went up this tick. |
| `released2` | `number` | `0` | — | — | read only | Buttons of pad 2 that went up this tick. |
| `released3` | `number` | `0` | — | — | read only | Buttons of pad 3 that went up this tick. |
| `leftStick0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 0, -1 to 1, y down. |
| `leftStick1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 1, -1 to 1, y down. |
| `leftStick2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 2, -1 to 1, y down. |
| `leftStick3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 3, -1 to 1, y down. |
| `rightStick0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 0, -1 to 1, y down. |
| `rightStick1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 1, -1 to 1, y down. |
| `rightStick2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 2, -1 to 1, y down. |
| `rightStick3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 3, -1 to 1, y down. |
| `triggers0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 0, 0 to 1. |
| `triggers1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 1, 0 to 1. |
| `triggers2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 2, 0 to 1. |
| `triggers3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 3, 0 to 1. |

```ogt
resource Gamepad {
  doc "The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys."
  doc "Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods."
  field connectedMask u32 "Connected pads: bit `pad`." {
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
  fn connected {
    doc "Whether a pad is connected."
    param pad u32 "The pad, 0 to 3; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn down {
    doc "Whether a button is held."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 3; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn pressed {
    doc "Whether a button went down this tick."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 3; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn released {
    doc "Whether a button went up this tick."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 3; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn axis {
    doc "An analog input: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Dead zones are the game's."
    param axis GamepadAxis "Which input."
    param pad u32 "The pad, 0 to 3; omitted is 0." {
      optional true
    }
    returns f32
  }
}
```

<sub>`api.ogt` line 1030</sub>

<a id="member-Gamepad-connected"></a>

#### `Gamepad.connected`

Whether a pad is connected.

```ts
Gamepad.connected(pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `pad` | `number` | yes | The pad, 0 to 3; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_connected` · publish builds: kept

```ogt
fn connected {
  doc "Whether a pad is connected."
  param pad u32 "The pad, 0 to 3; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 1108</sub>

<a id="member-Gamepad-down"></a>

#### `Gamepad.down`

Whether a button is held.

```ts
Gamepad.down(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 3; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_down` · publish builds: kept

```ogt
fn down {
  doc "Whether a button is held."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 3; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 1115</sub>

<a id="member-Gamepad-pressed"></a>

#### `Gamepad.pressed`

Whether a button went down this tick.

```ts
Gamepad.pressed(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 3; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_pressed` · publish builds: kept

```ogt
fn pressed {
  doc "Whether a button went down this tick."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 3; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 1123</sub>

<a id="member-Gamepad-released"></a>

#### `Gamepad.released`

Whether a button went up this tick.

```ts
Gamepad.released(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 3; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_released` · publish builds: kept

```ogt
fn released {
  doc "Whether a button went up this tick."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 3; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>`api.ogt` line 1131</sub>

<a id="member-Gamepad-axis"></a>

#### `Gamepad.axis`

An analog input: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Dead zones are the game's.

```ts
Gamepad.axis(axis: GamepadAxis, pad?: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `axis` | `GamepadAxis` | no | Which input. |
| `pad` | `number` | yes | The pad, 0 to 3; omitted is 0. |

**Returns** `number`.

Binding symbol `og_gamepad_axis` · publish builds: kept

```ogt
fn axis {
  doc "An analog input: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Dead zones are the game's."
  param axis GamepadAxis "Which input."
  param pad u32 "The pad, 0 to 3; omitted is 0." {
    optional true
  }
  returns f32
}
```

<sub>`api.ogt` line 1139</sub>

<a id="record-KeyDown"></a>

### `og.KeyDown` (event)

A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.

Token `og.KeyDown`, registered under the schema name `og.KeyDown`; its value type is `KeyDown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `repeat` | `boolean` | `false` | — | — | — | Whether this is an auto-repeat. |

```ogt
event KeyDown {
  doc "A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems."
  field key String "The physical key code."
  field repeat bool "Whether this is an auto-repeat."
}
```

<sub>`api.ogt` line 1149</sub>

<a id="record-KeyUp"></a>

### `og.KeyUp` (event)

A key went up.

Token `og.KeyUp`, registered under the schema name `og.KeyUp`; its value type is `KeyUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |

```ogt
event KeyUp {
  doc "A key went up."
  field key String "The physical key code."
}
```

<sub>`api.ogt` line 1155</sub>

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

```ogt
event PointerDown {
  doc "A pointer button went down (or a touch landed)."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers)."
}
```

<sub>`api.ogt` line 1160</sub>

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

```ogt
event PointerUp {
  doc "A pointer button went up (or a touch lifted)."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches."
}
```

<sub>`api.ogt` line 1168</sub>

<a id="record-PointerMove"></a>

### `og.PointerMove` (event)

A pointer moved.

Token `og.PointerMove`, registered under the schema name `og.PointerMove`; its value type is `PointerMove`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `id` | `number` | `0` | — | — | — | Which pointer: 0 the primary one, 1 to 4 further touches. |

```ogt
event PointerMove {
  doc "A pointer moved."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches."
}
```

<sub>`api.ogt` line 1176</sub>

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

<sub>`api.ogt` line 1183</sub>

## Types

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

<sub>`api.ogt` line 692</sub>

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

<sub>`api.ogt` line 713</sub>

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

<sub>`api.ogt` line 723</sub>

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

<sub>`api.ogt` line 730</sub>

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

<sub>`api.ogt` line 738</sub>

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

<sub>`api.ogt` line 746</sub>

<a id="type-KeyInput"></a>

### `KeyInput` (struct)

A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `action` | `KeyAction` | `"press"` | — | — | — | Press, down or up. |
| `modifiers` | `Modifiers` | `{ alt: false, control: false, platform: false, shift: false }` | — | — | optional | Held modifiers; omitted means none. |

```ogt
struct KeyInput {
  doc "A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA."
  field key String "The physical key code."
  field action KeyAction "Press, down or up."
  field modifiers Modifiers "Held modifiers; omitted means none." {
    optional true
  }
}
```

<sub>`api.ogt` line 754</sub>

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
}
```

<sub>`api.ogt` line 763</sub>

<a id="type-GamepadInput"></a>

### `GamepadInput` (struct)

A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `0` | ≤ 3 | — | optional | Which pad, 0 to 3; omitted is 0. |
| `connected` | `boolean` | `false` | — | — | optional | Whether the pad is connected (omitted is true); a disconnected pad holds nothing. |
| `buttons` | `readonly GamepadButton[]` | `[]` | — | — | optional | The buttons held (omitted is none). |
| `axes` | `readonly number[]` | `[]` | — | — | optional | Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0. |
| `triggers` | `readonly number[]` | `[]` | — | — | optional | The left and right analog triggers, 0 to 1. Missing values are 0. |

```ogt
struct GamepadInput {
  doc "A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own."
  field pad u32 "Which pad, 0 to 3; omitted is 0." {
    max 3
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
}
```

<sub>`api.ogt` line 779</sub>

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

<sub>`api.ogt` line 799</sub>

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

```ogt
union InjectedInput {
  doc "One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it."
  tag kind
  case key KeyInput "A key event."
  case pointer PointerInput "A pointer event."
  case wheel WheelInput "A wheel event."
  case gamepad GamepadInput "A gamepad's state."
}
```

<sub>`api.ogt` line 815</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_POINTERS"></a>`MAX_POINTERS` | `number` | `5` | The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. |
| <a id="const-MAX_GAMEPADS"></a>`MAX_GAMEPADS` | `number` | `4` | The most gamepads og.Gamepad reads: pads 0 to 3. |

```ogt
const MAX_POINTERS u32 5 "The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4."
```

<sub>`api.ogt` line 689</sub>

```ogt
const MAX_GAMEPADS u32 4 "The most gamepads og.Gamepad reads: pads 0 to 3."
```

<sub>`api.ogt` line 690</sub>
