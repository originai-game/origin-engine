<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-Input"></a>

### `og.Input` (resource)

The input state at the start of this tick (og-input's og.Input, version 3). Edges (justPressed, justReleased) last exactly one tick; read the state through the methods.

Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.

Token `og.Input`, registered under the schema name `og.Input`; its value type is `Input`.

#### Pointer

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pointer` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | The pointer position. |
| `pointerInside` | `boolean` | `false` | — | — | read only | Whether a pointer event has placed the pointer over the play surface. |
| `wheel` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Scroll accumulated during this tick. |
| `buttons` | `number` | `0` | — | — | read only | Held pointer buttons, as a mask. |
| `buttonsJustPressed` | `number` | `0` | — | — | read only | Buttons that went down this tick. |
| `buttonsJustReleased` | `number` | `0` | — | — | read only | Buttons that went up this tick. |
| `buttonsReleaseNext` | `number` | `0` | — | — | read only | Buttons a click releases at the end of the next tick. |

#### Keys held

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `keys0` | `number` | `0` | — | — | read only | Held keys, word 0: codes 0 to 31 of KEY_CODES. |
| `keys1` | `number` | `0` | — | — | read only | Held keys, word 1: codes 32 to 63 of KEY_CODES. |
| `keys2` | `number` | `0` | — | — | read only | Held keys, word 2: codes 64 to 95 of KEY_CODES. |
| `keys3` | `number` | `0` | — | — | read only | Held keys, word 3: codes 96 to 127 of KEY_CODES. |

#### Keys that went down this tick

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `justPressed0` | `number` | `0` | — | — | read only | Keys that went down this tick, word 0: codes 0 to 31 of KEY_CODES. |
| `justPressed1` | `number` | `0` | — | — | read only | Keys that went down this tick, word 1: codes 32 to 63 of KEY_CODES. |
| `justPressed2` | `number` | `0` | — | — | read only | Keys that went down this tick, word 2: codes 64 to 95 of KEY_CODES. |
| `justPressed3` | `number` | `0` | — | — | read only | Keys that went down this tick, word 3: codes 96 to 127 of KEY_CODES. |

#### Keys that went up this tick

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `justReleased0` | `number` | `0` | — | — | read only | Keys that went up this tick, word 0: codes 0 to 31 of KEY_CODES. |
| `justReleased1` | `number` | `0` | — | — | read only | Keys that went up this tick, word 1: codes 32 to 63 of KEY_CODES. |
| `justReleased2` | `number` | `0` | — | — | read only | Keys that went up this tick, word 2: codes 64 to 95 of KEY_CODES. |
| `justReleased3` | `number` | `0` | — | — | read only | Keys that went up this tick, word 3: codes 96 to 127 of KEY_CODES. |

#### Keys a press releases next tick

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `releaseNext0` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 0: codes 0 to 31 of KEY_CODES. |
| `releaseNext1` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 1: codes 32 to 63 of KEY_CODES. |
| `releaseNext2` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 2: codes 64 to 95 of KEY_CODES. |
| `releaseNext3` | `number` | `0` | — | — | read only | Keys a press releases at the end of the next tick, word 3: codes 96 to 127 of KEY_CODES. |

```ogt
resource Input {
  doc "The input state at the start of this tick (og-input's og.Input, version 3). Edges (justPressed, justReleased) last exactly one tick; read the state through the methods."
  doc "Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4."
  group "Pointer"
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
  field buttonsJustPressed u32 "Buttons that went down this tick." {
    readonly true
  }
  field buttonsJustReleased u32 "Buttons that went up this tick." {
    readonly true
  }
  field buttonsReleaseNext u32 "Buttons a click releases at the end of the next tick." {
    readonly true
  }
  group "Keys held"
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
  group "Keys that went down this tick"
  field justPressed0 u32 "Keys that went down this tick, word 0: codes 0 to 31 of KEY_CODES." {
    readonly true
  }
  field justPressed1 u32 "Keys that went down this tick, word 1: codes 32 to 63 of KEY_CODES." {
    readonly true
  }
  field justPressed2 u32 "Keys that went down this tick, word 2: codes 64 to 95 of KEY_CODES." {
    readonly true
  }
  field justPressed3 u32 "Keys that went down this tick, word 3: codes 96 to 127 of KEY_CODES." {
    readonly true
  }
  group "Keys that went up this tick"
  field justReleased0 u32 "Keys that went up this tick, word 0: codes 0 to 31 of KEY_CODES." {
    readonly true
  }
  field justReleased1 u32 "Keys that went up this tick, word 1: codes 32 to 63 of KEY_CODES." {
    readonly true
  }
  field justReleased2 u32 "Keys that went up this tick, word 2: codes 64 to 95 of KEY_CODES." {
    readonly true
  }
  field justReleased3 u32 "Keys that went up this tick, word 3: codes 96 to 127 of KEY_CODES." {
    readonly true
  }
  group "Keys a press releases next tick"
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
    doc "Whether a key is down."
    param key String "A KeyboardEvent.code name, such as KeyW or Space."
    returns bool
  }
  fn justPressed {
    doc "Whether a key went down this tick."
    param key String "A KeyboardEvent.code name."
    returns bool
  }
  fn justReleased {
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
    doc "Whether a pointer button is down."
    param button PointerButton "The button."
    returns bool
  }
  fn buttonJustPressed {
    doc "Whether a pointer button went down this tick."
    param button PointerButton "The button."
    returns bool
  }
  fn buttonJustReleased {
    doc "Whether a pointer button went up this tick."
    param button PointerButton "The button."
    returns bool
  }
}
```

<sub>Module `input` line 773</sub>

<a id="member-Input-down"></a>

#### `Input.down`

Whether a key is down.

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
  doc "Whether a key is down."
  param key String "A KeyboardEvent.code name, such as KeyW or Space."
  returns bool
}
```

<sub>Module `input` line 853</sub>

<a id="member-Input-justPressed"></a>

#### `Input.justPressed`

Whether a key went down this tick.

```ts
Input.justPressed(key: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | A KeyboardEvent.code name. |

**Returns** `boolean`.

Binding symbol `og_input_just_pressed` · publish builds: kept

```ogt
fn justPressed {
  doc "Whether a key went down this tick."
  param key String "A KeyboardEvent.code name."
  returns bool
}
```

<sub>Module `input` line 858</sub>

<a id="member-Input-justReleased"></a>

#### `Input.justReleased`

Whether a key went up this tick.

```ts
Input.justReleased(key: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | A KeyboardEvent.code name. |

**Returns** `boolean`.

Binding symbol `og_input_just_released` · publish builds: kept

```ogt
fn justReleased {
  doc "Whether a key went up this tick."
  param key String "A KeyboardEvent.code name."
  returns bool
}
```

<sub>Module `input` line 863</sub>

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

<sub>Module `input` line 868</sub>

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

<sub>Module `input` line 874</sub>

<a id="member-Input-buttonDown"></a>

#### `Input.buttonDown`

Whether a pointer button is down.

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
  doc "Whether a pointer button is down."
  param button PointerButton "The button."
  returns bool
}
```

<sub>Module `input` line 882</sub>

<a id="member-Input-buttonJustPressed"></a>

#### `Input.buttonJustPressed`

Whether a pointer button went down this tick.

```ts
Input.buttonJustPressed(button: PointerButton): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `PointerButton` | no | The button. |

**Returns** `boolean`.

Binding symbol `og_input_button_just_pressed` · publish builds: kept

```ogt
fn buttonJustPressed {
  doc "Whether a pointer button went down this tick."
  param button PointerButton "The button."
  returns bool
}
```

<sub>Module `input` line 887</sub>

<a id="member-Input-buttonJustReleased"></a>

#### `Input.buttonJustReleased`

Whether a pointer button went up this tick.

```ts
Input.buttonJustReleased(button: PointerButton): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `PointerButton` | no | The button. |

**Returns** `boolean`.

Binding symbol `og_input_button_just_released` · publish builds: kept

```ogt
fn buttonJustReleased {
  doc "Whether a pointer button went up this tick."
  param button PointerButton "The button."
  returns bool
}
```

<sub>Module `input` line 892</sub>
