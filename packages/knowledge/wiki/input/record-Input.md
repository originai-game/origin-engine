<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-Input"></a>

### `og.Input` (resource)

The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods.

Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.

Token `og.Input`, registered under the schema name `og.Input`; its value type is `Input`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pointer` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | The pointer position. |
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
    worldspace none
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

<sub>Module `input` line 757</sub>

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

<sub>Module `input` line 832</sub>

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

<sub>Module `input` line 837</sub>

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

<sub>Module `input` line 842</sub>

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

<sub>Module `input` line 847</sub>

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

<sub>Module `input` line 853</sub>

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

<sub>Module `input` line 861</sub>

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

<sub>Module `input` line 866</sub>

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

<sub>Module `input` line 871</sub>
