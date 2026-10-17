<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `input` module

Input as the game sees it: the Input resource for polling, events for edges.

The shell collects pointer, keyboard and wheel input on the main thread; the probe and the command channel inject through the same path.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module input {
  doc "Input as the game sees it: the Input resource for polling, events for edges."
  doc "The shell collects pointer, keyboard and wheel input on the main thread; the probe and the command channel inject through the same path."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 634</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`KeyAction`](#type-KeyAction) | enum | What an injected key event does. |
| [`PointerAction`](#type-PointerAction) | enum | What an injected pointer event does. |
| [`PointerButton`](#type-PointerButton) | enum | A physical pointer button; `none` is a hover move. |
| [`Modifiers`](#type-Modifiers) | struct | Modifier keys held during a key event. |
| [`KeyInput`](#type-KeyInput) | struct | A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA. |
| [`PointerInput`](#type-PointerInput) | struct | A pointer event at a point in CSS pixels of the play surface. |
| [`WheelInput`](#type-WheelInput) | struct | A wheel or trackpad scroll at a point. |
| [`InjectedInput`](#type-InjectedInput) | union | One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it. |
| [`og.Input`](#record-Input) | resource | The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods. |
| [`og.KeyDown`](#record-KeyDown) | event | A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems. |
| [`og.KeyUp`](#record-KeyUp) | event | A key went up. |
| [`og.PointerDown`](#record-PointerDown) | event | A pointer button went down. |
| [`og.PointerUp`](#record-PointerUp) | event | A pointer button went up. |
| [`og.PointerMove`](#record-PointerMove) | event | The pointer moved. |
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

<sub>`api.ogt` line 714</sub>

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

<sub>`api.ogt` line 788</sub>

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

<sub>`api.ogt` line 793</sub>

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

<sub>`api.ogt` line 798</sub>

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

<sub>`api.ogt` line 803</sub>

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

<sub>`api.ogt` line 809</sub>

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

<sub>`api.ogt` line 817</sub>

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

<sub>`api.ogt` line 822</sub>

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

<sub>`api.ogt` line 827</sub>

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

<sub>`api.ogt` line 834</sub>

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

<sub>`api.ogt` line 840</sub>

<a id="record-PointerDown"></a>

### `og.PointerDown` (event)

A pointer button went down.

Token `og.PointerDown`, registered under the schema name `og.PointerDown`; its value type is `PointerDown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button. |

```ogt
event PointerDown {
  doc "A pointer button went down."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
}
```

<sub>`api.ogt` line 845</sub>

<a id="record-PointerUp"></a>

### `og.PointerUp` (event)

A pointer button went up.

Token `og.PointerUp`, registered under the schema name `og.PointerUp`; its value type is `PointerUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button. |

```ogt
event PointerUp {
  doc "A pointer button went up."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
}
```

<sub>`api.ogt` line 852</sub>

<a id="record-PointerMove"></a>

### `og.PointerMove` (event)

The pointer moved.

Token `og.PointerMove`, registered under the schema name `og.PointerMove`; its value type is `PointerMove`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |

```ogt
event PointerMove {
  doc "The pointer moved."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
}
```

<sub>`api.ogt` line 859</sub>

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

<sub>`api.ogt` line 865</sub>

## Types

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

<sub>`api.ogt` line 638</sub>

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

<sub>`api.ogt` line 645</sub>

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

<sub>`api.ogt` line 653</sub>

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

<sub>`api.ogt` line 661</sub>

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

<sub>`api.ogt` line 669</sub>

<a id="type-PointerInput"></a>

### `PointerInput` (struct)

A pointer event at a point in CSS pixels of the play surface.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `PointerAction` | `"click"` | — | — | — | Click, move, down or up. |
| `x` | `number` | `0` | — | `cssPx` | — | Horizontal position. |
| `y` | `number` | `0` | — | `cssPx` | — | Vertical position. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button; `none` for a hover move. |

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
}
```

<sub>`api.ogt` line 678</sub>

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

<sub>`api.ogt` line 690</sub>

<a id="type-InjectedInput"></a>

### `InjectedInput` (union)

One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "key"` | [`KeyInput`](#type-KeyInput) | A key event. |
| `kind: "pointer"` | [`PointerInput`](#type-PointerInput) | A pointer event. |
| `kind: "wheel"` | [`WheelInput`](#type-WheelInput) | A wheel event. |

```ogt
union InjectedInput {
  doc "One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it."
  tag kind
  case key KeyInput "A key event."
  case pointer PointerInput "A pointer event."
  case wheel WheelInput "A wheel event."
}
```

<sub>`api.ogt` line 706</sub>
